use ratatui::layout::Alignment;
use ratatui::widgets::{Block, Borders, Paragraph};
use ratatui::Frame;

pub fn render(frame: &mut Frame, area: ratatui::layout::Rect) {
    let widget = Paragraph::new("Inbox view (tasks without project/area)")
        .alignment(Alignment::Center)
        .block(Block::default().title("Inbox").borders(Borders::ALL));
    frame.render_widget(widget, area);
}
