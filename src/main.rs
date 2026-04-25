mod app;
mod config;
mod data_link;
mod github;
mod json_auto_push;
mod local_json_sync;
mod main_cli;
mod model;
mod paths;
mod process;
mod repo_url_cache;
mod self_update;
mod ui;

use anyhow::Result;
use app::{App, AppEvent};
use clap::Parser;
use crossterm::{
    event::{self, Event, KeyEventKind},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use main_cli::{Cli, Subcommand};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout},
    text::Line,
    widgets::{Block, Paragraph},
    Terminal,
};
use self_update::{build_commit_hash, check_self_update, run_self_update};
use std::{io, time::Duration};

#[cfg(test)]
#[path = "main_tests.rs"]
mod tests;

fn main() -> Result<()> {
    let cli = Cli::parse();
    match cli.command {
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
        Some(Subcommand::Check) => {
            println!("{}", check_self_update()?);
            return Ok(());
        }
        None => {}
    }

    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let result = run_terminal_session(&mut terminal);

    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;
    result
}

fn run_terminal_session(terminal: &mut Terminal<CrosstermBackend<io::Stdout>>) -> Result<()> {
    terminal.draw(render_boot_screen)?;
    let mut app = App::load()?;
    run_app(terminal, &mut app)
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

    if let Err(error) = app.persist_history() {
        eprintln!("warning: failed to persist history: {error}");
    }
    Ok(())
}

fn render_boot_screen(frame: &mut ratatui::Frame) {
    let area = frame.area();
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage(45),
            Constraint::Length(3),
            Constraint::Percentage(52),
        ])
        .split(area);

    let message = Paragraph::new(Line::from("own-repos-curator を起動中..."))
        .block(Block::bordered().title(" Loading "));
    frame.render_widget(message, chunks[1]);
}
