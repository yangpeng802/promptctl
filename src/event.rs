use std::time::Duration;

use anyhow::Result;
use crossterm::event::{self, Event, KeyEventKind};
use ratatui::DefaultTerminal;

use crate::app::App;
use crate::ui;

/// Single-threaded TUI loop. `ratatui::init()` installs a panic hook that
/// restores the terminal, and `restore()` runs on every exit path.
///
/// The screen is redrawn on demand only (after an event or when the status
/// message expires), so an idle session emits no output and uses no CPU.
pub fn run(app: &mut App) -> Result<()> {
    let mut terminal = ratatui::init();
    let result = run_loop(&mut terminal, app);
    ratatui::restore();
    result
}

fn run_loop(terminal: &mut DefaultTerminal, app: &mut App) -> Result<()> {
    let mut needs_draw = true;
    loop {
        if needs_draw {
            terminal.draw(|frame| ui::render(frame, app))?;
            needs_draw = false;
        }

        // Expire the status message; redraw only when something changed.
        needs_draw |= app.tick();

        if event::poll(Duration::from_millis(120))? {
            match event::read()? {
                // Windows sends both press and release events.
                Event::Key(key) if key.kind == KeyEventKind::Press => app.handle_key(key),
                Event::Resize(_, _) => {}
                _ => {}
            }
            needs_draw = true;
        }

        if app.should_quit {
            return Ok(());
        }
    }
}
