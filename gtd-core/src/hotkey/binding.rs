use serde::{Deserialize, Serialize};

use super::Action;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct KeyModifiers {
    pub ctrl: bool,
    pub shift: bool,
    pub alt: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct HotkeyBinding {
    pub key: String,
    pub modifiers: KeyModifiers,
    pub action: Action,
}
