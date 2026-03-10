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
        View::Today => today::render(frame, area, app),
        View::Upcoming => upcoming::render(frame, area, app),
        View::Anytime => anytime::render(frame, area, app),
        View::Someday => someday::render(frame, area, app),
    }
}
