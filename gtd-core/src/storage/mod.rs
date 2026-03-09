use uuid::Uuid;

use crate::models::{
    Area, ChecklistItem, HotkeyConfig, Project, Tag, Task, TaskStatus,
};

pub type StorageResult<T> = Result<T, Box<dyn std::error::Error + Send + Sync>>;

#[derive(Debug, Clone, Default)]
pub struct TaskFilter {
    pub project_id: Option<Uuid>,
    pub area_id: Option<Uuid>,
    pub is_today: Option<bool>,
    pub is_someday: Option<bool>,
    pub status: Option<TaskStatus>,
}

pub trait Storage: Send + Sync {
    fn get_areas(&self) -> StorageResult<Vec<Area>>;
    fn create_area(&self, area: &Area) -> StorageResult<()>;
    fn update_area(&self, area: &Area) -> StorageResult<()>;
    fn delete_area(&self, id: Uuid) -> StorageResult<()>;

    fn get_projects(&self, area_id: Option<Uuid>) -> StorageResult<Vec<Project>>;
    fn create_project(&self, project: &Project) -> StorageResult<()>;
    fn update_project(&self, project: &Project) -> StorageResult<()>;
    fn delete_project(&self, id: Uuid) -> StorageResult<()>;

    fn get_tasks(&self, filter: TaskFilter) -> StorageResult<Vec<Task>>;
    fn create_task(&self, task: &Task) -> StorageResult<()>;
    fn update_task(&self, task: &Task) -> StorageResult<()>;
    fn delete_task(&self, id: Uuid) -> StorageResult<()>;

    fn get_tags(&self) -> StorageResult<Vec<Tag>>;
    fn create_tag(&self, tag: &Tag) -> StorageResult<()>;
    fn delete_tag(&self, id: Uuid) -> StorageResult<()>;

    fn get_checklist(&self, task_id: Uuid) -> StorageResult<Vec<ChecklistItem>>;
    fn create_checklist_item(&self, item: &ChecklistItem) -> StorageResult<()>;
    fn update_checklist_item(&self, item: &ChecklistItem) -> StorageResult<()>;
    fn delete_checklist_item(&self, id: Uuid) -> StorageResult<()>;

    fn get_hotkeys(&self) -> StorageResult<Vec<HotkeyConfig>>;
    fn save_hotkey(&self, config: &HotkeyConfig) -> StorageResult<()>;
}
