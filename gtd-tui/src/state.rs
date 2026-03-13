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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Mode {
    Normal,
    Editing,
    ConfirmDelete,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DeleteTarget {
    Task,
    ChecklistItem,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Layer {
    TaskItem,
    ChecklistItem,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Focus {
    Title,
    Notes,
    DueDate,
    Checklist,
}

impl Focus {
    pub fn next(self) -> Self {
        match self {
            Focus::Title => Focus::Notes,
            Focus::Notes => Focus::DueDate,
            Focus::DueDate => Focus::Checklist,
            Focus::Checklist => Focus::Title,
        }
    }

    pub fn prev(self) -> Self {
        match self {
            Focus::Title => Focus::Checklist,
            Focus::Notes => Focus::Title,
            Focus::DueDate => Focus::Notes,
            Focus::Checklist => Focus::DueDate,
        }
    }
}
