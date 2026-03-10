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
    pub layer: Layer,
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
        if let Some(next) = self
            .cursor
            .checked_add_signed(chrono::Duration::days(delta))
        {
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
        if self.keymap.quit.matches(key) || key.code == KeyCode::Esc {
            self.should_quit = true;
        } else if self.keymap.view_inbox.matches(key) || key.code == KeyCode::Char('i') {
            self.view = View::Inbox;
        } else if self.keymap.view_today.matches(key) || key.code == KeyCode::Char('t') {
            self.view = View::Today;
        } else if self.keymap.view_upcoming.matches(key) || key.code == KeyCode::Char('u') {
            self.view = View::Upcoming;
        } else if self.keymap.view_anytime.matches(key) || key.code == KeyCode::Char('a') {
            self.view = View::Anytime;
        } else if self.keymap.view_someday.matches(key) || key.code == KeyCode::Char('s') {
            self.view = View::Someday;
        } else if self.keymap.select_next.matches(key) || key.code == KeyCode::Down {
            self.select_next();
        } else if self.keymap.select_prev.matches(key) || key.code == KeyCode::Up {
            self.select_prev();
        } else if self.keymap.new_task.matches(key) {
            self.start_new_task();
        } else if self.keymap.edit_task.matches(key) && self.view == View::Inbox {
            self.start_edit_task()?;
        } else if self.keymap.toggle_task.matches(key) {
            self.toggle_selected_task()?;
        } else if self.keymap.refresh.matches(key) {
            self.refresh_tasks()?;
        }
        Ok(())
    }

    fn on_key_edit(&mut self, key: KeyEvent) -> Result<()> {
        if self.keymap.save_edit.matches(key) {
            return self.save_edit();
        }

        let editor = match self.editor.as_mut() {
            Some(editor) => editor,
            None => return Ok(()),
        };

        if key.code == KeyCode::Esc || self.keymap.cancel_edit.matches(key) {
            if editor.edit_active {
                editor.edit_active = false;
            } else if editor.layer == Layer::ChecklistItem {
                editor.layer = Layer::TaskItem;
                editor.checklist_index = 0;
            } else {
                self.cancel_edit();
            }
            return Ok(());
        }

        match editor.layer {
            Layer::TaskItem => self.handle_task_item_layer(key),
            Layer::ChecklistItem => self.handle_checklist_item_layer(key),
        }
        Ok(())
    }

    fn handle_task_item_layer(&mut self, key: KeyEvent) {
        let editor = match self.editor.as_mut() {
            Some(editor) => editor,
            None => return,
        };

        if editor.edit_active {
            self.handle_edit_mode(key);
            return;
        }

        if self.keymap.next_focus.matches(key) {
            editor.focus = editor.focus.next();
            if editor.focus == Focus::Checklist {
                if editor.checklist.is_empty() {
                    editor.checklist.push(ChecklistDraft {
                        title: String::new(),
                        checked: false,
                    });
                }
                editor.checklist_index = 0;
            }
        } else if self.keymap.prev_focus.matches(key) {
            editor.focus = editor.focus.prev();
            if editor.focus == Focus::Checklist {
                if editor.checklist.is_empty() {
                    editor.checklist.push(ChecklistDraft {
                        title: String::new(),
                        checked: false,
                    });
                }
                editor.checklist_index = 0;
            }
        } else if self.keymap.date_edit_mode.matches(key) && editor.focus == Focus::Checklist {
            editor.layer = Layer::ChecklistItem;
            if editor.checklist.is_empty() {
                editor.checklist.push(ChecklistDraft {
                    title: String::new(),
                    checked: false,
                });
            }
            editor.checklist_index = 0;
        } else if self.keymap.date_edit_mode.matches(key) {
            editor.edit_active = true;
            if editor.focus == Focus::DueDate {
                if let Some(due) = editor.due_date {
                    editor.date_picker.cursor = due;
                }
            }
        } else if key.code == KeyCode::Down && editor.focus == Focus::DueDate {
            editor.edit_active = true;
            editor.focus = Focus::Checklist;
            if editor.checklist.is_empty() {
                editor.checklist.push(ChecklistDraft {
                    title: String::new(),
                    checked: false,
                });
            }
            editor.checklist_index = 0;
        } else if self.keymap.checklist_toggle.matches(key) && editor.focus == Focus::Checklist {
            if let Some(item) = editor.checklist.get_mut(editor.checklist_index) {
                item.checked = !item.checked;
            }
        } else if self.keymap.checklist_edit_toggle.matches(key) {
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
        } else if self.keymap.date_prev_day.matches(key) && editor.focus == Focus::DueDate {
            let base = editor.due_date.unwrap_or_else(|| Utc::now().date_naive());
            let next = base - chrono::Duration::days(1);
            editor.due_date = Some(next);
            editor.date_picker.cursor = next;
        } else if self.keymap.date_next_day.matches(key) && editor.focus == Focus::DueDate {
            let base = editor.due_date.unwrap_or_else(|| Utc::now().date_naive());
            let next = base + chrono::Duration::days(1);
            editor.due_date = Some(next);
            editor.date_picker.cursor = next;
        }
    }

    fn handle_checklist_item_layer(&mut self, key: KeyEvent) {
        let editor = match self.editor.as_mut() {
            Some(editor) => editor,
            None => return,
        };

        if editor.edit_active {
            self.handle_checklist_edit_mode();
            return;
        }

        if self.keymap.prev_focus.matches(key) {
            if editor.checklist_index > 0 {
                editor.checklist_index -= 1;
            }
        } else if self.keymap.next_focus.matches(key) {
            if !editor.checklist.is_empty() {
                editor.checklist_index =
                    (editor.checklist_index + 1).min(editor.checklist.len().saturating_sub(1));
            }
        } else if self.keymap.date_edit_mode.matches(key) {
            editor.edit_active = true;
        } else if self.keymap.cancel_edit.matches(key) {
            editor.layer = Layer::TaskItem;
            editor.checklist_index = 0;
        } else if self.keymap.checklist_toggle.matches(key) {
            if let Some(item) = editor.checklist.get_mut(editor.checklist_index) {
                item.checked = !item.checked;
            }
        }
    }

    fn handle_edit_mode(&mut self, key: KeyEvent) {
        let editor = match self.editor.as_mut() {
            Some(editor) => editor,
            None => return,
        };

        match editor.focus {
            Focus::Title => {
                if let KeyCode::Char(ch) = key.code {
                    editor.title.push(ch);
                } else if key.code == KeyCode::Backspace {
                    editor.title.pop();
                } else if key.code == KeyCode::Enter {
                    editor.focus = Focus::Notes;
                    editor.edit_active = false;
                }
            }
            Focus::Notes => {
                if let KeyCode::Char(ch) = key.code {
                    editor.notes.push(ch);
                } else if key.code == KeyCode::Backspace {
                    editor.notes.pop();
                } else if key.code == KeyCode::Enter {
                    editor.focus = Focus::DueDate;
                    editor.edit_active = false;
                }
            }
            Focus::DueDate => {
                if self.keymap.date_prev_day_in_edit_mode.matches(key) {
                    editor.date_picker.move_days(-1);
                } else if self.keymap.date_next_day_in_edit_mode.matches(key) {
                    editor.date_picker.move_days(1);
                } else if self.keymap.date_prev_week.matches(key) {
                    editor.date_picker.move_days(-7);
                } else if self.keymap.date_next_week.matches(key) {
                    editor.date_picker.move_days(7);
                } else if self.keymap.date_prev_month.matches(key) {
                    editor.date_picker.move_months(-1);
                } else if self.keymap.date_next_month.matches(key) {
                    editor.date_picker.move_months(1);
                } else if self.keymap.date_today.matches(key) {
                    let today = Utc::now().date_naive();
                    editor.date_picker.cursor = today;
                    editor.due_date = Some(today);
                } else if self.keymap.date_tomorrow.matches(key) {
                    let tomorrow = Utc::now().date_naive() + chrono::Duration::days(1);
                    editor.date_picker.cursor = tomorrow;
                    editor.due_date = Some(tomorrow);
                } else if key.code == KeyCode::Enter {
                    editor.due_date = Some(editor.date_picker.cursor);
                    editor.focus = Focus::Checklist;
                    editor.edit_active = false;
                    if editor.checklist.is_empty() {
                        editor.checklist.push(ChecklistDraft {
                            title: String::new(),
                            checked: false,
                        });
                    }
                    editor.checklist_index = 0;
                } else if key.code == KeyCode::Down {
                    editor.edit_active = false;
                    editor.focus = Focus::Checklist;
                    if editor.checklist.is_empty() {
                        editor.checklist.push(ChecklistDraft {
                            title: String::new(),
                            checked: false,
                        });
                    }
                    editor.checklist_index = 0;
                }
            }
            Focus::Checklist => {
                if let KeyCode::Char(ch) = key.code {
                    if editor.checklist.is_empty() {
                        editor.checklist.push(ChecklistDraft {
                            title: String::new(),
                            checked: false,
                        });
                        editor.checklist_index = 0;
                    }
                    editor.checklist[editor.checklist_index].title.push(ch);
                } else if key.code == KeyCode::Backspace {
                    if !editor.checklist.is_empty() {
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
                } else if key.code == KeyCode::Enter {
                    editor.checklist.insert(
                        editor.checklist_index + 1,
                        ChecklistDraft {
                            title: String::new(),
                            checked: false,
                        },
                    );
                    editor.checklist_index += 1;
                }
            }
        }
    }

    fn handle_checklist_edit_mode(&mut self) {
        let editor = match self.editor.as_mut() {
            Some(editor) => editor,
            None => return,
        };

        if editor.checklist.is_empty() {
            editor.checklist.push(ChecklistDraft {
                title: String::new(),
                checked: false,
            });
            editor.checklist_index = 0;
        }
    }

    fn start_new_task(&mut self) {
        let insert_after = if self.tasks.is_empty() {
            0
        } else {
            self.selected
        };
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
            layer: Layer::TaskItem,
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
            layer: Layer::TaskItem,
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
    pub quit: KeyBinding,
    pub view_inbox: KeyBinding,
    pub view_today: KeyBinding,
    pub view_upcoming: KeyBinding,
    pub view_anytime: KeyBinding,
    pub view_someday: KeyBinding,
    pub select_next: KeyBinding,
    pub select_prev: KeyBinding,
    pub new_task: KeyBinding,
    pub edit_task: KeyBinding,
    pub toggle_task: KeyBinding,
    pub refresh: KeyBinding,
    pub save_edit: KeyBinding,
    pub cancel_edit: KeyBinding,
    pub next_focus: KeyBinding,
    pub prev_focus: KeyBinding,
    pub checklist_edit_toggle: KeyBinding,
    pub date_prev_day: KeyBinding,
    pub date_next_day: KeyBinding,
    pub date_prev_day_in_edit_mode: KeyBinding,
    pub date_next_day_in_edit_mode: KeyBinding,
    pub date_prev_week: KeyBinding,
    pub date_next_week: KeyBinding,
    pub date_edit_mode: KeyBinding,
    pub date_prev_month: KeyBinding,
    pub date_next_month: KeyBinding,
    pub date_today: KeyBinding,
    pub date_tomorrow: KeyBinding,
    pub checklist_toggle: KeyBinding,
    pub checklist_add: KeyBinding,
    pub checklist_next: KeyBinding,
    pub checklist_prev: KeyBinding,
}

impl Keymap {
    pub fn default_keymap() -> Self {
        Self {
            quit: KeyBinding {
                ctrl: false,
                key: 'q',
            },
            view_inbox: KeyBinding {
                ctrl: false,
                key: '1',
            },
            view_today: KeyBinding {
                ctrl: false,
                key: '2',
            },
            view_upcoming: KeyBinding {
                ctrl: false,
                key: '3',
            },
            view_anytime: KeyBinding {
                ctrl: false,
                key: '4',
            },
            view_someday: KeyBinding {
                ctrl: false,
                key: '5',
            },
            select_next: KeyBinding {
                ctrl: false,
                key: 'j',
            },
            select_prev: KeyBinding {
                ctrl: false,
                key: 'k',
            },
            new_task: KeyBinding {
                ctrl: false,
                key: 'n',
            },
            edit_task: KeyBinding {
                ctrl: false,
                key: 'l',
            },
            toggle_task: KeyBinding {
                ctrl: false,
                key: 'x',
            },
            refresh: KeyBinding {
                ctrl: false,
                key: 'r',
            },
            save_edit: KeyBinding {
                ctrl: true,
                key: 's',
            },
            cancel_edit: KeyBinding {
                ctrl: false,
                key: 'q',
            },
            next_focus: KeyBinding {
                ctrl: false,
                key: 'j',
            },
            prev_focus: KeyBinding {
                ctrl: false,
                key: 'k',
            },
            checklist_edit_toggle: KeyBinding {
                ctrl: true,
                key: 'e',
            },
            date_prev_day: KeyBinding {
                ctrl: true,
                key: 'h',
            },
            date_next_day: KeyBinding {
                ctrl: true,
                key: 'l',
            },
            date_prev_day_in_edit_mode: KeyBinding {
                ctrl: false,
                key: 'h',
            },
            date_next_day_in_edit_mode: KeyBinding {
                ctrl: false,
                key: 'l',
            },
            date_prev_week: KeyBinding {
                ctrl: false,
                key: 'k',
            },
            date_next_week: KeyBinding {
                ctrl: false,
                key: 'j',
            },
            date_edit_mode: KeyBinding {
                ctrl: false,
                key: 'l',
            },
            date_prev_month: KeyBinding {
                ctrl: false,
                key: 'p',
            },
            date_next_month: KeyBinding {
                ctrl: false,
                key: 'n',
            },
            date_today: KeyBinding {
                ctrl: false,
                key: 't',
            },
            date_tomorrow: KeyBinding {
                ctrl: false,
                key: 'm',
            },
            checklist_toggle: KeyBinding {
                ctrl: false,
                key: 'x',
            },
            checklist_add: KeyBinding {
                ctrl: false,
                key: 'l',
            },
            checklist_next: KeyBinding {
                ctrl: false,
                key: 'j',
            },
            checklist_prev: KeyBinding {
                ctrl: false,
                key: 'k',
            },
        }
    }

    pub fn from_config(config: &KeysConfig) -> Self {
        let default = Self::default_keymap();
        Self {
            quit: config
                .quit
                .as_deref()
                .and_then(parse_key_binding)
                .unwrap_or(default.quit),
            view_inbox: config
                .view_inbox
                .as_deref()
                .and_then(parse_key_binding)
                .unwrap_or(default.view_inbox),
            view_today: config
                .view_today
                .as_deref()
                .and_then(parse_key_binding)
                .unwrap_or(default.view_today),
            view_upcoming: config
                .view_upcoming
                .as_deref()
                .and_then(parse_key_binding)
                .unwrap_or(default.view_upcoming),
            view_anytime: config
                .view_anytime
                .as_deref()
                .and_then(parse_key_binding)
                .unwrap_or(default.view_anytime),
            view_someday: config
                .view_someday
                .as_deref()
                .and_then(parse_key_binding)
                .unwrap_or(default.view_someday),
            select_next: config
                .select_next
                .as_deref()
                .and_then(parse_key_binding)
                .unwrap_or(default.select_next),
            select_prev: config
                .select_prev
                .as_deref()
                .and_then(parse_key_binding)
                .unwrap_or(default.select_prev),
            new_task: config
                .new_task
                .as_deref()
                .and_then(parse_key_binding)
                .unwrap_or(default.new_task),
            edit_task: config
                .edit_task
                .as_deref()
                .and_then(parse_key_binding)
                .unwrap_or(default.edit_task),
            toggle_task: config
                .toggle_task
                .as_deref()
                .and_then(parse_key_binding)
                .unwrap_or(default.toggle_task),
            refresh: config
                .refresh
                .as_deref()
                .and_then(parse_key_binding)
                .unwrap_or(default.refresh),
            save_edit: config
                .save_edit
                .as_deref()
                .and_then(parse_key_binding)
                .unwrap_or(default.save_edit),
            cancel_edit: config
                .cancel_edit
                .as_deref()
                .and_then(parse_key_binding)
                .unwrap_or(default.cancel_edit),
            next_focus: config
                .next_focus
                .as_deref()
                .and_then(parse_key_binding)
                .unwrap_or(default.next_focus),
            prev_focus: config
                .prev_focus
                .as_deref()
                .and_then(parse_key_binding)
                .unwrap_or(default.prev_focus),
            checklist_edit_toggle: config
                .checklist_edit_toggle
                .as_deref()
                .and_then(parse_key_binding)
                .unwrap_or(default.checklist_edit_toggle),
            date_prev_day: config
                .date_prev_day
                .as_deref()
                .and_then(parse_key_binding)
                .unwrap_or(default.date_prev_day),
            date_next_day: config
                .date_next_day
                .as_deref()
                .and_then(parse_key_binding)
                .unwrap_or(default.date_next_day),
            date_prev_day_in_edit_mode: config
                .date_prev_day
                .as_deref()
                .and_then(parse_key_binding)
                .unwrap_or(default.date_prev_day_in_edit_mode),
            date_next_day_in_edit_mode: config
                .date_next_day
                .as_deref()
                .and_then(parse_key_binding)
                .unwrap_or(default.date_next_day_in_edit_mode),
            date_prev_week: config
                .date_prev_week
                .as_deref()
                .and_then(parse_key_binding)
                .unwrap_or(default.date_prev_week),
            date_next_week: config
                .date_next_week
                .as_deref()
                .and_then(parse_key_binding)
                .unwrap_or(default.date_next_week),
            date_edit_mode: config
                .date_edit_mode
                .as_deref()
                .and_then(parse_key_binding)
                .unwrap_or(default.date_edit_mode),
            date_prev_month: config
                .date_prev_month
                .as_deref()
                .and_then(parse_key_binding)
                .unwrap_or(default.date_prev_month),
            date_next_month: config
                .date_next_month
                .as_deref()
                .and_then(parse_key_binding)
                .unwrap_or(default.date_next_month),
            date_today: config
                .date_today
                .as_deref()
                .and_then(parse_key_binding)
                .unwrap_or(default.date_today),
            date_tomorrow: config
                .date_tomorrow
                .as_deref()
                .and_then(parse_key_binding)
                .unwrap_or(default.date_tomorrow),
            checklist_toggle: config
                .checklist_toggle
                .as_deref()
                .and_then(parse_key_binding)
                .unwrap_or(default.checklist_toggle),
            checklist_add: config
                .checklist_add
                .as_deref()
                .and_then(parse_key_binding)
                .unwrap_or(default.checklist_add),
            checklist_next: config
                .checklist_next
                .as_deref()
                .and_then(parse_key_binding)
                .unwrap_or(default.checklist_next),
            checklist_prev: config
                .checklist_prev
                .as_deref()
                .and_then(parse_key_binding)
                .unwrap_or(default.checklist_prev),
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
