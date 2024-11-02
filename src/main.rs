// TODO When opened outside of a terminal, the program should not open the terminal.
// However, when opened inside a terminal, the program should be able to output to the terminal.
// #![windows_subsystem = "windows"]

use std::io::IsTerminal;

use args::Args;
use clap::Parser;

mod app;
mod args;
mod context;

fn main() -> anyhow::Result<()> {
    let stdout = std::io::stdout();
    let args = stdout.is_terminal().then(Args::parse);
    let mut app = app::App::new(args);
    app.run()?;
    Ok(())
}
