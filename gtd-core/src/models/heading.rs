use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Heading {
    pub id: Uuid,
    pub project_id: Uuid,
    pub title: String,
    pub sort_order: i32,
}
