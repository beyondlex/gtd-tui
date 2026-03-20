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
        let is_completed = matches!(task.status, gtd_core::models::TaskStatus::Completed);
        let status = if is_completed { "[x]" } else { "[ ]" };
        let due = task
            .due_date
            .map(|d| format!(" ({})", d.format("%Y-%m-%d")))
            .unwrap_or_default();
        let prefix = if selected { ">" } else { " " };

        if selected {
            let title_style = if is_completed {
                app.editor_theme.completed
            } else {
                app.editor_theme.task_selected
            };
            let status_style = if is_completed {
                app.editor_theme.completed
            } else {
                Style::default()
            };
            let due_style = if is_completed {
                app.editor_theme.completed
            } else {
                Style::default()
            };
            lines.push(Line::from(vec![
                Span::raw(prefix),
                Span::raw(" "),
                Span::styled(status, status_style),
                Span::raw(" "),
                Span::styled(&task.title, title_style),
                Span::styled(due, due_style),
            ]));
        } else if is_completed {
            lines.push(Line::from(vec![
                Span::raw(prefix),
                Span::raw(" "),
                Span::styled(status, app.editor_theme.completed),
                Span::raw(" "),
                Span::styled(&task.title, app.editor_theme.completed),
                Span::styled(due, app.editor_theme.completed),
            ]));
        } else {
            lines.push(Line::from(format!("{prefix} {status} {}{due}", task.title)));
        }
    }

    if app.tasks.is_empty() {
        lines.push(Line::from("No tasks for today."));
    }

    let completed_count = app
        .tasks
        .iter()
        .filter(|t| matches!(t.status, gtd_core::models::TaskStatus::Completed))
        .count();
    let total_count = app.tasks.len();
    let title_suffix = if total_count > 0 {
        format!(" ({}/{})", completed_count, total_count)
    } else {
        String::new()
    };

    let widget = Paragraph::new(lines).block(
        Block::default()
            .title(format!("Today{}", title_suffix))
            .borders(Borders::ALL),
    );
    frame.render_widget(widget, area);
}
