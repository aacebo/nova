mod cmd;

use clap::Parser;
use cmd::*;

#[derive(Debug, Clone, Parser)]
#[command(name = "nova", about)]
struct Input {
    #[command(subcommand)]
    pub cmd: Cmd,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let input = Input::parse();
    input.cmd.run()
}
