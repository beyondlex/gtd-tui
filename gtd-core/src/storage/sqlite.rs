use std::error::Error;
use std::path::Path;
use std::sync::Mutex;

use chrono::{DateTime, NaiveDate, Utc};
use rusqlite::types::{Type, Value};
use rusqlite::{params, params_from_iter, Connection};
use uuid::Uuid;

use crate::models::{
    Area, ChecklistItem, HotkeyConfig, Project, ProjectStatus, Tag, Task, TaskStatus,
};
use crate::storage::{Storage, StorageResult, TaskFilter};

const SCHEMA: &str = r#"
CREATE TABLE IF NOT EXISTS areas (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    color TEXT,
    sort_order INTEGER DEFAULT 0,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS projects (
    id TEXT PRIMARY KEY,
    area_id TEXT REFERENCES areas(id),
    title TEXT NOT NULL,
    notes TEXT,
    status TEXT DEFAULT 'active',
    start_date TEXT,
    due_date TEXT,
    sort_order INTEGER DEFAULT 0,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS headings (
    id TEXT PRIMARY KEY,
    project_id TEXT NOT NULL REFERENCES projects(id),
    title TEXT NOT NULL,
    sort_order INTEGER DEFAULT 0
);

CREATE TABLE IF NOT EXISTS tasks (
    id TEXT PRIMARY KEY,
    project_id TEXT REFERENCES projects(id),
    heading_id TEXT REFERENCES headings(id),
    area_id TEXT REFERENCES areas(id),
    title TEXT NOT NULL,
    notes TEXT,
    status TEXT DEFAULT 'pending',
    start_date TEXT,
    due_date TEXT,
    is_today INTEGER DEFAULT 0,
    is_someday INTEGER DEFAULT 0,
    sort_order INTEGER DEFAULT 0,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS tags (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL UNIQUE,
    color TEXT
);

CREATE TABLE IF NOT EXISTS task_tags (
    task_id TEXT NOT NULL REFERENCES tasks(id),
    tag_id TEXT NOT NULL REFERENCES tags(id),
    PRIMARY KEY (task_id, tag_id)
);

CREATE TABLE IF NOT EXISTS checklist_items (
    id TEXT PRIMARY KEY,
    task_id TEXT NOT NULL REFERENCES tasks(id),
    title TEXT NOT NULL,
    is_checked INTEGER DEFAULT 0,
    sort_order INTEGER DEFAULT 0
);

CREATE TABLE IF NOT EXISTS recurrence_rules (
    id TEXT PRIMARY KEY,
    task_id TEXT NOT NULL REFERENCES tasks(id),
    frequency TEXT NOT NULL,
    interval INTEGER DEFAULT 1,
    days_of_week TEXT,
    day_of_month INTEGER,
    end_date TEXT
);

CREATE TABLE IF NOT EXISTS hotkey_config (
    id TEXT PRIMARY KEY,
    action TEXT NOT NULL UNIQUE,
    key TEXT NOT NULL,
    modifiers TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS settings (
    key TEXT PRIMARY KEY,
    value TEXT NOT NULL
);
"#;

pub struct SqliteStorage {
    conn: Mutex<Connection>,
}

impl SqliteStorage {
    pub fn new(path: impl AsRef<Path>) -> StorageResult<Self> {
        let conn = Connection::open(path)?;
        let storage = Self {
            conn: Mutex::new(conn),
        };
        storage.init()?;
        Ok(storage)
    }

    fn init(&self) -> StorageResult<()> {
        let conn = self
            .conn
            .lock()
            .map_err(|_| "sqlite connection lock poisoned")?;
        conn.execute_batch(SCHEMA)?;
        Ok(())
    }
}

impl Storage for SqliteStorage {
    fn get_areas(&self) -> StorageResult<Vec<Area>> {
        let conn = self
            .conn
            .lock()
            .map_err(|_| "sqlite connection lock poisoned")?;
        let mut stmt = conn.prepare(
            "SELECT id, name, color, sort_order, created_at, updated_at FROM areas ORDER BY sort_order, name",
        )?;
        let rows = stmt.query_map([], |row| {
            let id: String = row.get(0)?;
            let created_at: String = row.get(4)?;
            let updated_at: String = row.get(5)?;

            Ok(Area {
                id: parse_uuid(&id).map_err(|err| map_parse_err(0, err))?,
                name: row.get(1)?,
                color: row.get(2)?,
                sort_order: row.get(3)?,
                created_at: parse_datetime(&created_at).map_err(|err| map_parse_err(4, err))?,
                updated_at: parse_datetime(&updated_at).map_err(|err| map_parse_err(5, err))?,
            })
        })?;

        collect_rows(rows)
    }

    fn create_area(&self, area: &Area) -> StorageResult<()> {
        let conn = self
            .conn
            .lock()
            .map_err(|_| "sqlite connection lock poisoned")?;
        conn.execute(
            "INSERT INTO areas (id, name, color, sort_order, created_at, updated_at) VALUES (?, ?, ?, ?, ?, ?)",
            params![
                area.id.to_string(),
                area.name,
                area.color,
                area.sort_order,
                area.created_at.to_rfc3339(),
                area.updated_at.to_rfc3339(),
            ],
        )?;
        Ok(())
    }

    fn update_area(&self, area: &Area) -> StorageResult<()> {
        let conn = self
            .conn
            .lock()
            .map_err(|_| "sqlite connection lock poisoned")?;
        conn.execute(
            "UPDATE areas SET name = ?, color = ?, sort_order = ?, updated_at = ? WHERE id = ?",
            params![
                area.name,
                area.color,
                area.sort_order,
                area.updated_at.to_rfc3339(),
                area.id.to_string(),
            ],
        )?;
        Ok(())
    }

    fn delete_area(&self, id: Uuid) -> StorageResult<()> {
        let conn = self
            .conn
            .lock()
            .map_err(|_| "sqlite connection lock poisoned")?;
        conn.execute("DELETE FROM areas WHERE id = ?", params![id.to_string()])?;
        Ok(())
    }

    fn get_projects(&self, area_id: Option<Uuid>) -> StorageResult<Vec<Project>> {
        let conn = self
            .conn
            .lock()
            .map_err(|_| "sqlite connection lock poisoned")?;
        let (sql, params) = if let Some(area_id) = area_id {
            (
                "SELECT id, area_id, title, notes, status, start_date, due_date, sort_order, created_at, updated_at FROM projects WHERE area_id = ? ORDER BY sort_order, title",
                vec![Value::from(area_id.to_string())],
            )
        } else {
            (
                "SELECT id, area_id, title, notes, status, start_date, due_date, sort_order, created_at, updated_at FROM projects ORDER BY sort_order, title",
                Vec::new(),
            )
        };

        let mut stmt = conn.prepare(sql)?;
        let rows = stmt.query_map(params_from_iter(params), |row| {
            let id: String = row.get(0)?;
            let created_at: String = row.get(8)?;
            let updated_at: String = row.get(9)?;
            let status: String = row.get(4)?;
            let start_date: Option<String> = row.get(5)?;
            let due_date: Option<String> = row.get(6)?;

            Ok(Project {
                id: parse_uuid(&id).map_err(|err| map_parse_err(0, err))?,
                area_id: parse_uuid_opt(row.get(1)?).map_err(|err| map_parse_err(1, err))?,
                title: row.get(2)?,
                notes: row.get(3)?,
                status: parse_project_status(&status).map_err(|err| map_parse_err(4, err))?,
                start_date: parse_date_opt(start_date).map_err(|err| map_parse_err(5, err))?,
                due_date: parse_date_opt(due_date).map_err(|err| map_parse_err(6, err))?,
                sort_order: row.get(7)?,
                created_at: parse_datetime(&created_at).map_err(|err| map_parse_err(8, err))?,
                updated_at: parse_datetime(&updated_at).map_err(|err| map_parse_err(9, err))?,
            })
        })?;

        collect_rows(rows)
    }

    fn create_project(&self, project: &Project) -> StorageResult<()> {
        let conn = self
            .conn
            .lock()
            .map_err(|_| "sqlite connection lock poisoned")?;
        conn.execute(
            "INSERT INTO projects (id, area_id, title, notes, status, start_date, due_date, sort_order, created_at, updated_at) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
            params![
                project.id.to_string(),
                project.area_id.map(|id| id.to_string()),
                project.title,
                project.notes,
                project_status_to_str(&project.status),
                project.start_date.map(|d| d.to_string()),
                project.due_date.map(|d| d.to_string()),
                project.sort_order,
                project.created_at.to_rfc3339(),
                project.updated_at.to_rfc3339(),
            ],
        )?;
        Ok(())
    }

    fn update_project(&self, project: &Project) -> StorageResult<()> {
        let conn = self
            .conn
            .lock()
            .map_err(|_| "sqlite connection lock poisoned")?;
        conn.execute(
            "UPDATE projects SET area_id = ?, title = ?, notes = ?, status = ?, start_date = ?, due_date = ?, sort_order = ?, updated_at = ? WHERE id = ?",
            params![
                project.area_id.map(|id| id.to_string()),
                project.title,
                project.notes,
                project_status_to_str(&project.status),
                project.start_date.map(|d| d.to_string()),
                project.due_date.map(|d| d.to_string()),
                project.sort_order,
                project.updated_at.to_rfc3339(),
                project.id.to_string(),
            ],
        )?;
        Ok(())
    }

    fn delete_project(&self, id: Uuid) -> StorageResult<()> {
        let conn = self
            .conn
            .lock()
            .map_err(|_| "sqlite connection lock poisoned")?;
        conn.execute("DELETE FROM projects WHERE id = ?", params![id.to_string()])?;
        Ok(())
    }

    fn get_tasks(&self, filter: TaskFilter) -> StorageResult<Vec<Task>> {
        let conn = self
            .conn
            .lock()
            .map_err(|_| "sqlite connection lock poisoned")?;
        let mut sql = String::from(
            "SELECT id, project_id, heading_id, area_id, title, notes, status, start_date, due_date, is_today, is_someday, sort_order, created_at, updated_at FROM tasks",
        );
        let mut clauses = Vec::new();
        let mut params: Vec<Value> = Vec::new();

        if let Some(project_id) = filter.project_id {
            clauses.push("project_id = ?");
            params.push(Value::from(project_id.to_string()));
        }
        if let Some(area_id) = filter.area_id {
            clauses.push("area_id = ?");
            params.push(Value::from(area_id.to_string()));
        }
        if let Some(is_today) = filter.is_today {
            clauses.push("is_today = ?");
            params.push(Value::from(bool_to_i32(is_today)));
        }
        if let Some(is_someday) = filter.is_someday {
            clauses.push("is_someday = ?");
            params.push(Value::from(bool_to_i32(is_someday)));
        }
        if let Some(status) = filter.status {
            clauses.push("status = ?");
            params.push(Value::from(task_status_to_str(&status).to_string()));
        }

        if !clauses.is_empty() {
            sql.push_str(" WHERE ");
            sql.push_str(&clauses.join(" AND "));
        }
        sql.push_str(" ORDER BY sort_order, created_at");

        let mut stmt = conn.prepare(&sql)?;
        let rows = stmt.query_map(params_from_iter(params), |row| {
            let id: String = row.get(0)?;
            let status: String = row.get(6)?;
            let start_date: Option<String> = row.get(7)?;
            let due_date: Option<String> = row.get(8)?;
            let created_at: String = row.get(12)?;
            let updated_at: String = row.get(13)?;

            Ok(Task {
                id: parse_uuid(&id).map_err(|err| map_parse_err(0, err))?,
                project_id: parse_uuid_opt(row.get(1)?).map_err(|err| map_parse_err(1, err))?,
                heading_id: parse_uuid_opt(row.get(2)?).map_err(|err| map_parse_err(2, err))?,
                area_id: parse_uuid_opt(row.get(3)?).map_err(|err| map_parse_err(3, err))?,
                title: row.get(4)?,
                notes: row.get(5)?,
                status: parse_task_status(&status).map_err(|err| map_parse_err(6, err))?,
                start_date: parse_date_opt(start_date).map_err(|err| map_parse_err(7, err))?,
                due_date: parse_date_opt(due_date).map_err(|err| map_parse_err(8, err))?,
                is_today: i32_to_bool(row.get(9)?),
                is_someday: i32_to_bool(row.get(10)?),
                sort_order: row.get(11)?,
                created_at: parse_datetime(&created_at).map_err(|err| map_parse_err(12, err))?,
                updated_at: parse_datetime(&updated_at).map_err(|err| map_parse_err(13, err))?,
            })
        })?;

        collect_rows(rows)
    }

    fn create_task(&self, task: &Task) -> StorageResult<()> {
        let conn = self
            .conn
            .lock()
            .map_err(|_| "sqlite connection lock poisoned")?;
        conn.execute(
            "INSERT INTO tasks (id, project_id, heading_id, area_id, title, notes, status, start_date, due_date, is_today, is_someday, sort_order, created_at, updated_at) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
            params![
                task.id.to_string(),
                task.project_id.map(|id| id.to_string()),
                task.heading_id.map(|id| id.to_string()),
                task.area_id.map(|id| id.to_string()),
                task.title,
                task.notes,
                task_status_to_str(&task.status),
                task.start_date.map(|d| d.to_string()),
                task.due_date.map(|d| d.to_string()),
                bool_to_i32(task.is_today),
                bool_to_i32(task.is_someday),
                task.sort_order,
                task.created_at.to_rfc3339(),
                task.updated_at.to_rfc3339(),
            ],
        )?;
        Ok(())
    }

    fn update_task(&self, task: &Task) -> StorageResult<()> {
        let conn = self
            .conn
            .lock()
            .map_err(|_| "sqlite connection lock poisoned")?;
        conn.execute(
            "UPDATE tasks SET project_id = ?, heading_id = ?, area_id = ?, title = ?, notes = ?, status = ?, start_date = ?, due_date = ?, is_today = ?, is_someday = ?, sort_order = ?, updated_at = ? WHERE id = ?",
            params![
                task.project_id.map(|id| id.to_string()),
                task.heading_id.map(|id| id.to_string()),
                task.area_id.map(|id| id.to_string()),
                task.title,
                task.notes,
                task_status_to_str(&task.status),
                task.start_date.map(|d| d.to_string()),
                task.due_date.map(|d| d.to_string()),
                bool_to_i32(task.is_today),
                bool_to_i32(task.is_someday),
                task.sort_order,
                task.updated_at.to_rfc3339(),
                task.id.to_string(),
            ],
        )?;
        Ok(())
    }

    fn delete_task(&self, id: Uuid) -> StorageResult<()> {
        let conn = self
            .conn
            .lock()
            .map_err(|_| "sqlite connection lock poisoned")?;
        conn.execute(
            "DELETE FROM checklist_items WHERE task_id = ?",
            params![id.to_string()],
        )?;
        conn.execute("DELETE FROM tasks WHERE id = ?", params![id.to_string()])?;
        Ok(())
    }

    fn get_tags(&self) -> StorageResult<Vec<Tag>> {
        let conn = self
            .conn
            .lock()
            .map_err(|_| "sqlite connection lock poisoned")?;
        let mut stmt = conn.prepare("SELECT id, name, color FROM tags ORDER BY name")?;
        let rows = stmt.query_map([], |row| {
            let id: String = row.get(0)?;
            Ok(Tag {
                id: parse_uuid(&id).map_err(|err| map_parse_err(0, err))?,
                name: row.get(1)?,
                color: row.get(2)?,
            })
        })?;

        collect_rows(rows)
    }

    fn create_tag(&self, tag: &Tag) -> StorageResult<()> {
        let conn = self
            .conn
            .lock()
            .map_err(|_| "sqlite connection lock poisoned")?;
        conn.execute(
            "INSERT INTO tags (id, name, color) VALUES (?, ?, ?)",
            params![tag.id.to_string(), tag.name, tag.color],
        )?;
        Ok(())
    }

    fn delete_tag(&self, id: Uuid) -> StorageResult<()> {
        let conn = self
            .conn
            .lock()
            .map_err(|_| "sqlite connection lock poisoned")?;
        conn.execute("DELETE FROM tags WHERE id = ?", params![id.to_string()])?;
        Ok(())
    }

    fn get_checklist(&self, task_id: Uuid) -> StorageResult<Vec<ChecklistItem>> {
        let conn = self
            .conn
            .lock()
            .map_err(|_| "sqlite connection lock poisoned")?;
        let mut stmt = conn.prepare(
            "SELECT id, task_id, title, is_checked, sort_order FROM checklist_items WHERE task_id = ? ORDER BY sort_order",
        )?;
        let rows = stmt.query_map(params![task_id.to_string()], |row| {
            let id: String = row.get(0)?;
            let task_id: String = row.get(1)?;

            Ok(ChecklistItem {
                id: parse_uuid(&id).map_err(|err| map_parse_err(0, err))?,
                task_id: parse_uuid(&task_id).map_err(|err| map_parse_err(1, err))?,
                title: row.get(2)?,
                is_checked: i32_to_bool(row.get(3)?),
                sort_order: row.get(4)?,
            })
        })?;

        collect_rows(rows)
    }

    fn create_checklist_item(&self, item: &ChecklistItem) -> StorageResult<()> {
        let conn = self
            .conn
            .lock()
            .map_err(|_| "sqlite connection lock poisoned")?;
        conn.execute(
            "INSERT INTO checklist_items (id, task_id, title, is_checked, sort_order) VALUES (?, ?, ?, ?, ?)",
            params![
                item.id.to_string(),
                item.task_id.to_string(),
                item.title,
                bool_to_i32(item.is_checked),
                item.sort_order,
            ],
        )?;
        Ok(())
    }

    fn update_checklist_item(&self, item: &ChecklistItem) -> StorageResult<()> {
        let conn = self
            .conn
            .lock()
            .map_err(|_| "sqlite connection lock poisoned")?;
        conn.execute(
            "UPDATE checklist_items SET title = ?, is_checked = ?, sort_order = ? WHERE id = ?",
            params![
                item.title,
                bool_to_i32(item.is_checked),
                item.sort_order,
                item.id.to_string(),
            ],
        )?;
        Ok(())
    }

    fn delete_checklist_item(&self, id: Uuid) -> StorageResult<()> {
        let conn = self
            .conn
            .lock()
            .map_err(|_| "sqlite connection lock poisoned")?;
        conn.execute(
            "DELETE FROM checklist_items WHERE id = ?",
            params![id.to_string()],
        )?;
        Ok(())
    }

    fn delete_checklist_for_task(&self, task_id: Uuid) -> StorageResult<()> {
        let conn = self
            .conn
            .lock()
            .map_err(|_| "sqlite connection lock poisoned")?;
        conn.execute(
            "DELETE FROM checklist_items WHERE task_id = ?",
            params![task_id.to_string()],
        )?;
        Ok(())
    }

    fn get_hotkeys(&self) -> StorageResult<Vec<HotkeyConfig>> {
        let conn = self
            .conn
            .lock()
            .map_err(|_| "sqlite connection lock poisoned")?;
        let mut stmt =
            conn.prepare("SELECT id, action, key, modifiers FROM hotkey_config ORDER BY action")?;
        let rows = stmt.query_map([], |row| {
            let id: String = row.get(0)?;
            let modifiers: String = row.get(3)?;
            let modifiers: Vec<String> =
                serde_json::from_str(&modifiers).map_err(|err| map_parse_err(3, Box::new(err)))?;

            Ok(HotkeyConfig {
                id: parse_uuid(&id).map_err(|err| map_parse_err(0, err))?,
                action: row.get(1)?,
                key: row.get(2)?,
                modifiers,
            })
        })?;

        collect_rows(rows)
    }

    fn save_hotkey(&self, config: &HotkeyConfig) -> StorageResult<()> {
        let conn = self
            .conn
            .lock()
            .map_err(|_| "sqlite connection lock poisoned")?;
        let modifiers = serde_json::to_string(&config.modifiers)?;
        conn.execute(
            "INSERT INTO hotkey_config (id, action, key, modifiers) VALUES (?, ?, ?, ?)
             ON CONFLICT(action) DO UPDATE SET key = excluded.key, modifiers = excluded.modifiers",
            params![config.id.to_string(), config.action, config.key, modifiers,],
        )?;
        Ok(())
    }
}

fn project_status_to_str(status: &ProjectStatus) -> &'static str {
    match status {
        ProjectStatus::Active => "active",
        ProjectStatus::Completed => "completed",
        ProjectStatus::Dropped => "dropped",
    }
}

fn parse_project_status(value: &str) -> StorageResult<ProjectStatus> {
    match value {
        "active" => Ok(ProjectStatus::Active),
        "completed" => Ok(ProjectStatus::Completed),
        "dropped" => Ok(ProjectStatus::Dropped),
        _ => Err(format!("unknown project status: {value}").into()),
    }
}

fn task_status_to_str(status: &TaskStatus) -> &'static str {
    match status {
        TaskStatus::Pending => "pending",
        TaskStatus::Completed => "completed",
        TaskStatus::Cancelled => "cancelled",
    }
}

fn parse_task_status(value: &str) -> StorageResult<TaskStatus> {
    match value {
        "pending" => Ok(TaskStatus::Pending),
        "completed" => Ok(TaskStatus::Completed),
        "cancelled" => Ok(TaskStatus::Cancelled),
        _ => Err(format!("unknown task status: {value}").into()),
    }
}

fn bool_to_i32(value: bool) -> i32 {
    if value {
        1
    } else {
        0
    }
}

fn i32_to_bool(value: i32) -> bool {
    value != 0
}

fn parse_uuid(value: &str) -> StorageResult<Uuid> {
    Ok(Uuid::parse_str(value)?)
}

fn parse_uuid_opt(value: Option<String>) -> StorageResult<Option<Uuid>> {
    match value {
        Some(value) => Ok(Some(Uuid::parse_str(&value)?)),
        None => Ok(None),
    }
}

fn parse_datetime(value: &str) -> StorageResult<DateTime<Utc>> {
    let parsed = DateTime::parse_from_rfc3339(value)?;
    Ok(parsed.with_timezone(&Utc))
}

fn parse_date_opt(value: Option<String>) -> StorageResult<Option<NaiveDate>> {
    match value {
        Some(value) => Ok(Some(NaiveDate::parse_from_str(&value, "%Y-%m-%d")?)),
        None => Ok(None),
    }
}

fn map_parse_err(column: usize, err: Box<dyn Error + Send + Sync>) -> rusqlite::Error {
    rusqlite::Error::FromSqlConversionFailure(column, Type::Text, err)
}

fn collect_rows<T>(rows: impl Iterator<Item = rusqlite::Result<T>>) -> StorageResult<Vec<T>> {
    let mut out = Vec::new();
    for row in rows {
        out.push(row?);
    }
    Ok(out)
}
