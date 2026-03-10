use ratatui::layout::Alignment;
use ratatui::style::{Color, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph};
use ratatui::Frame;

use crate::app::{App, Mode};

pub fn render(frame: &mut Frame, area: ratatui::layout::Rect, app: &App) {
    let mut lines: Vec<Line> = Vec::new();

    for (index, task) in app.tasks.iter().enumerate() {
        let selected = index == app.selected && app.mode == Mode::Normal;
        let status = match task.status {
            gtd_core::models::TaskStatus::Completed => "[x]",
            _ => "[ ]",
        };
        let due = task
            .due_date
            .map(|d| format!(" ({})", d.format("%Y-%m-%d")))
            .unwrap_or_default();
        let prefix = if selected { ">" } else { " " };

        if selected {
            lines.push(Line::from(vec![
                Span::raw(prefix),
                Span::raw(" "),
                Span::raw(status),
                Span::raw(" "),
                Span::styled(&task.title, app.editor_theme.task_selected),
                Span::raw(due),
            ]));
        } else {
            lines.push(Line::from(format!("{prefix} {status} {}{due}", task.title)));
        }
    }

    if app.tasks.is_empty() {
        lines.push(Line::from("No upcoming tasks."));
    }

    let widget =
        Paragraph::new(lines).block(Block::default().title("Upcoming").borders(Borders::ALL));
    frame.render_widget(widget, area);
}
