use std::sync::{Arc, Mutex};
use std::time::Duration;

use figment::Figment;
use figment::providers::{Format, Yaml};
use nova::codec::Codec;
use nova::fs::FileSystem;
use nova::http::Http;

use crate::widgets::board::{Board, Widget};
use crate::{Error, widgets};

#[derive(Debug, Clone, clap::Args)]
pub struct Args {
    pub files: Vec<String>,
}

impl Args {
    pub fn run(&self) -> Result<(), Error> {
        let mut manifests: Vec<nova::Manifest> = Vec::new();

        for pattern in &self.files {
            for path in glob::glob(pattern)? {
                let manifest: nova::Manifest = Figment::new().merge(Yaml::file(path?)).extract()?;
                manifests.push(manifest);
            }
        }

        if manifests.is_empty() {
            return Err(Error::NotFound(self.files.clone()));
        }

        let mut entries: Vec<(String, i64)> = Vec::new();

        for manifest in &manifests {
            let name = &manifest.name;

            if let Some(trigger) = manifest.on.iter().find(|t| t.is_run())
                && !entries.iter().any(|(n, _)| n == name)
            {
                entries.push((name.clone(), trigger.priority().unwrap_or(0)));
            }
        }

        entries.sort_by_key(|(_, priority)| std::cmp::Reverse(*priority));

        let names: Vec<String> = entries.iter().map(|(name, _)| name.clone()).collect();
        let board = Board::new(&names);
        let diagnostics: Arc<Mutex<Vec<nova::Diagnostic>>> = Default::default();
        let sink = diagnostics.clone();
        let mut builder = nova::new()
            .observe(board.clone())
            .observe(nova::event::on_diagnostic(move |d: &nova::Diagnostic| {
                sink.lock().unwrap().push(d.clone());
            }));

        builder = builder.fs().json().yaml().http();

        for manifest in manifests {
            builder = builder.routine(manifest);
        }

        let runtime = builder.build()?;
        let (done_tx, done_rx) = std::sync::mpsc::channel::<Result<(), String>>();
        let run = std::thread::scope(|scope| {
            scope.spawn(|| {
                for name in &names {
                    if let Err(err) = runtime.call(name, nova::args!()) {
                        let _ = done_tx.send(Err(err.to_string()));
                        return;
                    }
                }

                let _ = done_tx.send(Ok(()));
            });

            let mut painter = widgets::Painter::new();
            let mut tick = 0usize;
            let run = loop {
                match done_rx.recv_timeout(Duration::from_millis(100)) {
                    Ok(result) => break result,
                    Err(std::sync::mpsc::RecvTimeoutError::Timeout) => {
                        let state = board.state();
                        let widget = Widget::new(&state, tick, false);
                        painter.draw(&widget, widget.width(), widget.height());
                        tick += 1;
                    }
                    Err(std::sync::mpsc::RecvTimeoutError::Disconnected) => break Ok(()),
                }
            };

            let state = board.state();
            let widget = Widget::new(&state, tick, true);
            painter.finish(&widget, widget.width(), widget.height());
            drop(state);

            run
        });

        drop(runtime);
        run.map_err(|err| Error::Runtime(err.into()))?;

        for diagnostic in diagnostics.lock().unwrap().iter() {
            let widget = widgets::diagnostic::new(diagnostic);
            widgets::println(&widget, widget.width(), widget.height());
        }

        Ok(())
    }
}
