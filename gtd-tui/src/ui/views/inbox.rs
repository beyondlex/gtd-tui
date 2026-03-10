use ratatui::layout::Rect;
use ratatui::style::Style;
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph};
use ratatui::Frame;

use crate::app::{App, Focus, Layer, Mode};

pub fn render(frame: &mut Frame, area: Rect, app: &App) {
    let mut lines: Vec<Line<'static>> = Vec::new();

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
        lines.push(Line::from(format!("{prefix} {status} {}{due}", task.title)));

        if let Some(editor) = &app.editor {
            if editor.insert_after == index {
                lines.extend(editor_lines(app, editor));
            }
        }
    }

    if app.tasks.is_empty() {
        if let Some(editor) = &app.editor {
            lines.extend(editor_lines(app, editor));
        } else {
            lines.push(Line::from("No tasks. Press n to add one."));
        }
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
            .title(format!("Inbox{}", title_suffix))
            .borders(Borders::ALL),
    );
    frame.render_widget(widget, area);
}

fn editor_lines(app: &App, editor: &crate::app::EditorState) -> Vec<Line<'static>> {
    let mut out: Vec<Line<'static>> = Vec::new();
    let title_prefix = focus_prefix(editor.focus == Focus::Title);
    let notes_prefix = focus_prefix(editor.focus == Focus::Notes);
    let due_prefix = focus_prefix(editor.focus == Focus::DueDate);
    let checklist_prefix = focus_prefix(editor.focus == Focus::Checklist);

    let due_label = if editor.focus == Focus::DueDate && editor.edit_active {
        editor.date_picker.cursor.format("%Y-%m-%d").to_string()
    } else {
        editor
            .due_date
            .map(|d| d.format("%Y-%m-%d").to_string())
            .unwrap_or_else(|| "(none)".to_string())
    };

    let title_cursor = if app.cursor_visible && editor.focus == Focus::Title && editor.edit_active {
        "|"
    } else {
        ""
    };
    let notes_cursor = if app.cursor_visible && editor.focus == Focus::Notes && editor.edit_active {
        "|"
    } else {
        ""
    };

    let title_line = if editor.focus == Focus::Title && editor.edit_active {
        Line::from(vec![
            Span::raw(format!("  {title_prefix} Title: ")),
            Span::styled(
                format!("{}{}", editor.title, title_cursor),
                app.editor_theme.checklist_edit,
            ),
        ])
    } else {
        Line::from(format!(
            "  {title_prefix} Title: {}{title_cursor}",
            editor.title
        ))
    };
    let notes_line = if editor.focus == Focus::Notes && editor.edit_active {
        Line::from(vec![
            Span::raw(format!("  {notes_prefix} Notes: ")),
            Span::styled(
                format!("{}{}", editor.notes, notes_cursor),
                app.editor_theme.checklist_edit,
            ),
        ])
    } else {
        Line::from(format!(
            "  {notes_prefix} Notes: {}{notes_cursor}",
            editor.notes
        ))
    };
    out.push(title_line);
    out.push(notes_line);
    out.push(Line::from(format!(
        "  {due_prefix} Due: {}  (t=Today, m=Tomorrow)",
        due_label
    )));
    if editor.focus == Focus::DueDate && editor.edit_active {
        out.push(Line::from(""));
        out.extend(calendar_lines(
            editor,
            app.calendar_theme,
            app.cursor_visible,
        ));
    }

    let checked_count = editor.checklist.iter().filter(|item| item.checked).count();
    let total_count = editor.checklist.len();
    let count_str = if total_count > 0 {
        format!("({}/{})", checked_count, total_count)
    } else {
        String::new()
    };

    let checklist_header = if editor.focus == Focus::Checklist && editor.edit_active {
        Line::from(vec![
            Span::raw(format!("  {checklist_prefix} Checklist: ")),
            Span::styled("(editing)", app.editor_theme.checklist_edit),
        ])
    } else if editor.focus == Focus::Checklist && editor.layer == Layer::ChecklistItem {
        Line::from(format!("  {checklist_prefix} Checklist: {}", count_str))
    } else {
        Line::from(format!("  {checklist_prefix} Checklist: {}", count_str))
    };
    out.push(checklist_header);

    if editor.layer == Layer::TaskItem {
        return out;
    }

    if editor.checklist.is_empty() {
        out.push(Line::from("    - [ ]"));
    } else {
        for (idx, item) in editor.checklist.iter().enumerate() {
            let selected = editor.focus == Focus::Checklist && idx == editor.checklist_index;
            let marker = if selected { ">" } else { " " };
            let cursor = if selected && app.cursor_visible && editor.edit_active {
                "|"
            } else {
                ""
            };
            let checkbox = if item.checked { "[x]" } else { "[ ]" };
            if selected && editor.edit_active {
                out.push(Line::from(Span::styled(
                    format!("    {marker} {checkbox} {}{cursor}", item.title),
                    app.editor_theme.checklist_edit,
                )));
            } else {
                out.push(Line::from(format!(
                    "    {marker} {checkbox} {}{cursor}",
                    item.title
                )));
            }
        }
    }

    out
}

