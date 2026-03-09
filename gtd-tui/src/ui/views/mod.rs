mod anytime;
mod inbox;
mod someday;
mod today;
mod upcoming;

use ratatui::layout::Rect;
use ratatui::Frame;

use crate::app::View;

pub fn render(frame: &mut Frame, area: Rect, view: View) {
    match view {
        View::Inbox => inbox::render(frame, area),
        View::Today => today::render(frame, area),
        View::Upcoming => upcoming::render(frame, area),
        View::Anytime => anytime::render(frame, area),
        View::Someday => someday::render(frame, area),
    }
}
