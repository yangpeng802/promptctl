use std::process::ExitCode;

use clap::Parser;

use pm::app::{App, PresetHint};
use pm::cli::{self, Cli};
use pm::config::Config;
use pm::event;
use pm::history::History;

fn main() -> ExitCode {
    let cli = Cli::parse();
    match cli::execute(cli) {
        cli::Outcome::Done(code) => code,
        cli::Outcome::Tui(hint) => match start_tui(hint) {
            Ok(()) => ExitCode::SUCCESS,
            Err(err) => {
                eprintln!("pm: {err:#}");
                ExitCode::FAILURE
            }
        },
    }
}

fn start_tui(hint: PresetHint) -> anyhow::Result<()> {
    use std::io::IsTerminal;
    if !std::io::stdin().is_terminal() && !std::io::stdout().is_terminal() {
        anyhow::bail!("no interactive terminal available for the TUI");
    }
    let (config, warning) = Config::load();
    if let Some(warning) = warning {
        eprintln!("pm: warning: {warning}");
    }
    let history = History::load();
    let mut app = App::new(config, history, hint);
    event::run(&mut app)
}
