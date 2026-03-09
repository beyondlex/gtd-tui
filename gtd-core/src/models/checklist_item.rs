use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ChecklistItem {
    pub id: Uuid,
    pub task_id: Uuid,
    pub title: String,
    pub is_checked: bool,
    pub sort_order: i32,
}
