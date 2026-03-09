mod layout;
pub mod theme;
mod views;

use ratatui::layout::Alignment;
use ratatui::style::{Modifier, Style};
use ratatui::widgets::{Block, Borders, List, ListItem, Paragraph};
use ratatui::Frame;

use crate::app::{App, Mode, View};

pub fn draw(frame: &mut Frame, app: &App) {
    let chunks = layout::main_chunks(frame.size());

    let views = View::all();
    let items = views.iter().map(|view| {
        let label = format!("{}", view.title());
        ListItem::new(label)
    });

    let list = List::new(items)
        .block(Block::default().title("Views").borders(Borders::ALL))
        .highlight_style(Style::default().add_modifier(Modifier::BOLD))
        .highlight_symbol("> ");

    let mut state = list_state(app.view);
    frame.render_stateful_widget(list, chunks.sidebar, &mut state);

    let hint = match app.mode {
        Mode::Normal => Paragraph::new(
            "Keys: 1-5 or i/t/u/a/s switch views, j/k move, n new, l edit, x toggle, r refresh, q quit",
        ),
        Mode::Editing => Paragraph::new(
            "Edit: j/k move, l edit checklist item, Ctrl+S save, Esc cancel",
        ),
    }
        .alignment(Alignment::Center)
        .block(Block::default().title("Help").borders(Borders::ALL));
    frame.render_widget(hint, chunks.footer);

    views::render(frame, chunks.content, app);
}

fn list_state(view: View) -> ratatui::widgets::ListState {
    let mut state = ratatui::widgets::ListState::default();
    let index = match view {
        View::Inbox => 0,
        View::Today => 1,
        View::Upcoming => 2,
        View::Anytime => 3,
        View::Someday => 4,
    };
    state.select(Some(index));
    state
}
