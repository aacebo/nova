use figment::Figment;
use figment::providers::{Format, Yaml};

use crate::{Error, widgets};

#[derive(Debug, Clone, clap::Args)]
pub struct Args {
    pub files: Vec<String>,
}

impl Args {
    pub fn run(&self) -> Result<(), Error> {
        let mut figment = Figment::new();
        let mut matched = 0usize;

        for pattern in &self.files {
            for path in glob::glob(pattern)? {
                figment = figment.merge(Yaml::file(path?));
                matched += 1;
            }
        }

        if matched == 0 {
            return Err(Error::NotFound(self.files.clone()));
        }

        let manifest: nova::Manifest = figment.extract()?;
        let entrypoint = manifest.name.clone().unwrap_or_else(|| "main".into());
        let runtime = nova::Runtime::try_from(manifest)?;
        let output = runtime.call(&entrypoint, nova::Args::new())?;

        for diagnostic in &output.diagnostics {
            let widget = widgets::diagnostic::new(diagnostic);
            widgets::println(&widget, widget.width(), widget.height());
        }

        Ok(())
    }
}
