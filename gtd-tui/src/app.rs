use crossterm::event::KeyCode;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum View {
    Inbox,
    Today,
    Upcoming,
    Anytime,
    Someday,
}

impl View {
    pub fn title(self) -> &'static str {
        match self {
            View::Inbox => "Inbox",
            View::Today => "Today",
            View::Upcoming => "Upcoming",
            View::Anytime => "Anytime",
            View::Someday => "Someday",
        }
    }

    pub fn all() -> [View; 5] {
        [
            View::Inbox,
            View::Today,
            View::Upcoming,
            View::Anytime,
            View::Someday,
        ]
    }
}

pub struct App {
    pub should_quit: bool,
    pub view: View,
}

impl App {
    pub fn new() -> Self {
        Self {
            should_quit: false,
            view: View::Inbox,
        }
    }

    pub fn on_key(&mut self, code: KeyCode) {
        match code {
            KeyCode::Char('q') | KeyCode::Esc => self.should_quit = true,
            KeyCode::Char('1') | KeyCode::Char('i') => self.view = View::Inbox,
            KeyCode::Char('2') | KeyCode::Char('t') => self.view = View::Today,
            KeyCode::Char('3') | KeyCode::Char('u') => self.view = View::Upcoming,
            KeyCode::Char('4') | KeyCode::Char('a') => self.view = View::Anytime,
            KeyCode::Char('5') | KeyCode::Char('s') => self.view = View::Someday,
            _ => {}
        }
    }
}
