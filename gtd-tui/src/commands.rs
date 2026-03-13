use anyhow::{anyhow, Result};
use chrono::Utc;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use gtd_core::storage::Storage;
use uuid::Uuid;

use super::editor::{ensure_checklist_not_empty, ChecklistDraft, DatePickerState, EditorState};
use super::state::{DeleteTarget, Focus, Layer, Mode, View};
use super::{get_keymap, App};

impl App {
    pub fn on_key(&mut self, key: KeyEvent) -> Result<()> {
        match self.mode {
            Mode::Normal => self.on_key_normal(key),
            Mode::Editing => self.on_key_edit(key),
            Mode::ConfirmDelete => self.on_key_confirm_delete(key),
        }
    }

    pub fn on_tick(&mut self) {
        let now = std::time::Instant::now();
        if now.duration_since(self.last_blink) >= std::time::Duration::from_millis(500) {
            self.cursor_visible = !self.cursor_visible;
            self.last_blink = now;
        }
    }

    pub fn on_key_normal(&mut self, key: KeyEvent) -> Result<()> {
        let keymap = get_keymap();

        if keymap.quit.matches(key)
            || (key.modifiers.contains(KeyModifiers::CONTROL) && key.code == KeyCode::Char('c'))
        {
            self.should_quit = true;
        } else if keymap.view_inbox.matches(key) || key.code == KeyCode::Char('i') {
            self.view = View::Inbox;
        } else if keymap.view_today.matches(key) || key.code == KeyCode::Char('t') {
            self.view = View::Today;
        } else if keymap.view_upcoming.matches(key) || key.code == KeyCode::Char('u') {
            self.view = View::Upcoming;
        } else if keymap.view_anytime.matches(key) {
            self.view = View::Anytime;
        } else if keymap.view_someday.matches(key) || key.code == KeyCode::Char('s') {
            self.view = View::Someday;
        } else if keymap.select_next.matches(key) || key.code == KeyCode::Down {
            self.select_next();
        } else if keymap.select_prev.matches(key) || key.code == KeyCode::Up {
            self.select_prev();
        } else if keymap.new_task.matches(key) {
            self.start_new_task();
        } else if keymap.edit_task.matches(key) && self.view == View::Inbox {
            self.start_edit_task()?;
        } else if keymap.toggle_task.matches(key) {
            self.toggle_selected_task()?;
        } else if keymap.refresh.matches(key) {
            self.refresh_tasks()?;
        } else if key.code == KeyCode::Char('d') && !self.tasks.is_empty() {
            self.mode = Mode::ConfirmDelete;
            self.delete_confirm = Some(DeleteTarget::Task);
        } else if keymap.new_item_below.matches(key) {
            self.start_new_task();
        } else if keymap.new_item_above.matches(key) {
            let insert_after = self.selected.saturating_sub(1);
            let insert_at_beginning = self.selected == 0;
            self.start_new_task_at(insert_after, insert_at_beginning);
        } else if keymap.move_item_down.matches(key) {
            self.move_task_down()?;
        } else if keymap.move_item_up.matches(key) {
            self.move_task_up()?;
        }

        Ok(())
    }

