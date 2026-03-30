use chrono::{Datelike, NaiveDate};
use uuid::Uuid;

use super::state::{Focus, Layer};

pub fn days_in_month(year: i32, month: u32) -> u32 {
    let next_month = if month == 12 { 1 } else { month + 1 };
    let next_year = if month == 12 { year + 1 } else { year };
    let first_next = NaiveDate::from_ymd_opt(next_year, next_month, 1)
        .unwrap_or_else(|| NaiveDate::from_ymd_opt(year, month, 28).unwrap());
    let last = first_next - chrono::Duration::days(1);
    last.day()
}

#[derive(Debug, Clone)]
pub struct EditorState {
    pub task_id: Option<Uuid>,
    pub insert_after: usize,
    pub insert_at_beginning: bool,
    pub title: String,
    pub notes: String,
    pub due_date: Option<NaiveDate>,
    pub checklist: Vec<ChecklistDraft>,
    pub checklist_index: usize,
    pub edit_active: bool,
    pub focus: Focus,
    pub layer: Layer,
    pub date_picker: DatePickerState,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ChecklistDraft {
    pub title: String,
    pub checked: bool,
}

impl ChecklistDraft {
    pub fn new() -> Self {
        Self {
            title: String::new(),
            checked: false,
        }
    }
}

impl Default for ChecklistDraft {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, Copy)]
pub struct DatePickerState {
    pub cursor: NaiveDate,
}

impl DatePickerState {
    pub fn new(seed: NaiveDate) -> Self {
        Self { cursor: seed }
    }

    pub fn move_days(&mut self, delta: i64) {
        if let Some(next) = self
            .cursor
            .checked_add_signed(chrono::Duration::days(delta))
        {
            self.cursor = next;
        }
    }

    pub fn move_months(&mut self, delta: i32) {
        let mut year = self.cursor.year();
        let mut month = self.cursor.month() as i32 + delta;
        while month > 12 {
            month -= 12;
            year += 1;
        }
        while month < 1 {
            month += 12;
            year -= 1;
        }
        let day = self.cursor.day().min(days_in_month(year, month as u32));
        if let Some(next) = NaiveDate::from_ymd_opt(year, month as u32, day) {
            self.cursor = next;
        }
    }
}

pub fn ensure_checklist_not_empty(checklist: &mut Vec<ChecklistDraft>) {
    if checklist.is_empty() {
        checklist.push(ChecklistDraft::new());
    }
}

pub fn new_checklist_item(
    checklist: &mut Vec<ChecklistDraft>,
    checklist_index: &mut usize,
    insert_at_beginning: bool,
) {
    if insert_at_beginning {
        checklist.insert(0, ChecklistDraft::new());
        *checklist_index = 0;
    } else {
        checklist.push(ChecklistDraft::new());
        *checklist_index = checklist.len() - 1;
    }
}
