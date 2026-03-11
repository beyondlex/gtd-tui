use ratatui::layout::Rect;
use ratatui::style::{Color, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph};
use ratatui::Frame;

use crate::app::{App, Focus, Layer, Mode};

pub fn render(frame: &mut Frame, area: Rect, app: &App) {
    let mut lines: Vec<Line> = Vec::new();

    // Check if we should show editor before the first task (insert_at_beginning)
    if let Some(editor) = &app.editor {
        if editor.insert_at_beginning && editor.task_id.is_none() {
            lines.push(Line::from("  ┌──────────────────────────────────────┐"));
            lines.extend(editor_lines(app, editor));
            lines.push(Line::from("  └──────────────────────────────────────┘"));
        }
    }

    for (index, task) in app.tasks.iter().enumerate() {
        let selected = index == app.selected && app.mode == Mode::Normal;
        let status = match task.status {
            gtd_core::models::TaskStatus::Completed => "[x]",
            _ => "[ ]",
        };
        let due = task
            .due_date
            .map(|d| format!(" {}", d.format("%m-%d")))
            .unwrap_or_default();
        let prefix = if selected { ">" } else { " " };

        if selected {
            lines.push(Line::from(vec![
                Span::raw(prefix),
                Span::raw(" "),
                Span::raw(status),
                Span::raw(" "),
                Span::styled(&task.title, app.editor_theme.task_selected),
                Span::styled(due, app.editor_theme.date_label),
            ]));
        } else {
            lines.push(Line::from(vec![
                Span::raw(prefix),
                Span::raw(" "),
                Span::raw(status),
                Span::raw(" "),
                Span::raw(&task.title),
                Span::styled(due, app.editor_theme.date_label),
            ]));
        }

        if let Some(editor) = &app.editor {
            let should_show_editor = if editor.insert_at_beginning {
                false
            } else if editor.insert_after == 0 {
                index == 0
            } else {
                editor.insert_after == index
            };
            if should_show_editor {
                // Add separator for new task
                if editor.task_id.is_none() {
                    lines.push(Line::from("  ┌──────────────────────────────────────┐"));
                }
                lines.extend(editor_lines(app, editor));
                // Add bottom separator for new task
                if editor.task_id.is_none() {
                    lines.push(Line::from("  └──────────────────────────────────────┘"));
                }
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

fn editor_lines<'a>(app: &'a App, editor: &'a crate::app::EditorState) -> Vec<Line<'a>> {
    let mut out: Vec<Line<'a>> = Vec::new();
    let title_prefix = focus_prefix(editor.focus == Focus::Title);
    let notes_prefix = focus_prefix(editor.focus == Focus::Notes);
    let due_prefix = focus_prefix(editor.focus == Focus::DueDate);
    let checklist_prefix = focus_prefix(editor.focus == Focus::Checklist);

    let due_label = if editor.focus == Focus::DueDate && editor.edit_active {
        editor.date_picker.cursor.format("%m-%d").to_string()
    } else {
        editor
            .due_date
            .map(|d| d.format("%m-%d").to_string())
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

    // Check if this is a new task and title is empty
    let is_new_task = editor.task_id.is_none();
    let display_title = if is_new_task && editor.title.trim().is_empty() {
        "new task".to_string()
    } else {
        editor.title.clone()
    };

    // Use different style for new task placeholder
    let title_style = if is_new_task && editor.title.trim().is_empty() {
        // Light gray color for placeholder
        Style::default().fg(Color::DarkGray)
    } else if editor.focus == Focus::Title && editor.edit_active {
        app.editor_theme.checklist_edit
    } else {
        Style::default()
    };

    let title_field_style = if editor.focus == Focus::Title {
        app.editor_theme.field_title
    } else {
        Style::default()
    };

    let title_line = if editor.focus == Focus::Title && editor.edit_active {
        Line::from(vec![
            Span::styled(format!("  {title_prefix} Title: "), title_field_style),
            Span::styled(format!("{}{}", display_title, title_cursor), title_style),
        ])
    } else {
        Line::from(vec![
            Span::styled(format!("  {title_prefix} Title: "), title_field_style),
            Span::styled(format!("{}{}", display_title, title_cursor), title_style),
        ])
    };

    // Use different style for notes in new task placeholder
    let notes_style = if is_new_task && editor.title.trim().is_empty() {
        Style::default().fg(Color::DarkGray)
    } else if editor.focus == Focus::Notes && editor.edit_active {
        app.editor_theme.checklist_edit
    } else {
        Style::default()
    };

    let notes_field_style = if editor.focus == Focus::Notes {
        app.editor_theme.field_notes
    } else {
        Style::default()
    };

    let notes_line = if editor.focus == Focus::Notes && editor.edit_active {
        Line::from(vec![
            Span::styled(format!("  {notes_prefix} Notes: "), notes_field_style),
            Span::styled(format!("{}{}", editor.notes, notes_cursor), notes_style),
        ])
    } else {
        Line::from(vec![
            Span::styled(format!("  {notes_prefix} Notes: "), notes_field_style),
            Span::styled(format!("{}{}", editor.notes, notes_cursor), notes_style),
        ])
    };

    let due_field_style = if editor.focus == Focus::DueDate {
        app.editor_theme.field_due
    } else {
        Style::default()
    };

    out.push(title_line);
    out.push(notes_line);
    out.push(Line::from(vec![
        Span::styled(format!("  {due_prefix} Due: "), due_field_style),
        Span::styled(
            format!("{} ", due_label),
            Style::default().fg(Color::DarkGray),
        ),
        Span::styled(
            "(t=Today, m=Tomorrow)".to_string(),
            Style::default().fg(Color::DarkGray),
        ),
    ]));
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

    let checklist_field_style = if editor.focus == Focus::Checklist {
        app.editor_theme.field_checklist
    } else {
        Style::default()
    };

    let checklist_header = if editor.focus == Focus::Checklist && editor.edit_active {
        Line::from(vec![
            Span::styled(
                format!("  {checklist_prefix} Checklist: "),
                checklist_field_style,
            ),
            Span::styled("(editing)", app.editor_theme.checklist_edit),
        ])
    } else if editor.focus == Focus::Checklist && editor.layer == Layer::ChecklistItem {
        Line::from(vec![
            Span::styled(
                format!("  {checklist_prefix} Checklist: "),
                checklist_field_style,
            ),
            Span::styled(count_str, Style::default()),
        ])
    } else {
        Line::from(vec![
            Span::styled(
                format!("  {checklist_prefix} Checklist: "),
                checklist_field_style,
            ),
            Span::styled(count_str, Style::default()),
        ])
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
            } else if selected {
                out.push(Line::from(vec![
                    Span::raw("    "),
                    Span::raw(marker),
                    Span::raw(" "),
                    Span::raw(checkbox),
                    Span::raw(" "),
                    Span::styled(&item.title, app.editor_theme.checklist_item_selected),
                    Span::raw(cursor),
                ]));
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
