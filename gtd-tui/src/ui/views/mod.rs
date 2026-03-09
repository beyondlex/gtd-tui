mod anytime;
mod inbox;
mod someday;
mod today;
mod upcoming;

use ratatui::layout::Rect;
use ratatui::Frame;

use crate::app::{App, View};

pub fn render(frame: &mut Frame, area: Rect, app: &App) {
    match app.view {
        View::Inbox => inbox::render(frame, area, app),
        View::Today => today::render(frame, area),
        View::Upcoming => upcoming::render(frame, area),
        View::Anytime => anytime::render(frame, area),
        View::Someday => someday::render(frame, area),
    }
}
