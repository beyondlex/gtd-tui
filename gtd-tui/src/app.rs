#[path = "commands.rs"]
mod commands;
#[path = "editor.rs"]
mod editor;
#[path = "keymap.rs"]
mod keymap;
#[path = "state.rs"]
mod state;

pub use editor::{ChecklistDraft, DatePickerState, EditorState};
pub use keymap::{KeyBinding, Keymap};
pub use state::{DeleteTarget, Focus, Layer, Mode, View};

use std::time::Instant;

use anyhow::Result;
use chrono::{Datelike, NaiveDate};
use gtd_core::models::Task;
use gtd_core::storage::SqliteStorage;

pub struct App {
    pub should_quit: bool,
    pub view: View,
    pub mode: Mode,
    pub tasks: Vec<Task>,
    pub selected: usize,
    pub editor: Option<EditorState>,
    pub calendar_theme: crate::ui::theme::CalendarTheme,
    pub editor_theme: crate::ui::theme::EditorTheme,
    pub keymap: Keymap,
    pub cursor_visible: bool,
    pub delete_confirm: Option<DeleteTarget>,
    pub pending_g: bool,
    last_blink: Instant,
    storage: SqliteStorage,
}

static KEYMAP: std::sync::OnceLock<Keymap> = std::sync::OnceLock::new();

pub fn init_keymap(keymap: Keymap) {
    let _ = KEYMAP.set(keymap);
}

pub fn get_keymap() -> &'static Keymap {
    KEYMAP.get().expect("keymap not initialized")
}

#[macro_export]
macro_rules! KEYMAP {
    () => {
        $crate::get_keymap()
    };
}

impl App {
    pub fn new(
        storage: SqliteStorage,
        calendar_theme: crate::ui::theme::CalendarTheme,
        editor_theme: crate::ui::theme::EditorTheme,
        keymap: Keymap,
    ) -> Result<Self> {
        init_keymap(keymap.clone());

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
            pending_g: false,
            last_blink: Instant::now(),
            storage,
        };
        app.refresh_tasks()?;
        Ok(app)
    }
}

pub use editor::days_in_month;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ui::theme::{CalendarTheme, EditorTheme};
    use gtd_core::storage::Storage;
    use uuid::Uuid;

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
        let binding = keymap::parse_key_binding("A").expect("binding");

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
        let now = chrono::Utc::now();
        for (sort_order, title) in ["First", "Second", "Third"].into_iter().enumerate() {
            let task = Task {
                id: Uuid::new_v4(),
                project_id: None,
                heading_id: None,
                area_id: None,
                title: title.to_string(),
                notes: None,
                status: gtd_core::models::TaskStatus::Pending,
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
