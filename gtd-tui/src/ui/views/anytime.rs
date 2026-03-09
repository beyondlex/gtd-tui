use ratatui::layout::Alignment;
use ratatui::widgets::{Block, Borders, Paragraph};
use ratatui::Frame;

pub fn render(frame: &mut Frame, area: ratatui::layout::Rect) {
    let widget = Paragraph::new("Anytime view (area/project hierarchy)")
        .alignment(Alignment::Center)
        .block(Block::default().title("Anytime").borders(Borders::ALL));
    frame.render_widget(widget, area);
}