    pub fn on_key_edit(&mut self, key: KeyEvent) -> Result<()> {
        let keymap = get_keymap();

        if keymap.save_edit.matches(key) {
            return self.save_edit();
        }

        if key.code == KeyCode::Esc && self.editor.as_ref().map(|e| e.edit_active).unwrap_or(false)
        {
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

        if keymap.nav_up.matches(key) {
            if !editor.edit_active && editor.layer == Layer::ChecklistItem {
                editor.layer = Layer::TaskItem;
                editor.checklist_index = 0;
            } else if editor.layer == Layer::TaskItem && !editor.edit_active {
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

    pub fn handle_task_item_layer(&mut self, key: KeyEvent) {
        let keymap = get_keymap();

        let needs_auto_save = if let Some(editor) = self.editor.as_ref() {
            editor.edit_active
                && (keymap.next_focus.matches(key)
                    || keymap.prev_focus.matches(key)
                    || (keymap.date_edit_mode.matches(key) && editor.focus == Focus::Checklist)
                    || (keymap.checklist_edit_toggle.matches(key)
                        && editor.focus != Focus::DueDate))
                || (!editor.edit_active
                    && editor.focus == Focus::DueDate
                    && key.code == KeyCode::Down)
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

        if keymap.checklist_edit_toggle.matches(key) {
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

        if editor.edit_active && editor.focus == Focus::DueDate && key.code == KeyCode::Enter {
            editor.due_date = Some(editor.date_picker.cursor);
            let _ = self.auto_save_current_edit();
            self.handle_edit_mode(key);
            return;
        }

        if editor.edit_active {
            self.handle_edit_mode(key);
            return;
        }

        if keymap.next_focus.matches(key) {
            editor.focus = editor.focus.next();
            if editor.focus == Focus::Checklist {
                super::editor::ensure_checklist_not_empty(&mut editor.checklist);
                editor.checklist_index = 0;
            }
        } else if keymap.prev_focus.matches(key) {
            editor.focus = editor.focus.prev();
            if editor.focus == Focus::Checklist {
                super::editor::ensure_checklist_not_empty(&mut editor.checklist);
                editor.checklist_index = 0;
            }
        } else if keymap.date_edit_mode.matches(key) && editor.focus == Focus::Checklist {
            editor.layer = Layer::ChecklistItem;
            super::editor::ensure_checklist_not_empty(&mut editor.checklist);
            editor.checklist_index = 0;
        } else if keymap.date_edit_mode.matches(key) {
            editor.edit_active = true;
            if editor.focus == Focus::DueDate {
                if let Some(due) = editor.due_date {
                    editor.date_picker.cursor = due;
                }
            }
        } else if key.code == KeyCode::Down && editor.focus == Focus::DueDate {
            editor.edit_active = true;
            editor.focus = Focus::Checklist;
            super::editor::ensure_checklist_not_empty(&mut editor.checklist);
            editor.checklist_index = 0;
        } else if keymap.checklist_toggle.matches(key) && editor.focus == Focus::Checklist {
            if let Some(item) = editor.checklist.get_mut(editor.checklist_index) {
                item.checked = !item.checked;
            }
        } else if keymap.date_prev_day.matches(key) && editor.focus == Focus::DueDate {
            let base = editor.due_date.unwrap_or_else(|| Utc::now().date_naive());
            let next = base - chrono::Duration::days(1);
            editor.due_date = Some(next);
            editor.date_picker.cursor = next;
            let _ = self.auto_save_current_edit();
        } else if keymap.date_next_day.matches(key) && editor.focus == Focus::DueDate {
            let base = editor.due_date.unwrap_or_else(|| Utc::now().date_naive());
            let next = base + chrono::Duration::days(1);
            editor.due_date = Some(next);
            editor.date_picker.cursor = next;
            let _ = self.auto_save_current_edit();
        } else if keymap.date_today.matches(key)
            && editor.focus == Focus::DueDate
            && !editor.edit_active
        {
            let today = Utc::now().date_naive();
            editor.due_date = Some(today);
            editor.date_picker.cursor = today;
            let _ = self.auto_save_current_edit();
        } else if keymap.date_tomorrow.matches(key)
            && editor.focus == Focus::DueDate
            && !editor.edit_active
        {
            let tomorrow = Utc::now().date_naive() + chrono::Duration::days(1);
            editor.due_date = Some(tomorrow);
            editor.date_picker.cursor = tomorrow;
            let _ = self.auto_save_current_edit();
        } else if key.code == KeyCode::Char('o') && editor.focus == Focus::Checklist {
            let shift_pressed = key.modifiers.contains(KeyModifiers::SHIFT);
            super::editor::new_checklist_item(
                &mut editor.checklist,
                &mut editor.checklist_index,
                shift_pressed,
            );
            editor.layer = Layer::ChecklistItem;
            editor.edit_active = true;
        }
    }

    pub fn handle_checklist_item_layer(&mut self, key: KeyEvent) {
        let keymap = get_keymap();

        let editor = match self.editor.as_mut() {
            Some(editor) => editor,
            None => return,
        };

        if editor.edit_active {
            self.handle_checklist_edit_mode(&key);
            return;
        }

        if keymap.prev_focus.matches(key) {
            if editor.checklist_index > 0 {
                editor.checklist_index -= 1;
            }
        } else if keymap.next_focus.matches(key) {
            if !editor.checklist.is_empty() {
                editor.checklist_index =
                    (editor.checklist_index + 1).min(editor.checklist.len().saturating_sub(1));
            }
        } else if keymap.date_edit_mode.matches(key) {
            editor.edit_active = true;
        } else if keymap.nav_up.matches(key) && !editor.edit_active {
            editor.layer = Layer::TaskItem;
            editor.checklist_index = 0;
        } else if keymap.checklist_toggle.matches(key) {
            if let Some(item) = editor.checklist.get_mut(editor.checklist_index) {
                item.checked = !item.checked;
            }
        } else if keymap.new_item_below.matches(key) {
            editor
                .checklist
                .insert(editor.checklist_index + 1, ChecklistDraft::new());
            editor.checklist_index += 1;
            editor.edit_active = true;
        } else if keymap.new_item_above.matches(key) {
            editor
                .checklist
                .insert(editor.checklist_index, ChecklistDraft::new());
            editor.edit_active = true;
        } else if keymap.move_item_down.matches(key) {
            if editor.checklist_index < editor.checklist.len() - 1 {
                editor
                    .checklist
                    .swap(editor.checklist_index, editor.checklist_index + 1);
                editor.checklist_index += 1;
            }
        } else if keymap.move_item_up.matches(key) {
            if editor.checklist_index > 0 {
                editor
                    .checklist
                    .swap(editor.checklist_index, editor.checklist_index - 1);
                editor.checklist_index -= 1;
            }
        }
    }

    pub fn handle_edit_mode(&mut self, key: KeyEvent) {
        let keymap = get_keymap();

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
                if keymap.date_prev_day_in_edit_mode.matches(key) {
                    editor.date_picker.move_days(-1);
                    editor.due_date = Some(editor.date_picker.cursor);
                    let _ = self.auto_save_current_edit();
                } else if keymap.date_next_day_in_edit_mode.matches(key) {
                    editor.date_picker.move_days(1);
                    editor.due_date = Some(editor.date_picker.cursor);
                    let _ = self.auto_save_current_edit();
                } else if keymap.date_prev_week.matches(key) {
                    editor.date_picker.move_days(-7);
                    editor.due_date = Some(editor.date_picker.cursor);
                    let _ = self.auto_save_current_edit();
                } else if keymap.date_next_week.matches(key) {
                    editor.date_picker.move_days(7);
                    editor.due_date = Some(editor.date_picker.cursor);
                    let _ = self.auto_save_current_edit();
                } else if keymap.date_prev_month.matches(key) {
                    editor.date_picker.move_months(-1);
                    editor.due_date = Some(editor.date_picker.cursor);
                    let _ = self.auto_save_current_edit();
                } else if keymap.date_next_month.matches(key) {
                    editor.date_picker.move_months(1);
                    editor.due_date = Some(editor.date_picker.cursor);
                    let _ = self.auto_save_current_edit();
                } else if keymap.date_today.matches(key) {
                    let today = Utc::now().date_naive();
                    editor.date_picker.cursor = today;
                    editor.due_date = Some(today);
                    let _ = self.auto_save_current_edit();
                } else if keymap.date_tomorrow.matches(key) {
                    let tomorrow = Utc::now().date_naive() + chrono::Duration::days(1);
                    editor.date_picker.cursor = tomorrow;
                    editor.due_date = Some(tomorrow);
                    let _ = self.auto_save_current_edit();
                } else if key.code == KeyCode::Enter {
                    editor.due_date = Some(editor.date_picker.cursor);
                    editor.focus = Focus::Checklist;
                    editor.edit_active = false;
                    super::editor::ensure_checklist_not_empty(&mut editor.checklist);
                    editor.checklist_index = 0;
                } else if key.code == KeyCode::Down {
                    editor.edit_active = false;
                    editor.focus = Focus::Checklist;
                    super::editor::ensure_checklist_not_empty(&mut editor.checklist);
                    editor.checklist_index = 0;
                }
            }
            Focus::Checklist => {}
        }
    }

    pub fn handle_checklist_edit_mode(&mut self, key: &KeyEvent) {
        let editor = match self.editor.as_mut() {
            Some(editor) => editor,
            None => return,
        };

        super::editor::ensure_checklist_not_empty(&mut editor.checklist);

        if let KeyCode::Char(ch) = key.code {
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
            editor
                .checklist
                .insert(editor.checklist_index + 1, ChecklistDraft::new());
            editor.checklist_index += 1;
        }
    }

    pub fn on_key_confirm_delete(&mut self, key: KeyEvent) -> Result<()> {
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

    pub fn start_new_task(&mut self) {
        self.start_new_task_at(
            if self.tasks.is_empty() {
                0
            } else {
                self.selected
            },
            false,
        );
    }

    pub fn start_new_task_at(&mut self, insert_after: usize, insert_at_beginning: bool) {
        let today = Utc::now().date_naive();
        self.editor = Some(EditorState {
            task_id: None,
            insert_after,
            insert_at_beginning,
            title: String::new(),
            notes: String::new(),
            due_date: None,
            checklist: vec![ChecklistDraft::new()],
            checklist_index: 0,
            edit_active: false,
            focus: Focus::Title,
            layer: Layer::TaskItem,
            date_picker: DatePickerState::new(today),
        });
        self.mode = Mode::Editing;
    }

    pub fn start_edit_task(&mut self) -> Result<()> {
        if self.tasks.is_empty() {
            return Ok(());
        }
        let task = self.tasks[self.selected].clone();
        let checklist_items = self
            .storage
            .get_checklist(task.id)
            .map_err(|e| anyhow!(e))?;
        let checklist = if checklist_items.is_empty() {
            vec![ChecklistDraft::new()]
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

    pub fn cancel_edit(&mut self) {
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
            let task = gtd_core::models::Task {
                id: task_id,
                project_id: None,
                heading_id: None,
                area_id: None,
                title: title.to_string(),
                notes,
                status: gtd_core::models::TaskStatus::Pending,
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

    pub fn reindex_tasks_for_insert(
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

    pub fn replace_checklist(&self, task_id: Uuid, checklist: Vec<ChecklistDraft>) -> Result<()> {
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
            let item = gtd_core::models::ChecklistItem {
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

    pub fn auto_save_current_edit(&mut self) -> Result<()> {
        let (task_id, layer, focus, new_title, new_notes, due_date, checklist_items) =
            match self.editor.as_ref() {
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
            return Ok(());
        };

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
                if task.due_date != due_date {
                    task_clone.due_date = due_date;
                    needs_task_update = true;
                }
            }
            (Layer::ChecklistItem, _) => {
                if let Ok(existing_checklist) = self.storage.get_checklist(task_id) {
                    let has_changes = checklist_items.iter().enumerate().any(|(i, item)| {
                        existing_checklist.get(i).map(|e| e.title.as_str())
                            != Some(item.title.as_str())
                    });
                    if has_changes {
                        needs_checklist_update = true;
                    }
                }
            }
            _ => {}
        }

        if needs_task_update {
            task_clone.updated_at = now;
            self.storage
                .update_task(&task_clone)
                .map_err(|e| anyhow!(e))?;

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

    pub fn refresh_tasks(&mut self) -> Result<()> {
        self.tasks = self
            .storage
            .get_tasks(gtd_core::storage::TaskFilter::default())
            .map_err(|e| anyhow!(e))?;
        if self.tasks.is_empty() {
            self.selected = 0;
        } else if self.selected >= self.tasks.len() {
            self.selected = self.tasks.len() - 1;
        }
        Ok(())
    }

    pub fn toggle_selected_task(&mut self) -> Result<()> {
        if self.tasks.is_empty() {
            return Ok(());
        }
        let mut task = self.tasks[self.selected].clone();
        task.status = match task.status {
            gtd_core::models::TaskStatus::Pending => gtd_core::models::TaskStatus::Completed,
            gtd_core::models::TaskStatus::Completed => gtd_core::models::TaskStatus::Pending,
            gtd_core::models::TaskStatus::Cancelled => gtd_core::models::TaskStatus::Pending,
        };
        task.updated_at = Utc::now();
        self.storage.update_task(&task).map_err(|e| anyhow!(e))?;
        self.refresh_tasks()?;
        Ok(())
    }

    pub fn move_task_down(&mut self) -> Result<()> {
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

    pub fn move_task_up(&mut self) -> Result<()> {
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

    pub fn confirm_delete(&mut self) -> Result<()> {
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

    pub fn select_next(&mut self) {
        if self.tasks.is_empty() {
            self.selected = 0;
            return;
        }
        self.selected = (self.selected + 1).min(self.tasks.len() - 1);
    }

    pub fn select_prev(&mut self) {
        if self.tasks.is_empty() {
            self.selected = 0;
            return;
        }
        self.selected = self.selected.saturating_sub(1);
    }

    pub fn switch_task(&mut self, delta: i32) -> Result<()> {
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