fn focus_prefix(active: bool) -> &'static str {
    if active {
        ">"
    } else {
        " "
    }
}

fn calendar_lines(
    editor: &crate::app::EditorState,
    theme: crate::ui::theme::CalendarTheme,
    cursor_visible: bool,
) -> Vec<Line<'static>> {
    use chrono::{Datelike, NaiveDate};

    let cursor = editor.date_picker.cursor;
    let today = chrono::Utc::now().date_naive();
    let year = cursor.year();
    let month = cursor.month();
    let first = NaiveDate::from_ymd_opt(year, month, 1)
        .unwrap_or_else(|| NaiveDate::from_ymd_opt(year, month, 1).unwrap());
    let weekday = first.weekday().num_days_from_monday();
    let days_in_month = crate::app::days_in_month(year, month);

    let mut lines: Vec<Line<'static>> = Vec::new();
    lines.push(Line::from(format!("    {} {}", cursor.format("%B"), year)));

    let header = [
        ("Mo", false),
        ("Tu", false),
        ("We", false),
        ("Th", false),
        ("Fr", false),
        ("Sa", true),
        ("Su", true),
    ];
    let mut spans = vec![Span::raw("    ")];
    for (label, weekend) in header {
        let style = if weekend {
            theme.weekend
        } else {
            theme.weekday
        };
        spans.push(Span::styled(format!(" {label} "), style));
    }
    lines.push(Line::from(spans));

    let mut weeks: Vec<[Option<NaiveDate>; 7]> = Vec::new();
    let mut week = [None, None, None, None, None, None, None];
    let mut pos = weekday as usize;

    for day in 1..=days_in_month {
        let date = NaiveDate::from_ymd_opt(year, month, day)
            .unwrap_or_else(|| NaiveDate::from_ymd_opt(year, month, day).unwrap());
        week[pos] = Some(date);
        pos += 1;
        if pos == 7 {
            weeks.push(week);
            week = [None, None, None, None, None, None, None];
            pos = 0;
        }
    }

    if week.iter().any(|d| d.is_some()) {
        weeks.push(week);
    }

    for week in weeks.iter() {
        let mut spans = vec![Span::raw("    ")];
        for day in week.iter() {
            if let Some(date) = day {
                let is_cursor = *date == cursor;
                let is_today = *date == today;
                let is_weekend = date.weekday().number_from_monday() >= 6;
                let style = if is_cursor {
                    theme.selected
                } else if is_today {
                    theme.today
                } else if is_weekend {
                    theme.weekend
                } else {
                    theme.weekday
                };
                if is_cursor {
                    if cursor_visible {
                        spans.push(Span::styled("[", theme.bracket));
                        spans.push(Span::styled(format!("{:02}", date.day()), style));
                        spans.push(Span::styled("]", theme.bracket));
                    } else {
                        spans.push(Span::styled(format!(" {:02} ", date.day()), style));
                    }
                } else {
                    spans.push(Span::styled(format!(" {:02} ", date.day()), style));
                }
            } else {
                spans.push(Span::raw("    "));
            }
        }
        lines.push(Line::from(spans));
    }

    lines
}
