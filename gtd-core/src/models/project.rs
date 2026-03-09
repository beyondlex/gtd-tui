use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ProjectStatus {
    Active,
    Completed,
    Dropped,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Project {
    pub id: Uuid,
    pub area_id: Option<Uuid>,
    pub title: String,
    pub notes: Option<String>,
    pub status: ProjectStatus,
    pub start_date: Option<NaiveDate>,
    pub due_date: Option<NaiveDate>,
    pub sort_order: i32,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}
