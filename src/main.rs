mod app;
mod config;
mod github;
mod json_auto_push;
mod main_cli;
mod model;
mod paths;
mod self_update;
mod ui;

use anyhow::Result;
use app::{App, AppEvent};
use crossterm::{
    event::{self, Event, KeyEventKind},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use main_cli::{parse_subcommand, Subcommand};
use ratatui::{backend::CrosstermBackend, Terminal};
use self_update::{build_commit_hash, run_self_update};
use std::{io, time::Duration};

#[cfg(test)]
#[path = "main_tests.rs"]
mod tests;

fn main() -> Result<()> {
    let args: Vec<String> = std::env::args().collect();
    match parse_subcommand(&args) {
        Some(Subcommand::Hash) => {
            println!("{}", build_commit_hash());
            return Ok(());
        }
        Some(Subcommand::Update) => {
            let should_exit = run_self_update()?;
            if should_exit {
                std::process::exit(0);
            }
            return Ok(());
        }
        None => {}
    }

    let mut app = App::load()?;

    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let result = run_app(&mut terminal, &mut app);

    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;
    result
}

fn run_app(terminal: &mut Terminal<CrosstermBackend<io::Stdout>>, app: &mut App) -> Result<()> {
    loop {
        app.tick();
        terminal.draw(|f| ui::render(f, app))?;

        if event::poll(Duration::from_millis(250))? {
            if let Event::Key(key) = event::read()? {
                app.note_raw_key_event(key);
                if key.kind != KeyEventKind::Press {
                    app.note_ignored_key_event(key);
                    continue;
                }
                if matches!(app.handle_key(key), AppEvent::Quit) {
                    break;
                }
            }
        }
    }

    Ok(())
}
