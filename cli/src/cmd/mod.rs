pub mod run;

use clap::Subcommand;

use crate::Error;

#[derive(Debug, Clone, Subcommand)]
pub enum Cmd {
    #[command(about = "run a workflow")]
    Run(run::Args),
}

impl Cmd {
    pub fn run(&self) -> Result<(), Error> {
        match self {
            Self::Run(args) => args.run(),
        }
    }
}
