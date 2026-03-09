use gtd_core::models::{Area, ProjectStatus, TaskStatus};

#[test]
fn models_are_accessible() {
    let _ = Area {
        id: uuid::Uuid::new_v4(),
        name: "Inbox".to_string(),
        color: None,
        sort_order: 0,
        created_at: chrono::Utc::now(),
        updated_at: chrono::Utc::now(),
    };

    let _ = ProjectStatus::Active;
    let _ = TaskStatus::Pending;
}
