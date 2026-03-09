use ratatui::style::{Color, Modifier, Style};

use crate::config::CalendarThemeConfig;

#[derive(Debug, Clone, Copy)]
pub struct CalendarTheme {
    pub weekday: Style,
    pub weekend: Style,
    pub today: Style,
    pub selected: Style,
    pub bracket: Style,
}

impl CalendarTheme {
    pub fn from_config(config: &CalendarThemeConfig) -> Self {
        let weekday = parse_style(config.weekday.as_deref())
            .unwrap_or_else(|| Style::default().add_modifier(Modifier::BOLD));
        let weekend = parse_style(config.weekend.as_deref())
            .unwrap_or_else(|| Style::default().fg(Color::Red).add_modifier(Modifier::BOLD));
        let today = parse_style(config.today.as_deref())
            .unwrap_or_else(|| Style::default().fg(Color::Green).add_modifier(Modifier::BOLD));
        let selected = parse_style(config.selected.as_deref())
            .unwrap_or_else(|| Style::default().fg(Color::Blue).add_modifier(Modifier::BOLD));
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
            weekend: Style::default()
                .fg(Color::Red)
                .add_modifier(Modifier::BOLD),
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
