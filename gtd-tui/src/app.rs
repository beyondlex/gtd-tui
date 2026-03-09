use anyhow::{anyhow, Result};
use chrono::Utc;
use crossterm::event::KeyCode;
use gtd_core::models::{Task, TaskStatus};
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
    Input,
}

pub struct App {
    pub should_quit: bool,
    pub view: View,
    pub mode: Mode,
    pub input: String,
    pub tasks: Vec<Task>,
    pub selected: usize,
    storage: SqliteStorage,
}

impl App {
    pub fn new(storage: SqliteStorage) -> Result<Self> {
        let mut app = Self {
            should_quit: false,
            view: View::Inbox,
            mode: Mode::Normal,
            input: String::new(),
            tasks: Vec::new(),
            selected: 0,
            storage,
        };
        app.refresh_tasks()?;
        Ok(app)
    }

    pub fn on_key(&mut self, code: KeyCode) -> Result<()> {
        match self.mode {
            Mode::Normal => self.on_key_normal(code),
            Mode::Input => self.on_key_input(code),
        }
    }

    fn on_key_normal(&mut self, code: KeyCode) -> Result<()> {
        match code {
            KeyCode::Char('q') | KeyCode::Esc => self.should_quit = true,
            KeyCode::Char('1') | KeyCode::Char('i') => self.view = View::Inbox,
            KeyCode::Char('2') | KeyCode::Char('t') => self.view = View::Today,
            KeyCode::Char('3') | KeyCode::Char('u') => self.view = View::Upcoming,
            KeyCode::Char('4') | KeyCode::Char('a') => self.view = View::Anytime,
            KeyCode::Char('5') | KeyCode::Char('s') => self.view = View::Someday,
            KeyCode::Char('j') | KeyCode::Down => self.select_next(),
            KeyCode::Char('k') | KeyCode::Up => self.select_prev(),
            KeyCode::Char('n') => self.enter_input_mode(),
            KeyCode::Char('x') => self.toggle_selected_task()?,
            KeyCode::Char('r') => self.refresh_tasks()?,
            _ => {}
        }
        Ok(())
    }

    fn on_key_input(&mut self, code: KeyCode) -> Result<()> {
        match code {
            KeyCode::Esc => self.exit_input_mode(),
            KeyCode::Enter => self.commit_input()?,
            KeyCode::Backspace => {
                self.input.pop();
            }
            KeyCode::Char(ch) => {
                self.input.push(ch);
            }
            _ => {}
        }
        Ok(())
    }

    fn enter_input_mode(&mut self) {
        self.mode = Mode::Input;
        self.input.clear();
    }

    fn exit_input_mode(&mut self) {
        self.mode = Mode::Normal;
        self.input.clear();
    }

    fn commit_input(&mut self) -> Result<()> {
        let title = self.input.trim();
        if !title.is_empty() {
            let now = Utc::now();
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
                sort_order: self.tasks.len() as i32,
                created_at: now,
                updated_at: now,
            };
            self.storage.create_task(&task).map_err(|e| anyhow!(e))?;
            self.refresh_tasks()?;
        }
        self.exit_input_mode();
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
