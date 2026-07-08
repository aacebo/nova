mod cmd;
mod error;
mod widgets;

use clap::Parser;
use cmd::*;
use error::*;

#[derive(Debug, Clone, Parser)]
#[command(name = "nova", about)]
struct Input {
    #[command(subcommand)]
    pub cmd: Cmd,
}

fn main() {
    let input = Input::parse();

    match input.cmd.run() {
        Ok(()) => {}
        Err(Error::Clap(err)) => {
            let _ = err.print();
            std::process::exit(err.exit_code());
        }
        Err(err) => {
            let widget = widgets::error::Widget::from(&err);
            widgets::print(&widget, widget.width(), widget.height());
            std::process::exit(1);
        }
    }
}
