use ratatui::style::{Modifier, Style};
use ratatui::widgets::{Block, Borders, List, ListItem, ListState};
use ratatui::Frame;

use crate::app::App;

pub fn render(frame: &mut Frame, area: ratatui::layout::Rect, app: &App) {
    let items = app.tasks.iter().map(|task| {
        let status = match task.status {
            gtd_core::models::TaskStatus::Completed => "[x]",
            _ => "[ ]",
        };
        ListItem::new(format!("{status} {}", task.title))
    });

    let list = List::new(items)
        .block(Block::default().title("Inbox").borders(Borders::ALL))
        .highlight_style(Style::default().add_modifier(Modifier::BOLD))
        .highlight_symbol("> ");

    let mut state = ListState::default();
    if !app.tasks.is_empty() {
        state.select(Some(app.selected.min(app.tasks.len() - 1)));
    }

    frame.render_stateful_widget(list, area, &mut state);
}
