use std::io;
use std::time::Duration;

use anyhow::Result;
use crossterm::event::{self, Event, KeyCode};
use crossterm::terminal::{disable_raw_mode, enable_raw_mode};
use ratatui::backend::CrosstermBackend;
use ratatui::layout::Alignment;
use ratatui::widgets::{Block, Borders, Paragraph};
use ratatui::Terminal;

struct App {
    should_quit: bool,
}

impl App {
    fn new() -> Self {
        Self { should_quit: false }
    }

    fn on_key(&mut self, code: KeyCode) {
        match code {
            KeyCode::Char('q') | KeyCode::Esc => self.should_quit = true,
            _ => {}
        }
    }
}

fn main() -> Result<()> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    let backend = CrosstermBackend::new(&mut stdout);
    let mut terminal = Terminal::new(backend)?;

    let mut app = App::new();

    loop {
        terminal.draw(|frame| {
            let area = frame.size();
            let block = Block::default().title("gtd-tui").borders(Borders::ALL);
            let body = Paragraph::new("Press q or Esc to quit")
                .block(block)
                .alignment(Alignment::Center);
            frame.render_widget(body, area);
        })?;

        if event::poll(Duration::from_millis(250))? {
            if let Event::Key(key) = event::read()? {
                app.on_key(key.code);
            }
        }

        if app.should_quit {
            break;
        }
    }

    disable_raw_mode()?;
    Ok(())
}
