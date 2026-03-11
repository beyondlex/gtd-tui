use ratatui::style::{Color, Modifier, Style};

use crate::config::{CalendarThemeConfig, EditorThemeConfig};

#[derive(Debug, Clone, Copy)]
pub struct CalendarTheme {
    pub weekday: Style,
    pub weekend: Style,
    pub today: Style,
    pub selected: Style,
    pub bracket: Style,
}

#[derive(Debug, Clone, Copy)]
pub struct EditorTheme {
    pub checklist_edit: Style,
    pub task_selected: Style,
    pub date_label: Style,
    pub checklist_item_selected: Style,
    pub field_title: Style,
    pub field_notes: Style,
    pub field_due: Style,
    pub field_checklist: Style,
}

impl CalendarTheme {
    pub fn from_config(config: &CalendarThemeConfig) -> Self {
        let weekday = parse_style(config.weekday.as_deref())
            .unwrap_or_else(|| Style::default().add_modifier(Modifier::BOLD));
        let weekend = parse_style(config.weekend.as_deref())
            .unwrap_or_else(|| Style::default().fg(Color::Red).add_modifier(Modifier::BOLD));
        let today = parse_style(config.today.as_deref()).unwrap_or_else(|| {
            Style::default()
                .fg(Color::Green)
                .add_modifier(Modifier::BOLD)
        });
        let selected = parse_style(config.selected.as_deref()).unwrap_or_else(|| {
            Style::default()
                .fg(Color::Blue)
                .add_modifier(Modifier::BOLD)
        });
        let bracket = parse_style(config.bracket.as_deref())
            .unwrap_or_else(|| Style::default().fg(Color::Magenta));

        Self {
            weekday,
            weekend,
            today,
            selected,
            bracket,
        }
    }
}

impl Default for CalendarTheme {
    fn default() -> Self {
        Self {
            weekday: Style::default().add_modifier(Modifier::BOLD),
            weekend: Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
            today: Style::default()
                .fg(Color::Green)
                .add_modifier(Modifier::BOLD),
            selected: Style::default()
                .fg(Color::Blue)
                .add_modifier(Modifier::BOLD),
            bracket: Style::default().fg(Color::Magenta),
        }
    }
}

impl EditorTheme {
    pub fn from_config(config: &EditorThemeConfig) -> Self {
        let checklist_edit = parse_style(config.checklist_edit.as_deref()).unwrap_or_else(|| {
            Style::default()
                .fg(Color::LightYellow)
                .add_modifier(Modifier::BOLD)
        });
        let task_selected = parse_style(config.task_selected.as_deref()).unwrap_or_else(|| {
            Style::default()
                .fg(Color::Blue)
                .add_modifier(Modifier::BOLD)
        });
        let date_label = parse_style(config.date_label.as_deref()).unwrap_or_else(|| {
            Style::default()
                .fg(Color::DarkGray)
                .add_modifier(Modifier::BOLD)
        });
        let checklist_item_selected = parse_style(config.checklist_item_selected.as_deref())
            .unwrap_or_else(|| {
                Style::default()
                    .fg(Color::Blue)
                    .add_modifier(Modifier::BOLD)
            });
        let field_title = parse_style(config.field_title.as_deref()).unwrap_or_else(|| {
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD)
        });
        let field_notes = parse_style(config.field_notes.as_deref()).unwrap_or_else(|| {
            Style::default()
                .fg(Color::Green)
                .add_modifier(Modifier::BOLD)
        });
        let field_due = parse_style(config.field_due.as_deref()).unwrap_or_else(|| {
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD)
        });
        let field_checklist = parse_style(config.field_checklist.as_deref()).unwrap_or_else(|| {
            Style::default()
                .fg(Color::Magenta)
                .add_modifier(Modifier::BOLD)
        });
        Self {
            checklist_edit,
            task_selected,
            date_label,
            checklist_item_selected,
            field_title,
            field_notes,
            field_due,
            field_checklist,
        }
    }
}

impl Default for EditorTheme {
    fn default() -> Self {
        Self {
            checklist_edit: Style::default()
                .fg(Color::LightYellow)
                .add_modifier(Modifier::BOLD),
            task_selected: Style::default()
                .fg(Color::Blue)
                .add_modifier(Modifier::BOLD),
            date_label: Style::default()
                .fg(Color::DarkGray)
                .add_modifier(Modifier::BOLD),
            checklist_item_selected: Style::default()
                .fg(Color::Blue)
                .add_modifier(Modifier::BOLD),
            field_title: Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
            field_notes: Style::default()
                .fg(Color::Green)
                .add_modifier(Modifier::BOLD),
            field_due: Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
            field_checklist: Style::default()
                .fg(Color::Magenta)
                .add_modifier(Modifier::BOLD),
        }
    }
}

fn parse_style(value: Option<&str>) -> Option<Style> {
    let value = value?.trim().to_lowercase();
    if value.is_empty() {
        return None;
    }
    let mut style = Style::default();
    for token in value.split_whitespace() {
        if token == "bold" {
            style = style.add_modifier(Modifier::BOLD);
            continue;
        }
        if let Some(color) = parse_color(token) {
            style = style.fg(color);
        }
    }
    Some(style)
}

fn parse_color(token: &str) -> Option<Color> {
    match token {
        "black" => Some(Color::Black),
        "red" => Some(Color::Red),
        "green" => Some(Color::Green),
        "yellow" => Some(Color::Yellow),
        "blue" => Some(Color::Blue),
        "magenta" => Some(Color::Magenta),
        "cyan" => Some(Color::Cyan),
        "gray" | "grey" => Some(Color::Gray),
        "darkgray" | "darkgrey" => Some(Color::DarkGray),
        "lightred" => Some(Color::LightRed),
        "lightgreen" => Some(Color::LightGreen),
        "lightyellow" => Some(Color::LightYellow),
        "lightblue" => Some(Color::LightBlue),
        "lightmagenta" => Some(Color::LightMagenta),
        "lightcyan" => Some(Color::LightCyan),
        "white" => Some(Color::White),
        "reset" => Some(Color::Reset),
        _ => None,
    }
}
