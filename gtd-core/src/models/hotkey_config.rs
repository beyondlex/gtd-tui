use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct HotkeyConfig {
    pub id: Uuid,
    pub action: String,
    pub key: String,
    pub modifiers: Vec<String>,
}
