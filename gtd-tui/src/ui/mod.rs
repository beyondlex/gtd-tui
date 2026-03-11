mod layout;
pub mod theme;
mod views;

use ratatui::layout::Alignment;
use ratatui::style::{Modifier, Style};
use ratatui::widgets::{Block, Borders, List, ListItem, Paragraph};
use ratatui::Frame;

use crate::app::{App, DeleteTarget, Mode, View};

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
            "Keys: 1-5 or i/t/u/a/s switch views, j/k move, n new, l edit, x toggle, d delete, r refresh, q quit",
        ),
        Mode::Editing => Paragraph::new(
            "Edit: j/k move, l edit checklist item, d delete item, Ctrl+S save, Esc cancel",
        ),
        Mode::ConfirmDelete => Paragraph::new("Delete? y/Enter confirm, any key to cancel"),
    }
        .alignment(Alignment::Center)
        .block(Block::default().title("Help").borders(Borders::ALL));
    frame.render_widget(hint, chunks.footer);

    views::render(frame, chunks.content, app);

    if app.mode == Mode::ConfirmDelete {
        draw_delete_confirm_dialog(frame, app, frame.size());
    }
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

fn draw_delete_confirm_dialog(frame: &mut Frame, app: &App, area: ratatui::layout::Rect) {
    use ratatui::widgets::Clear;

    let msg = match app.delete_confirm {
        Some(DeleteTarget::Task) => "Delete this task?",
        Some(DeleteTarget::ChecklistItem) => "Delete this checklist item?",
        None => "Delete?",
    };

    let dialog_width = 40;
    let dialog_height = 5;
    let x = (area.width.saturating_sub(dialog_width)) / 2;
    let y = (area.height.saturating_sub(dialog_height)) / 2;

    let dialog_area = ratatui::layout::Rect::new(x, y, dialog_width, dialog_height);

    frame.render_widget(Clear, dialog_area);

    let block = Block::default()
        .title(" Confirm Delete ")
        .borders(Borders::ALL)
        .border_style(
            Style::default().fg(app
                .editor_theme
                .task_selected
                .fg
                .unwrap_or(ratatui::style::Color::Red)),
        );

    let text = Paragraph::new(msg)
        .alignment(Alignment::Center)
        .block(block);

    frame.render_widget(text, dialog_area);
}
