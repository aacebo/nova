use figment::Figment;
use figment::providers::{Format, Yaml};

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

        // entrypoints are the `run(..)`-triggered manifests, highest priority first
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

        let runtime = nova::Runtime::try_from(manifests)?;

        for (name, _) in &entries {
            let output = runtime.call(name, nova::KArgs::new())?;

            for diagnostic in &output.diagnostics {
                let widget = widgets::diagnostic::new(diagnostic);
                widgets::println(&widget, widget.width(), widget.height());
            }
        }

        Ok(())
    }
}
