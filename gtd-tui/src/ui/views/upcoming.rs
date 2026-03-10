use ratatui::Frame;
use ratatui::layout::Alignment;
use ratatui::widgets::{Block, Borders, Paragraph};

pub fn render(frame: &mut Frame, area: ratatui::layout::Rect) {
    let widget = Paragraph::new("Upcoming view (grouped by date)")
        .alignment(Alignment::Center)
        .block(Block::default().title("Upcoming").borders(Borders::ALL));
    frame.render_widget(widget, area);
}
