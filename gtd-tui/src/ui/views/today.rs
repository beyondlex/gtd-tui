use ratatui::Frame;
use ratatui::layout::Alignment;
use ratatui::widgets::{Block, Borders, Paragraph};

pub fn render(frame: &mut Frame, area: ratatui::layout::Rect) {
    let widget = Paragraph::new("Today view (tasks planned for today)")
        .alignment(Alignment::Center)
        .block(Block::default().title("Today").borders(Borders::ALL));
    frame.render_widget(widget, area);
}
