use ratatui::layout::{Constraint, Direction, Layout, Rect};

pub struct MainChunks {
    pub sidebar: Rect,
    pub content: Rect,
    pub footer: Rect,
}

pub fn main_chunks(area: Rect) -> MainChunks {
    let vertical = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(3), Constraint::Length(3)])
        .split(area);

    let horizontal = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Length(20), Constraint::Min(10)])
        .split(vertical[0]);

    MainChunks {
        sidebar: horizontal[0],
        content: horizontal[1],
        footer: vertical[1],
    }
}
