// TODO When opened outside of a terminal, the program should not open the terminal.
// However, when opened inside a terminal, the program should be able to output to the terminal.
// #![windows_subsystem = "windows"]

use clap::Parser;
use cleave::args::Args;
use std::io::IsTerminal;

fn main() -> anyhow::Result<()> {
    let stdout = std::io::stdin();
    let args = stdout.is_terminal().then(Args::parse);
    let app = cleave::app::App::new(args)?;
    app.run()?;
    Ok(())
}
