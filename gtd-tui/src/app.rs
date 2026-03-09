use anyhow::{anyhow, Result};
use chrono::{Datelike, NaiveDate, Utc};
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use gtd_core::models::{ChecklistItem, Task, TaskStatus};
use gtd_core::storage::{SqliteStorage, Storage, TaskFilter};
use std::time::{Duration, Instant};
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
    pub checklist: Vec<ChecklistDraft>,
    pub checklist_index: usize,
    pub edit_active: bool,
    pub focus: Focus,
    pub date_picker: DatePickerState,
}

#[derive(Debug, Clone)]
pub struct ChecklistDraft {
    pub title: String,
    pub checked: bool,
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

use crate::config::KeysConfig;
use crate::ui::theme::{CalendarTheme, EditorTheme};

pub struct App {
    pub should_quit: bool,
    pub view: View,
    pub mode: Mode,
    pub tasks: Vec<Task>,
    pub selected: usize,
    pub editor: Option<EditorState>,
    pub calendar_theme: CalendarTheme,
    pub editor_theme: EditorTheme,
    pub keymap: Keymap,
    pub cursor_visible: bool,
    last_blink: Instant,
    storage: SqliteStorage,
}

impl App {
    pub fn new(
        storage: SqliteStorage,
        calendar_theme: CalendarTheme,
        editor_theme: EditorTheme,
        keymap: Keymap,
    ) -> Result<Self> {
        let mut app = Self {
            should_quit: false,
            view: View::Inbox,
            mode: Mode::Normal,
            tasks: Vec::new(),
            selected: 0,
            editor: None,
            calendar_theme,
            editor_theme,
            keymap,
            cursor_visible: true,
            last_blink: Instant::now(),
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

    pub fn on_tick(&mut self) {
        let now = Instant::now();
        if now.duration_since(self.last_blink) >= Duration::from_millis(500) {
            self.cursor_visible = !self.cursor_visible;
            self.last_blink = now;
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

        let (focus, edit_active) = match self.editor.as_ref() {
            Some(editor) => (editor.focus, editor.edit_active),
            None => return Ok(()),
        };

        if matches!(key.code, KeyCode::Up | KeyCode::Down)
            && focus != Focus::Checklist
            && !(focus == Focus::DueDate && edit_active && key.code == KeyCode::Down)
        {
            let delta = if key.code == KeyCode::Up { -1 } else { 1 };
            return self.switch_task(delta);
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
            KeyCode::Char('j') => {
                editor.focus = editor.focus.next();
                editor.edit_active = false;
                if editor.focus == Focus::Checklist {
                    if editor.checklist.is_empty() {
                        editor.checklist.push(ChecklistDraft {
                            title: String::new(),
                            checked: false,
                        });
                        editor.checklist_index = 0;
                    }
                    editor.edit_active = true;
                }
            }
            KeyCode::Char('k') => {
                editor.focus = editor.focus.prev();
                editor.edit_active = false;
                if editor.focus == Focus::Checklist {
                    if editor.checklist.is_empty() {
                        editor.checklist.push(ChecklistDraft {
                            title: String::new(),
                            checked: false,
                        });
                        editor.checklist_index = 0;
                    }
                    editor.edit_active = true;
                }
            }
            KeyCode::Char('h') if editor.focus == Focus::DueDate && editor.edit_active => {
                editor.date_picker.move_days(-1)
            }
            KeyCode::Char('l') if editor.focus == Focus::DueDate && editor.edit_active => {
                editor.date_picker.move_days(1)
            }
            KeyCode::Char('k') if editor.focus == Focus::DueDate && editor.edit_active => {
                editor.date_picker.move_days(-7)
            }
            KeyCode::Char('j') if editor.focus == Focus::DueDate && editor.edit_active => {
                editor.date_picker.move_days(7)
            }
            KeyCode::Char('h') if editor.focus == Focus::DueDate && !editor.edit_active => {
                let base = editor.due_date.unwrap_or_else(|| Utc::now().date_naive());
                let next = base - chrono::Duration::days(1);
                editor.due_date = Some(next);
                editor.date_picker.cursor = next;
            }
            KeyCode::Char('l') if editor.focus == Focus::DueDate && !editor.edit_active => {
                let base = editor.due_date.unwrap_or_else(|| Utc::now().date_naive());
                let next = base + chrono::Duration::days(1);
                editor.due_date = Some(next);
                editor.date_picker.cursor = next;
            }
            KeyCode::Down if editor.focus == Focus::DueDate && editor.edit_active => {
                editor.edit_active = false;
                editor.focus = Focus::Checklist;
                if editor.checklist.is_empty() {
                    editor.checklist.push(ChecklistDraft {
                        title: String::new(),
                        checked: false,
                    });
                }
                editor.checklist_index = 0;
                editor.edit_active = true;
            }
            KeyCode::Up if editor.focus == Focus::Checklist => {
                if editor.checklist_index > 0 {
                    editor.checklist_index -= 1;
                }
            }
            KeyCode::Down if editor.focus == Focus::Checklist => {
                if !editor.checklist.is_empty() {
                    editor.checklist_index = (editor.checklist_index + 1)
                        .min(editor.checklist.len().saturating_sub(1));
                }
            }
            KeyCode::Char('x') if editor.focus == Focus::Checklist && !editor.edit_active => {
                if let Some(item) = editor.checklist.get_mut(editor.checklist_index) {
                    item.checked = !item.checked;
                }
            }
            _ if self.keymap.checklist_edit_toggle.matches(key) => {
                editor.edit_active = !editor.edit_active;
                if editor.focus == Focus::DueDate {
                    if editor.edit_active {
                        if let Some(due) = editor.due_date {
                            editor.date_picker.cursor = due;
                        }
                    } else {
                        editor.due_date = Some(editor.date_picker.cursor);
                    }
                }
                return Ok(());
            }
            KeyCode::Char('p') if editor.focus == Focus::DueDate && editor.edit_active => {
                editor.date_picker.move_months(-1)
            }
            KeyCode::Char('n') if editor.focus == Focus::DueDate && editor.edit_active => {
                editor.date_picker.move_months(1)
            }
            KeyCode::Char('t') if editor.focus == Focus::DueDate && editor.edit_active => {
                let today = Utc::now().date_naive();
                editor.date_picker.cursor = today;
                editor.due_date = Some(today);
            }
            KeyCode::Char('m') if editor.focus == Focus::DueDate && editor.edit_active => {
                let tomorrow = Utc::now().date_naive() + chrono::Duration::days(1);
                editor.date_picker.cursor = tomorrow;
                editor.due_date = Some(tomorrow);
            }
            KeyCode::Enter => Self::handle_enter(editor)?,
            KeyCode::Backspace => Self::handle_backspace(editor),
            KeyCode::Char(ch) => Self::handle_char(editor, ch),
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
                editor.checklist.insert(
                    editor.checklist_index + 1,
                    ChecklistDraft {
                        title: String::new(),
                        checked: false,
                    },
                );
                editor.checklist_index += 1;
                editor.edit_active = true;
            }
        }
        Ok(())
    }

    fn handle_backspace(editor: &mut EditorState) {
        match editor.focus {
            Focus::Title => {
                if editor.edit_active {
                    editor.title.pop();
                }
            }
            Focus::Notes => {
                if editor.edit_active {
                    editor.notes.pop();
                }
            }
            Focus::DueDate => {
                editor.due_date = None;
            }
            Focus::Checklist => {
                if editor.checklist.is_empty() {
                    return;
                }
                if !editor.edit_active {
                    return;
                }
                let current = &mut editor.checklist[editor.checklist_index];
                if current.title.is_empty() {
                    editor.checklist.remove(editor.checklist_index);
                    if editor.checklist_index > 0 {
                        editor.checklist_index -= 1;
                    }
                } else {
                    current.title.pop();
                }
            }
        }
    }

    fn handle_char(editor: &mut EditorState, ch: char) {
        match editor.focus {
            Focus::Title => {
                if editor.edit_active {
                    editor.title.push(ch);
                }
            }
            Focus::Notes => {
                if editor.edit_active {
                    editor.notes.push(ch);
                }
            }
            Focus::DueDate => {}
            Focus::Checklist => {
                if !editor.edit_active {
                    return;
                }
                if editor.checklist.is_empty() {
                    editor.checklist.push(ChecklistDraft {
                        title: String::new(),
                        checked: false,
                    });
                    editor.checklist_index = 0;
                }
                editor.checklist[editor.checklist_index].title.push(ch);
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
            checklist: vec![ChecklistDraft {
                title: String::new(),
                checked: false,
            }],
            checklist_index: 0,
            edit_active: false,
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
            vec![ChecklistDraft {
                title: String::new(),
                checked: false,
            }]
        } else {
            checklist_items
                .into_iter()
                .map(|item| ChecklistDraft {
                    title: item.title,
                    checked: item.is_checked,
                })
                .collect()
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
            edit_active: false,
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

    fn replace_checklist(&self, task_id: Uuid, checklist: Vec<ChecklistDraft>) -> Result<()> {
        let existing = self
            .storage
            .get_checklist(task_id)
            .map_err(|e| anyhow!(e))?;
        for item in existing {
            self.storage
                .delete_checklist_item(item.id)
                .map_err(|e| anyhow!(e))?;
        }
        for (index, item) in checklist.into_iter().enumerate() {
            let title = item.title.trim();
            if title.is_empty() {
                continue;
            }
            let item = ChecklistItem {
                id: Uuid::new_v4(),
                task_id,
                title: title.to_string(),
                is_checked: item.checked,
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

    fn switch_task(&mut self, delta: i32) -> Result<()> {
        if self.tasks.is_empty() {
            return Ok(());
        }
        let current = self.selected as i32;
        let max_index = self.tasks.len().saturating_sub(1) as i32;
        let target = (current + delta).clamp(0, max_index) as usize;
        if target == self.selected {
            return Ok(());
        }
        self.save_edit()?;
        self.selected = target.min(self.tasks.len().saturating_sub(1));
        self.start_edit_task()?;
        Ok(())
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

#[derive(Debug, Clone, Copy)]
pub struct KeyBinding {
    pub ctrl: bool,
    pub key: char,
}

impl KeyBinding {
    pub fn matches(&self, event: KeyEvent) -> bool {
        event.code == KeyCode::Char(self.key)
            && event.modifiers.contains(KeyModifiers::CONTROL) == self.ctrl
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Keymap {
    pub checklist_edit_toggle: KeyBinding,
}

impl Keymap {
    pub fn from_config(config: &KeysConfig) -> Self {
        let default = KeyBinding { ctrl: true, key: 'e' };
        let checklist_edit_toggle = config
            .checklist_edit_toggle
            .as_deref()
            .and_then(parse_key_binding)
            .unwrap_or(default);
        Self {
            checklist_edit_toggle,
        }
    }
}

fn parse_key_binding(value: &str) -> Option<KeyBinding> {
    let mut ctrl = false;
    let mut key: Option<char> = None;
    for part in value.split('+') {
        let token = part.trim().to_lowercase();
        if token == "ctrl" || token == "control" {
            ctrl = true;
            continue;
        }
        if token.chars().count() == 1 {
            key = token.chars().next();
        }
    }
    key.map(|key| KeyBinding { ctrl, key })
}
