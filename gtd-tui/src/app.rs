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
    pub delete_confirm: Option<DeleteTarget>,
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
            delete_confirm: None,
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
            Mode::ConfirmDelete => self.on_key_confirm_delete(key),
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
        if self.keymap.quit.matches(key)
            || (key.modifiers.contains(KeyModifiers::CONTROL) && key.code == KeyCode::Char('c'))
        {
            self.should_quit = true;
        } else if self.keymap.view_inbox.matches(key) || key.code == KeyCode::Char('i') {
            self.view = View::Inbox;
        } else if self.keymap.view_today.matches(key) || key.code == KeyCode::Char('t') {
            self.view = View::Today;
        } else if self.keymap.view_upcoming.matches(key) || key.code == KeyCode::Char('u') {
            self.view = View::Upcoming;
        } else if self.keymap.view_anytime.matches(key) {
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
        } else if key.code == KeyCode::Char('d') && !self.tasks.is_empty() {
            self.mode = Mode::ConfirmDelete;
            self.delete_confirm = Some(DeleteTarget::Task);
        } else if self.keymap.new_item_below.matches(key) {
            self.start_new_task();
        } else if self.keymap.new_item_above.matches(key) {
            let insert_after = self.selected.saturating_sub(1);
            let insert_at_beginning = self.selected == 0;
            self.start_new_task_at(insert_after, insert_at_beginning);
        } else if self.keymap.move_item_down.matches(key) {
            self.move_task_down()?;
        } else if self.keymap.move_item_up.matches(key) {
            self.move_task_up()?;
        }

        Ok(())
    }

    fn on_key_edit(&mut self, key: KeyEvent) -> Result<()> {
        if self.keymap.save_edit.matches(key) {
            return self.save_edit();
        }

        // Auto-save if Esc is pressed while editing (before getting mutable reference)
        if key.code == KeyCode::Esc && self.editor.as_ref().map(|e| e.edit_active).unwrap_or(false) {
            let _ = self.auto_save_current_edit();
        }

        let editor = match self.editor.as_mut() {
            Some(editor) => editor,
            None => return Ok(()),
        };

        if key.code == KeyCode::Esc {
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

        if key.code == KeyCode::Char('d')
            && !editor.edit_active
            && editor.layer == Layer::ChecklistItem
        {
            self.mode = Mode::ConfirmDelete;
            self.delete_confirm = Some(DeleteTarget::ChecklistItem);
            return Ok(());
        }

        if self.keymap.nav_up.matches(key) {
            // go up
            if !editor.edit_active && editor.layer == Layer::ChecklistItem {
                // go up: focus on task.property(Title, Notes, Due, Checklist) from checklist.item
                editor.layer = Layer::TaskItem;
                editor.checklist_index = 0;
            } else if editor.layer == Layer::TaskItem && !editor.edit_active {
                // focus on task.item from task.property
                self.cancel_edit();
                return Ok(());
            }
        }

        match editor.layer {
            Layer::TaskItem => self.handle_task_item_layer(key),
            Layer::ChecklistItem => self.handle_checklist_item_layer(key),
        }
        Ok(())
    }

    fn handle_task_item_layer(&mut self, key: KeyEvent) {
        // Auto-save before changing focus, entering checklist items, or toggling off edit mode
        let needs_auto_save = if let Some(editor) = self.editor.as_ref() {
            editor.edit_active && (
                self.keymap.next_focus.matches(key) ||
                self.keymap.prev_focus.matches(key) ||
                (self.keymap.date_edit_mode.matches(key) && editor.focus == Focus::Checklist) ||
                (self.keymap.checklist_edit_toggle.matches(key) && editor.focus != Focus::DueDate)
            )
        } else {
            false
        };
        if needs_auto_save {
            let _ = self.auto_save_current_edit();
        }

        let editor = match self.editor.as_mut() {
            Some(editor) => editor,
            None => return,
        };

        // Handle checklist_edit_toggle specially - it toggles edit mode
        if self.keymap.checklist_edit_toggle.matches(key) {
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
            return;
        }

        // Handle Enter key in DueDate edit mode - save date before focus change
        if editor.edit_active
            && editor.focus == Focus::DueDate
            && key.code == KeyCode::Enter
        {
            editor.due_date = Some(editor.date_picker.cursor);
            let _ = self.auto_save_current_edit();
            self.handle_edit_mode(key); // This will handle focus change
            return;
        }

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
        } else if self.keymap.date_prev_day.matches(key) && editor.focus == Focus::DueDate {
            let base = editor.due_date.unwrap_or_else(|| Utc::now().date_naive());
            let next = base - chrono::Duration::days(1);
            editor.due_date = Some(next);
            editor.date_picker.cursor = next;
            let _ = self.auto_save_current_edit();
        } else if self.keymap.date_next_day.matches(key) && editor.focus == Focus::DueDate {
            let base = editor.due_date.unwrap_or_else(|| Utc::now().date_naive());
            let next = base + chrono::Duration::days(1);
            editor.due_date = Some(next);
            editor.date_picker.cursor = next;
            let _ = self.auto_save_current_edit();
        } else if self.keymap.date_today.matches(key)
            && editor.focus == Focus::DueDate
            && !editor.edit_active
        {
            let today = Utc::now().date_naive();
            editor.due_date = Some(today);
            editor.date_picker.cursor = today;
            let _ = self.auto_save_current_edit();
        } else if self.keymap.date_tomorrow.matches(key)
            && editor.focus == Focus::DueDate
            && !editor.edit_active
        {
            let tomorrow = Utc::now().date_naive() + chrono::Duration::days(1);
            editor.due_date = Some(tomorrow);
            editor.date_picker.cursor = tomorrow;
            let _ = self.auto_save_current_edit();
        } else if key.code == KeyCode::Char('o') && editor.focus == Focus::Checklist {
            let shift_pressed = key.modifiers.contains(KeyModifiers::SHIFT);
            if shift_pressed {
                editor.checklist.insert(
                    0,
                    ChecklistDraft {
                        title: String::new(),
                        checked: false,
                    },
                );
                editor.checklist_index = 0;
            } else {
                editor.checklist.push(ChecklistDraft {
                    title: String::new(),
                    checked: false,
                });
                editor.checklist_index = editor.checklist.len() - 1;
            }
            editor.layer = Layer::ChecklistItem;
            editor.edit_active = true;
        }
    }

    fn handle_checklist_item_layer(&mut self, key: KeyEvent) {
        let editor = match self.editor.as_mut() {
            Some(editor) => editor,
            None => return,
        };

        if editor.edit_active {
            self.handle_checklist_edit_mode(&key);
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
        } else if self.keymap.nav_up.matches(key) && !editor.edit_active {
            editor.layer = Layer::TaskItem;
            editor.checklist_index = 0;
        } else if self.keymap.checklist_toggle.matches(key) {
            if let Some(item) = editor.checklist.get_mut(editor.checklist_index) {
                item.checked = !item.checked;
            }
        } else if self.keymap.new_item_below.matches(key) {
            // let shift_pressed = key.modifiers.contains(KeyModifiers::SHIFT);
            editor.checklist.insert(
                editor.checklist_index + 1,
                ChecklistDraft {
                    title: String::new(),
                    checked: false,
                },
            );
            editor.checklist_index += 1;
            editor.edit_active = true;
        } else if self.keymap.new_item_above.matches(key) {
            editor.checklist.insert(
                editor.checklist_index,
                ChecklistDraft {
                    title: String::new(),
                    checked: false,
                },
            );
            editor.edit_active = true;
        } else if self.keymap.move_item_down.matches(key) {
            if editor.checklist_index < editor.checklist.len() - 1 {
                editor
                    .checklist
                    .swap(editor.checklist_index, editor.checklist_index + 1);
                editor.checklist_index += 1;
            }
        } else if self.keymap.move_item_up.matches(key) {
            if editor.checklist_index > 0 {
                editor
                    .checklist
                    .swap(editor.checklist_index, editor.checklist_index - 1);
                editor.checklist_index -= 1;
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
                    editor.due_date = Some(editor.date_picker.cursor);
                    let _ = self.auto_save_current_edit();
                } else if self.keymap.date_next_day_in_edit_mode.matches(key) {
                    editor.date_picker.move_days(1);
                    editor.due_date = Some(editor.date_picker.cursor);
                    let _ = self.auto_save_current_edit();
                } else if self.keymap.date_prev_week.matches(key) {
                    editor.date_picker.move_days(-7);
                    editor.due_date = Some(editor.date_picker.cursor);
                    let _ = self.auto_save_current_edit();
                } else if self.keymap.date_next_week.matches(key) {
                    editor.date_picker.move_days(7);
                    editor.due_date = Some(editor.date_picker.cursor);
                    let _ = self.auto_save_current_edit();
                } else if self.keymap.date_prev_month.matches(key) {
                    editor.date_picker.move_months(-1);
                    editor.due_date = Some(editor.date_picker.cursor);
                    let _ = self.auto_save_current_edit();
                } else if self.keymap.date_next_month.matches(key) {
                    editor.date_picker.move_months(1);
                    editor.due_date = Some(editor.date_picker.cursor);
                    let _ = self.auto_save_current_edit();
                } else if self.keymap.date_today.matches(key) {
                    let today = Utc::now().date_naive();
                    editor.date_picker.cursor = today;
                    editor.due_date = Some(today);
                    let _ = self.auto_save_current_edit();
                } else if self.keymap.date_tomorrow.matches(key) {
                    let tomorrow = Utc::now().date_naive() + chrono::Duration::days(1);
                    editor.date_picker.cursor = tomorrow;
                    editor.due_date = Some(tomorrow);
                    let _ = self.auto_save_current_edit();
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
                // no op for checklist header
            }
        }
    }

    fn handle_checklist_edit_mode(&mut self, key: &KeyEvent) {
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
        } else {
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

    fn start_new_task(&mut self) {
        self.start_new_task_at(
            if self.tasks.is_empty() {
                0
            } else {
                self.selected
            },
            false,
        );
    }

    fn start_new_task_at(&mut self, insert_after: usize, insert_at_beginning: bool) {
        let today = Utc::now().date_naive();
        self.editor = Some(EditorState {
            task_id: None,
            insert_after,
            insert_at_beginning,
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
            insert_at_beginning: false,
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
            let insert_index = if self.tasks.is_empty() || editor.insert_at_beginning {
                0
            } else {
                (editor.insert_after + 1).min(self.tasks.len())
            };
            self.reindex_tasks_for_insert(insert_index, now)?;
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
                sort_order: insert_index as i32,
                created_at: now,
                updated_at: now,
            };
            self.storage.create_task(&task).map_err(|e| anyhow!(e))?;
            self.selected = insert_index;
            task_id
        };

        self.replace_checklist(task_id, editor.checklist)?;
        self.mode = Mode::Normal;
        self.editor = None;
        self.refresh_tasks()?;
        Ok(())
    }

    fn reindex_tasks_for_insert(
        &mut self,
        insert_index: usize,
        now: chrono::DateTime<Utc>,
    ) -> Result<()> {
        for (index, task) in self.tasks.iter_mut().enumerate() {
            let desired_sort_order = if index < insert_index {
                index as i32
            } else {
                (index + 1) as i32
            };
            if task.sort_order == desired_sort_order {
                continue;
            }
            task.sort_order = desired_sort_order;
            task.updated_at = now;
            self.storage.update_task(task).map_err(|e| anyhow!(e))?;
        }
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

    /// Auto-save the current field being edited without exiting edit mode.
    /// This is called when text input loses focus (edit_active becomes false).
    fn auto_save_current_edit(&mut self) -> Result<()> {
        // Extract needed data before any borrowing
        let (task_id, layer, focus, new_title, new_notes, due_date, checklist_items) = match self.editor.as_ref() {
            Some(editor) => (
                editor.task_id,
                editor.layer,
                editor.focus,
                editor.title.clone(),
                editor.notes.clone(),
                editor.due_date,
                editor.checklist.clone(),
            ),
            None => return Ok(()),
        };

        let Some(task_id) = task_id else {
            // New task - no need to auto-save until full save
            return Ok(());
        };

        // Find the task in our list
        let task = match self.tasks.iter().position(|t| t.id == task_id) {
            Some(idx) => &self.tasks[idx],
            None => return Ok(()),
        };

        let now = Utc::now();
        let mut task_clone = task.clone();
        let mut needs_task_update = false;
        let mut needs_checklist_update = false;

        match (layer, focus) {
            (Layer::TaskItem, Focus::Title) => {
                if task.title != new_title {
                    task_clone.title = new_title;
                    needs_task_update = true;
                }
            }
            (Layer::TaskItem, Focus::Notes) => {
                let notes = if new_notes.trim().is_empty() {
                    None
                } else {
                    Some(new_notes)
                };
                if task.notes != notes {
                    task_clone.notes = notes;
                    needs_task_update = true;
                }
            }
            (Layer::TaskItem, Focus::DueDate) => {
                // Save due_date changes (for date picker Enter key)
                if task.due_date != due_date {
                    task_clone.due_date = due_date;
                    needs_task_update = true;
                }
            }
            (Layer::ChecklistItem, _) => {
                // Save checklist item title changes
                if let Ok(existing_checklist) = self.storage.get_checklist(task_id) {
                    let has_changes = checklist_items.iter().enumerate().any(|(i, item)| {
                        existing_checklist.get(i).map(|e| e.title.as_str()) != Some(item.title.as_str())
                    });
                    if has_changes {
                        needs_checklist_update = true;
                    }
                }
            }
            _ => {
                // Checklist in TaskItem layer is saved via replace_checklist
            }
        }

        if needs_task_update {
            task_clone.updated_at = now;
            self.storage.update_task(&task_clone).map_err(|e| anyhow!(e))?;

            // Update local task list
            if let Some(idx) = self.tasks.iter().position(|t| t.id == task_id) {
                self.tasks[idx] = task_clone;
            }
        }

        if needs_checklist_update {
            self.replace_checklist(task_id, checklist_items)
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

    fn move_task_down(&mut self) -> Result<()> {
        if self.tasks.is_empty() || self.selected >= self.tasks.len() - 1 {
            return Ok(());
        }
        let idx = self.selected;
        let mut task1 = self.tasks[idx].clone();
        let mut task2 = self.tasks[idx + 1].clone();
        let temp_order = task1.sort_order;
        task1.sort_order = task2.sort_order;
        task2.sort_order = temp_order;
        task1.updated_at = Utc::now();
        task2.updated_at = Utc::now();
        self.storage.update_task(&task1).map_err(|e| anyhow!(e))?;
        self.storage.update_task(&task2).map_err(|e| anyhow!(e))?;
        self.selected += 1;
        self.refresh_tasks()?;
        Ok(())
    }

    fn move_task_up(&mut self) -> Result<()> {
        if self.tasks.is_empty() || self.selected == 0 {
            return Ok(());
        }
        let idx = self.selected;
        let mut task1 = self.tasks[idx].clone();
        let mut task2 = self.tasks[idx - 1].clone();
        let temp_order = task1.sort_order;
        task1.sort_order = task2.sort_order;
        task2.sort_order = temp_order;
        task1.updated_at = Utc::now();
        task2.updated_at = Utc::now();
        self.storage.update_task(&task1).map_err(|e| anyhow!(e))?;
        self.storage.update_task(&task2).map_err(|e| anyhow!(e))?;
        self.selected -= 1;
        self.refresh_tasks()?;
        Ok(())
    }

    fn on_key_confirm_delete(&mut self, key: KeyEvent) -> Result<()> {
        if key.code == KeyCode::Char('y') || key.code == KeyCode::Enter {
            self.confirm_delete()?;
        }

        match self.delete_confirm {
            Some(DeleteTarget::ChecklistItem) => {
                self.mode = Mode::Editing;
            }
            _ => {
                self.mode = Mode::Normal;
            }
        }
        self.delete_confirm = None;
        Ok(())
    }

    fn confirm_delete(&mut self) -> Result<()> {
        match self.delete_confirm {
            Some(DeleteTarget::Task) => {
                if self.tasks.is_empty() {
                    return Ok(());
                }
                let task = &self.tasks[self.selected];
                self.storage.delete_task(task.id).map_err(|e| anyhow!(e))?;
                if self.selected >= self.tasks.len().saturating_sub(1) {
                    self.selected = self.selected.saturating_sub(1);
                }
                self.refresh_tasks()?;
            }
            Some(DeleteTarget::ChecklistItem) => {
                if let Some(editor) = &mut self.editor {
                    let deleted_index = editor.checklist_index;
                    if deleted_index < editor.checklist.len() {
                        editor.checklist.remove(deleted_index);
                        if editor.checklist.is_empty() {
                            editor.layer = Layer::TaskItem;
                            editor.focus = Focus::Checklist;
                            editor.checklist_index = 0;
                        } else if deleted_index < editor.checklist.len() {
                        } else if !editor.checklist.is_empty() {
                            editor.checklist_index = editor.checklist.len() - 1;
                        }
                    }
                }
            }
            None => {}
        }
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct KeyBinding {
    pub ctrl: bool,
    pub shift: bool,
    pub key: char,
}

impl KeyBinding {
    pub fn matches(&self, event: KeyEvent) -> bool {
        let event_char = match event.code {
            KeyCode::Char(ch) => ch,
            _ => return false,
        };
        let shift_pressed = event.modifiers.contains(KeyModifiers::SHIFT);
        let shift_matches = if self.shift {
            shift_pressed || event_char.is_ascii_uppercase()
        } else {
            !shift_pressed && !event_char.is_ascii_uppercase()
        };

        event_char.to_ascii_lowercase() == self.key
            && event.modifiers.contains(KeyModifiers::CONTROL) == self.ctrl
            && shift_matches
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
    pub nav_up: KeyBinding,
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
    pub new_item_above: KeyBinding,
    pub new_item_below: KeyBinding,
    pub move_item_up: KeyBinding,
    pub move_item_down: KeyBinding,
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
                shift: false,
                key: 'q',
            },
            refresh: KeyBinding {
                ctrl: false,
                shift: false,
                key: 'r',
            },
            view_inbox: KeyBinding {
                ctrl: false,
                shift: false,
                key: '1',
            },
            view_today: KeyBinding {
                ctrl: false,
                shift: false,
                key: '2',
            },
            view_upcoming: KeyBinding {
                ctrl: false,
                shift: false,
                key: '3',
            },
            view_anytime: KeyBinding {
                ctrl: false,
                shift: false,
                key: '4',
            },
            view_someday: KeyBinding {
                ctrl: false,
                shift: false,
                key: '5',
            },
            select_next: KeyBinding {
                ctrl: false,
                shift: false,
                key: 'j',
            },
            select_prev: KeyBinding {
                ctrl: false,
                shift: false,
                key: 'k',
            },
            new_task: KeyBinding {
                ctrl: false,
                shift: false,
                key: 'n',
            },
            edit_task: KeyBinding {
                ctrl: false,
                shift: false,
                key: 'l',
            },
            toggle_task: KeyBinding {
                ctrl: false,
                shift: false,
                key: 'x',
            },
            save_edit: KeyBinding {
                ctrl: true,
                shift: false,
                key: 's',
            },
            nav_up: KeyBinding {
                ctrl: false,
                shift: false,
                key: 'q',
            },
            next_focus: KeyBinding {
                ctrl: false,
                shift: false,
                key: 'j',
            },
            prev_focus: KeyBinding {
                ctrl: false,
                shift: false,
                key: 'k',
            },
            checklist_edit_toggle: KeyBinding {
                ctrl: true,
                shift: false,
                key: 'e',
            },
            date_prev_day: KeyBinding {
                ctrl: true,
                shift: false,
                key: 'h',
            },
            date_next_day: KeyBinding {
                ctrl: true,
                shift: false,
                key: 'l',
            },
            date_prev_day_in_edit_mode: KeyBinding {
                ctrl: false,
                shift: false,
                key: 'h',
            },
            date_next_day_in_edit_mode: KeyBinding {
                ctrl: false,
                shift: false,
                key: 'l',
            },
            date_prev_week: KeyBinding {
                ctrl: false,
                shift: false,
                key: 'k',
            },
            date_next_week: KeyBinding {
                ctrl: false,
                shift: false,
                key: 'j',
            },
            date_edit_mode: KeyBinding {
                ctrl: false,
                shift: false,
                key: 'l',
            },
            date_prev_month: KeyBinding {
                ctrl: false,
                shift: false,
                key: 'p',
            },
            date_next_month: KeyBinding {
                ctrl: false,
                shift: false,
                key: 'n',
            },
            date_today: KeyBinding {
                ctrl: false,
                shift: false,
                key: 't',
            },
            date_tomorrow: KeyBinding {
                ctrl: false,
                shift: false,
                key: 'm',
            },
            new_item_above: KeyBinding {
                ctrl: false,
                shift: true,
                key: 'o',
            },
            new_item_below: KeyBinding {
                ctrl: false,
                shift: false,
                key: 'o',
            },
            move_item_up: KeyBinding {
                ctrl: false,
                shift: true,
                key: 'k',
            },
            move_item_down: KeyBinding {
                ctrl: false,
                shift: true,
                key: 'j',
            },
            checklist_toggle: KeyBinding {
                ctrl: false,
                shift: false,
                key: 'x',
            },
            checklist_add: KeyBinding {
                ctrl: false,
                shift: false,
                key: 'l',
            },
            checklist_next: KeyBinding {
                ctrl: false,
                shift: false,
                key: 'j',
            },
            checklist_prev: KeyBinding {
                ctrl: false,
                shift: false,
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
            nav_up: config
                .cancel_edit
                .as_deref()
                .and_then(parse_key_binding)
                .unwrap_or(default.nav_up),
            next_focus: config
                .next_focus
                .as_deref()
                .and_then(parse_key_binding)
                .unwrap_or(default.next_focus),
            new_item_above: config
                .new_item_above
                .as_deref()
                .and_then(parse_key_binding)
                .unwrap_or(default.new_item_above),
            new_item_below: config
                .new_item_below
                .as_deref()
                .and_then(parse_key_binding)
                .unwrap_or(default.new_item_below),
            move_item_up: config
                .move_item_up
                .as_deref()
                .and_then(parse_key_binding)
                .unwrap_or(default.move_item_up),
            move_item_down: config
                .move_item_down
                .as_deref()
                .and_then(parse_key_binding)
                .unwrap_or(default.move_item_down),
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ui::theme::{CalendarTheme, EditorTheme};
    use gtd_core::storage::Storage;

    fn test_app() -> App {
        let db_path = std::env::temp_dir().join(format!("gtd-tui-{}.db", Uuid::new_v4()));
        let storage = SqliteStorage::new(&db_path).expect("sqlite storage");
        App::new(
            storage,
            CalendarTheme::default(),
            EditorTheme::default(),
            Keymap::default_keymap(),
        )
        .expect("app")
    }

    #[test]
    fn parses_uppercase_binding_as_shifted_key() {
        let binding = parse_key_binding("A").expect("binding");

        assert_eq!(
            binding,
            KeyBinding {
                ctrl: false,
                shift: true,
                key: 'a',
            }
        );
    }

    #[test]
    fn new_task_is_inserted_after_selected_task() {
        let mut app = test_app();
        let now = Utc::now();
        for (sort_order, title) in ["First", "Second", "Third"].into_iter().enumerate() {
            let task = Task {
                id: Uuid::new_v4(),
                project_id: None,
                heading_id: None,
                area_id: None,
                title: title.to_string(),
                notes: None,
                status: TaskStatus::Pending,
                start_date: None,
                due_date: None,
                is_today: false,
                is_someday: false,
                sort_order: sort_order as i32,
                created_at: now,
                updated_at: now,
            };
            app.storage.create_task(&task).expect("task");
        }
        app.refresh_tasks().expect("refresh");
        app.selected = 1;

        app.start_new_task();
        let editor = app.editor.as_mut().expect("editor");
        assert_eq!(editor.insert_after, 1);
        assert!(editor.task_id.is_none());
        assert_eq!(app.tasks[1].title, "Second");

        editor.title = "Inserted".to_string();
        app.save_edit().expect("save");

        assert_eq!(
            app.tasks
                .iter()
                .map(|task| task.title.as_str())
                .collect::<Vec<_>>(),
            vec!["First", "Second", "Inserted", "Third"]
        );
        assert_eq!(app.selected, 2);
    }
}

fn parse_key_binding(value: &str) -> Option<KeyBinding> {
    let mut ctrl = false;
    let mut shift = false;
    let mut key: Option<char> = None;
    for part in value.split('+') {
        let token = part.trim();
        let lowered = token.to_lowercase();
        if lowered == "ctrl" || lowered == "control" {
            ctrl = true;
            continue;
        }
        if lowered == "shift" {
            shift = true;
            continue;
        }
        if token.chars().count() == 1 {
            let ch = token.chars().next()?;
            if ch.is_ascii_uppercase() {
                shift = true;
            }
            key = Some(ch.to_ascii_lowercase());
        }
    }
    key.map(|key| KeyBinding { ctrl, shift, key })
}
