use figment::Figment;
use figment::providers::{Format, Yaml};

#[derive(Debug, Clone, clap::Args)]
pub struct Args {
    pub files: Vec<String>,
}

impl Args {
    pub fn run(&self) -> Result<(), Box<dyn std::error::Error>> {
        let mut figment = Figment::new();

        for pattern in &self.files {
            for path in glob::glob(&pattern)? {
                figment = figment.merge(Yaml::file(path?));
            }
        }

        let manifest: nova::Manifest = figment.extract()?;
        println!("{:#?}", manifest);
        Ok(())
    }
}
