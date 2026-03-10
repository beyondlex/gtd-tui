use std::fs;
use std::io;
use std::time::Duration;

use anyhow::{Result, anyhow};
use crossterm::event::{self, Event};
use crossterm::execute;
use crossterm::terminal::{
    EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode,
};
use ratatui::Terminal;
use ratatui::backend::CrosstermBackend;

mod app;
mod config;
mod ui;

use app::App;
use app::Keymap;
use config::{db_path, load};
use gtd_core::storage::SqliteStorage;
use ui::theme::{CalendarTheme, EditorTheme};

struct TerminalGuard;

impl TerminalGuard {
    fn enter() -> Result<Self> {
        enable_raw_mode()?;
        let mut stdout = io::stdout();
        execute!(stdout, EnterAlternateScreen)?;
        Ok(Self)
    }
}

impl Drop for TerminalGuard {
    fn drop(&mut self) {
        let _ = disable_raw_mode();
        let mut stdout = io::stdout();
        let _ = execute!(stdout, LeaveAlternateScreen);
    }
}

fn main() -> Result<()> {
    let _guard = TerminalGuard::enter()?;
    let mut stdout = io::stdout();
    let backend = CrosstermBackend::new(&mut stdout);
    let mut terminal = Terminal::new(backend)?;

    let db_path = db_path()?;
    if let Some(parent) = db_path.parent() {
        fs::create_dir_all(parent)?;
    }
    let config = load()?;
    let calendar_theme = CalendarTheme::from_config(&config.theme.calendar);
    let editor_theme = EditorTheme::from_config(&config.theme.editor);
    let keymap = Keymap::from_config(&config.keys);
    let storage = SqliteStorage::new(&db_path).map_err(|e| anyhow!(e))?;
    let mut app = App::new(storage, calendar_theme, editor_theme, keymap)?;

    loop {
        terminal.draw(|frame| ui::draw(frame, &app))?;

        if event::poll(Duration::from_millis(250))? {
            if let Event::Key(key) = event::read()? {
                app.on_key(key)?;
            }
        }

        app.on_tick();

        if app.should_quit {
            break;
        }
    }

    Ok(())
}
