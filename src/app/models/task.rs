use serde::{Deserialize, Serialize};
use time::OffsetDateTime;
use uuid::Uuid;
use validator::Validate;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Task {
    pub id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub project_id: Option<Uuid>,
    pub user_id: Uuid,
    pub is_active: bool,
    pub created_at: Option<OffsetDateTime>,
    pub updated_at: Option<OffsetDateTime>,
}

#[derive(Debug, Serialize, Deserialize, Validate)]
pub struct CreateTaskRequest {
    #[validate(length(min = 1, max = 255, message = "Task name must be between 1 and 255 characters"))]
    pub name: String,
    #[validate(length(max = 1000, message = "Description cannot exceed 1000 characters"))]
    pub description: Option<String>,
    pub project_id: Option<Uuid>,
}

#[derive(Debug, Serialize, Deserialize, Validate)]
pub struct UpdateTaskRequest {
    #[validate(length(min = 1, max = 255, message = "Task name must be between 1 and 255 characters"))]
    pub name: Option<String>,
    #[validate(length(max = 1000, message = "Description cannot exceed 1000 characters"))]
    pub description: Option<String>,
    pub project_id: Option<Uuid>,
    pub is_active: Option<bool>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TaskResponse {
    pub id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub project_id: Option<Uuid>,
    pub is_active: bool,
    pub created_at: Option<OffsetDateTime>,
    pub updated_at: Option<OffsetDateTime>,
}

impl From<Task> for TaskResponse {
    fn from(task: Task) -> Self {
        Self {
            id: task.id,
            name: task.name,
            description: task.description,
            project_id: task.project_id,
            is_active: task.is_active,
            created_at: task.created_at,
            updated_at: task.updated_at,
        }
    }
}