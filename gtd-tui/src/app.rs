use anyhow::{anyhow, Result};
use chrono::{Datelike, NaiveDate, Utc};
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use gtd_core::models::{ChecklistItem, Task, TaskStatus};
use gtd_core::storage::{SqliteStorage, Storage, TaskFilter};
use uuid::Uuid;

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
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Focus {
    Title,
    Notes,
    DueDate,
    Checklist,
}

impl Focus {
    fn next(self) -> Self {
        match self {
            Focus::Title => Focus::Notes,
            Focus::Notes => Focus::DueDate,
            Focus::DueDate => Focus::Checklist,
            Focus::Checklist => Focus::Title,
        }
    }

    fn prev(self) -> Self {
        match self {
            Focus::Title => Focus::Checklist,
            Focus::Notes => Focus::Title,
            Focus::DueDate => Focus::Notes,
            Focus::Checklist => Focus::DueDate,
        }
    }
}

#[derive(Debug, Clone)]
pub struct EditorState {
    pub task_id: Option<Uuid>,
    pub insert_after: usize,
    pub title: String,
    pub notes: String,
    pub due_date: Option<NaiveDate>,
    pub checklist: Vec<String>,
    pub checklist_index: usize,
    pub focus: Focus,
    pub date_picker: DatePickerState,
}

#[derive(Debug, Clone, Copy)]
pub struct DatePickerState {
    pub cursor: NaiveDate,
}

impl DatePickerState {
    fn new(seed: NaiveDate) -> Self {
        Self { cursor: seed }
    }

    fn move_days(&mut self, delta: i64) {
        if let Some(next) = self.cursor.checked_add_signed(chrono::Duration::days(delta)) {
            self.cursor = next;
        }
    }

