use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Action {
    CursorDown,
    CursorUp,
    GotoTop,
    GotoBottom,
    GotoToday,
    GotoInbox,
    GotoProject { project_id: Uuid },
    NewTask,
    EditTask,
    DeleteTask,
    ToggleComplete,
    MoveToToday,
    MoveToSomeday,
    AddTag,
    RemoveTag,
    Search,
    Help,
    Quit,
}
