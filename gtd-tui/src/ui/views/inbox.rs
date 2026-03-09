use ratatui::layout::Rect;
use ratatui::widgets::{Block, Borders, Paragraph};
use ratatui::Frame;

use crate::app::{App, Focus, Mode};

pub fn render(frame: &mut Frame, area: Rect, app: &App) {
    let mut lines = Vec::new();

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
        lines.push(format!("{prefix} {status} {}{due}", task.title));

        if let Some(editor) = &app.editor {
            if editor.insert_after == index {
                lines.extend(editor_lines(editor));
            }
        }
    }

    if app.tasks.is_empty() {
        if let Some(editor) = &app.editor {
            lines.extend(editor_lines(editor));
        } else {
            lines.push("No tasks. Press n to add one.".to_string());
        }
    }

    let widget = Paragraph::new(lines.join("\n"))
        .block(Block::default().title("Inbox").borders(Borders::ALL));
    frame.render_widget(widget, area);
}

fn editor_lines(editor: &crate::app::EditorState) -> Vec<String> {
    let mut out = Vec::new();
    let title_prefix = focus_prefix(editor.focus == Focus::Title);
    let notes_prefix = focus_prefix(editor.focus == Focus::Notes);
    let due_prefix = focus_prefix(editor.focus == Focus::DueDate);
    let checklist_prefix = focus_prefix(editor.focus == Focus::Checklist);

    let due_label = editor
        .due_date
        .map(|d| d.format("%Y-%m-%d").to_string())
        .unwrap_or_else(|| "(none)".to_string());

    out.push(format!("  {title_prefix} Title: {}", editor.title));
    out.push(format!("  {notes_prefix} Notes: {}", editor.notes));
    out.push(format!(
        "  {due_prefix} Due: {}  (t=Today, m=Tomorrow)",
        due_label
    ));
    out.push(format!("  {checklist_prefix} Checklist:"));

    if editor.checklist.is_empty() {
        out.push("    - [ ]".to_string());
    } else {
        for (idx, item) in editor.checklist.iter().enumerate() {
            let marker = if editor.focus == Focus::Checklist && idx == editor.checklist_index {
                ">"
            } else {
                " "
            };
            out.push(format!("    {marker} [ ] {item}"));
        }
    }

    if editor.focus == Focus::DueDate {
        out.push("".to_string());
        out.extend(calendar_lines(editor));
    }

    out
}

fn focus_prefix(active: bool) -> &'static str {
    if active { ">" } else { " " }
}

fn calendar_lines(editor: &crate::app::EditorState) -> Vec<String> {
    use chrono::{Datelike, NaiveDate};

    let cursor = editor.date_picker.cursor;
    let year = cursor.year();
    let month = cursor.month();
    let first = NaiveDate::from_ymd_opt(year, month, 1)
        .unwrap_or_else(|| NaiveDate::from_ymd_opt(year, month, 1).unwrap());
    let weekday = first.weekday().num_days_from_monday();
    let days_in_month = crate::app::days_in_month(year, month);

    let mut lines = Vec::new();
    lines.push(format!("    {} {}", cursor.format("%B"), year));
    lines.push("    Mo Tu We Th Fr Sa Su".to_string());

    let mut line = String::from("    ");
    let mut pos = weekday as u32;
    for _ in 0..weekday {
        line.push_str("   ");
    }

    for day in 1..=days_in_month {
        let date = NaiveDate::from_ymd_opt(year, month, day)
            .unwrap_or_else(|| NaiveDate::from_ymd_opt(year, month, day).unwrap());
        let is_cursor = date == cursor;
        if is_cursor {
            line.push_str(&format!("[{:02}]", day));
        } else {
            line.push_str(&format!(" {:02} ", day));
        }
        pos += 1;
        if pos == 7 {
            lines.push(line.trim_end().to_string());
            line = String::from("    ");
            pos = 0;
        }
    }

    if line.trim().len() > 4 {
        lines.push(line.trim_end().to_string());
    }

    lines
}