    fn move_months(&mut self, delta: i32) {
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

pub struct App {
    pub should_quit: bool,
    pub view: View,
    pub mode: Mode,
    pub tasks: Vec<Task>,
    pub selected: usize,
    pub editor: Option<EditorState>,
    storage: SqliteStorage,
}

impl App {
    pub fn new(storage: SqliteStorage) -> Result<Self> {
        let mut app = Self {
            should_quit: false,
            view: View::Inbox,
            mode: Mode::Normal,
            tasks: Vec::new(),
            selected: 0,
            editor: None,
            storage,
        };
        app.refresh_tasks()?;
        Ok(app)
    }

    pub fn on_key(&mut self, key: KeyEvent) -> Result<()> {
        match self.mode {
            Mode::Normal => self.on_key_normal(key),
            Mode::Editing => self.on_key_edit(key),
        }
    }

    fn on_key_normal(&mut self, key: KeyEvent) -> Result<()> {
        match key.code {
            KeyCode::Char('q') | KeyCode::Esc => self.should_quit = true,
            KeyCode::Char('1') | KeyCode::Char('i') => self.view = View::Inbox,
            KeyCode::Char('2') | KeyCode::Char('t') => self.view = View::Today,
            KeyCode::Char('3') | KeyCode::Char('u') => self.view = View::Upcoming,
            KeyCode::Char('4') | KeyCode::Char('a') => self.view = View::Anytime,
            KeyCode::Char('5') | KeyCode::Char('s') => self.view = View::Someday,
            KeyCode::Char('j') | KeyCode::Down => self.select_next(),
            KeyCode::Char('k') | KeyCode::Up => self.select_prev(),
            KeyCode::Char('n') => self.start_new_task(),
            KeyCode::Char('e') => self.start_edit_task()?,
            KeyCode::Char('x') => self.toggle_selected_task()?,
            KeyCode::Char('r') => self.refresh_tasks()?,
            _ => {}
        }
        Ok(())
    }

    fn on_key_edit(&mut self, key: KeyEvent) -> Result<()> {
        if key.modifiers.contains(KeyModifiers::CONTROL)
            && matches!(key.code, KeyCode::Char('s') | KeyCode::Char('S'))
        {
            return self.save_edit();
        }

        let editor = match self.editor.as_mut() {
            Some(editor) => editor,
            None => return Ok(()),
        };

        match key.code {
            KeyCode::Esc => {
                self.cancel_edit();
                return Ok(());
            }
            KeyCode::Tab => editor.focus = editor.focus.next(),
            KeyCode::BackTab => editor.focus = editor.focus.prev(),
            KeyCode::Enter => Self::handle_enter(editor)?,
            KeyCode::Backspace => Self::handle_backspace(editor),
            KeyCode::Char(ch) => Self::handle_char(editor, ch),
            KeyCode::Left if editor.focus == Focus::DueDate => editor.date_picker.move_days(-1),
            KeyCode::Right if editor.focus == Focus::DueDate => editor.date_picker.move_days(1),
            KeyCode::Up if editor.focus == Focus::DueDate => editor.date_picker.move_days(-7),
            KeyCode::Down if editor.focus == Focus::DueDate => editor.date_picker.move_days(7),
            KeyCode::Char('p') if editor.focus == Focus::DueDate => editor.date_picker.move_months(-1),
            KeyCode::Char('n') if editor.focus == Focus::DueDate => editor.date_picker.move_months(1),
            KeyCode::Char('t') if editor.focus == Focus::DueDate => {
                editor.date_picker.cursor = Utc::now().date_naive()
            }
            KeyCode::Char('m') if editor.focus == Focus::DueDate => {
                editor.date_picker.cursor = Utc::now().date_naive() + chrono::Duration::days(1)
            }
            _ => {}
        }
        Ok(())
    }

    fn handle_enter(editor: &mut EditorState) -> Result<()> {
        match editor.focus {
            Focus::Title => editor.focus = Focus::Notes,
            Focus::Notes => editor.focus = Focus::DueDate,
            Focus::DueDate => {
                editor.due_date = Some(editor.date_picker.cursor);
                editor.focus = Focus::Checklist;
            }
            Focus::Checklist => {
                editor
                    .checklist
                    .insert(editor.checklist_index + 1, String::new());
                editor.checklist_index += 1;
            }
        }
        Ok(())
    }

    fn handle_backspace(editor: &mut EditorState) {
        match editor.focus {
            Focus::Title => {
                editor.title.pop();
            }
            Focus::Notes => {
                editor.notes.pop();
            }
            Focus::DueDate => {
                editor.due_date = None;
            }
            Focus::Checklist => {
                if editor.checklist.is_empty() {
                    return;
                }
                let current = &mut editor.checklist[editor.checklist_index];
                if current.is_empty() {
                    editor.checklist.remove(editor.checklist_index);
                    if editor.checklist_index > 0 {
                        editor.checklist_index -= 1;
                    }
                } else {
                    current.pop();
                }
            }
        }
    }

    fn handle_char(editor: &mut EditorState, ch: char) {
        match editor.focus {
            Focus::Title => editor.title.push(ch),
            Focus::Notes => editor.notes.push(ch),
            Focus::DueDate => {}
            Focus::Checklist => {
                if editor.checklist.is_empty() {
                    editor.checklist.push(String::new());
                    editor.checklist_index = 0;
                }
                editor.checklist[editor.checklist_index].push(ch);
            }
        }
    }

    fn start_new_task(&mut self) {
        let insert_after = if self.tasks.is_empty() { 0 } else { self.selected };
        let today = Utc::now().date_naive();
        self.editor = Some(EditorState {
            task_id: None,
            insert_after,
            title: String::new(),
            notes: String::new(),
            due_date: None,
            checklist: vec![String::new()],
            checklist_index: 0,
            focus: Focus::Title,
            date_picker: DatePickerState::new(today),
        });
        self.mode = Mode::Editing;
    }

    fn start_edit_task(&mut self) -> Result<()> {
        if self.tasks.is_empty() {
            return Ok(());
        }
        let task = self.tasks[self.selected].clone();
        let checklist_items = self
            .storage
            .get_checklist(task.id)
            .map_err(|e| anyhow!(e))?;
        let checklist = if checklist_items.is_empty() {
            vec![String::new()]
        } else {
            checklist_items.into_iter().map(|item| item.title).collect()
        };
        let today = Utc::now().date_naive();
        let seed_date = task.due_date.unwrap_or(today);
        self.editor = Some(EditorState {
            task_id: Some(task.id),
            insert_after: self.selected,
            title: task.title,
            notes: task.notes.unwrap_or_default(),
            due_date: task.due_date,
            checklist,
            checklist_index: 0,
            focus: Focus::Title,
            date_picker: DatePickerState::new(seed_date),
        });
        self.mode = Mode::Editing;
        Ok(())
    }

    fn cancel_edit(&mut self) {
        self.editor = None;
        self.mode = Mode::Normal;
    }

    pub fn save_edit(&mut self) -> Result<()> {
        let editor = match self.editor.take() {
            Some(editor) => editor,
            None => return Ok(()),
        };

        let title = editor.title.trim();
        if title.is_empty() {
            self.mode = Mode::Normal;
            return Ok(());
        }

        let now = Utc::now();
        let notes = if editor.notes.trim().is_empty() {
            None
        } else {
            Some(editor.notes)
        };

        let task_id = if let Some(task_id) = editor.task_id {
            let mut task = self.tasks[self.selected].clone();
            task.title = title.to_string();
            task.notes = notes;
            task.due_date = editor.due_date;
            task.updated_at = now;
            self.storage.update_task(&task).map_err(|e| anyhow!(e))?;
            task_id
        } else {
            let task_id = Uuid::new_v4();
            let task = Task {
                id: task_id,
                project_id: None,
                heading_id: None,
                area_id: None,
                title: title.to_string(),
                notes,
                status: TaskStatus::Pending,
                start_date: None,
                due_date: editor.due_date,
                is_today: false,
                is_someday: false,
                sort_order: self.tasks.len() as i32,
                created_at: now,
                updated_at: now,
            };
            self.storage.create_task(&task).map_err(|e| anyhow!(e))?;
            task_id
        };

        self.replace_checklist(task_id, editor.checklist)?;
        self.mode = Mode::Normal;
        self.editor = None;
        self.refresh_tasks()?;
        Ok(())
    }

    fn replace_checklist(&self, task_id: Uuid, checklist: Vec<String>) -> Result<()> {
        let existing = self
            .storage
            .get_checklist(task_id)
            .map_err(|e| anyhow!(e))?;
        for item in existing {
            self.storage
                .delete_checklist_item(item.id)
                .map_err(|e| anyhow!(e))?;
        }
        for (index, title) in checklist.into_iter().enumerate() {
            let title = title.trim();
            if title.is_empty() {
                continue;
            }
            let item = ChecklistItem {
                id: Uuid::new_v4(),
                task_id,
                title: title.to_string(),
                is_checked: false,
                sort_order: index as i32,
            };
            self.storage
                .create_checklist_item(&item)
                .map_err(|e| anyhow!(e))?;
        }
        Ok(())
    }

    fn refresh_tasks(&mut self) -> Result<()> {
        self.tasks = self
            .storage
            .get_tasks(TaskFilter::default())
            .map_err(|e| anyhow!(e))?;
        if self.tasks.is_empty() {
            self.selected = 0;
        } else if self.selected >= self.tasks.len() {
            self.selected = self.tasks.len() - 1;
        }
        Ok(())
    }

    fn toggle_selected_task(&mut self) -> Result<()> {
        if self.tasks.is_empty() {
            return Ok(());
        }
        let mut task = self.tasks[self.selected].clone();
        task.status = match task.status {
            TaskStatus::Pending => TaskStatus::Completed,
            TaskStatus::Completed => TaskStatus::Pending,
            TaskStatus::Cancelled => TaskStatus::Pending,
        };
        task.updated_at = Utc::now();
        self.storage.update_task(&task).map_err(|e| anyhow!(e))?;
        self.refresh_tasks()?;
        Ok(())
    }

    fn select_next(&mut self) {
        if self.tasks.is_empty() {
            self.selected = 0;
            return;
        }
        self.selected = (self.selected + 1).min(self.tasks.len() - 1);
    }

    fn select_prev(&mut self) {
        if self.tasks.is_empty() {
            self.selected = 0;
            return;
        }
        self.selected = self.selected.saturating_sub(1);
    }
}

pub(crate) fn days_in_month(year: i32, month: u32) -> u32 {
    let next_month = if month == 12 { 1 } else { month + 1 };
    let next_year = if month == 12 { year + 1 } else { year };
    let first_next = NaiveDate::from_ymd_opt(next_year, next_month, 1)
        .unwrap_or_else(|| NaiveDate::from_ymd_opt(year, month, 28).unwrap());
    let last = first_next - chrono::Duration::days(1);
    last.day()
}
