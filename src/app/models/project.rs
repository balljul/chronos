use serde::{Deserialize, Serialize};
use time::OffsetDateTime;
use uuid::Uuid;
use validator::Validate;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Project {
    pub id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub color: Option<String>,
    pub user_id: Uuid,
    pub is_active: bool,
    pub created_at: Option<OffsetDateTime>,
    pub updated_at: Option<OffsetDateTime>,
}

#[derive(Debug, Serialize, Deserialize, Validate)]
pub struct CreateProjectRequest {
    #[validate(length(min = 1, max = 255, message = "Project name must be between 1 and 255 characters"))]
    pub name: String,
    #[validate(length(max = 1000, message = "Description cannot exceed 1000 characters"))]
    pub description: Option<String>,
    #[validate(regex(path = "*crate::app::models::project::COLOR_REGEX", message = "Color must be a valid hex code"))]
    pub color: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Validate)]
pub struct UpdateProjectRequest {
    #[validate(length(min = 1, max = 255, message = "Project name must be between 1 and 255 characters"))]
    pub name: Option<String>,
    #[validate(length(max = 1000, message = "Description cannot exceed 1000 characters"))]
    pub description: Option<String>,
    #[validate(regex(path = "*crate::app::models::project::COLOR_REGEX", message = "Color must be a valid hex code"))]
    pub color: Option<String>,
    pub is_active: Option<bool>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ProjectResponse {
    pub id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub color: Option<String>,
    pub is_active: bool,
    pub created_at: Option<OffsetDateTime>,
    pub updated_at: Option<OffsetDateTime>,
}

impl From<Project> for ProjectResponse {
    fn from(project: Project) -> Self {
        Self {
            id: project.id,
            name: project.name,
            description: project.description,
            color: project.color,
            is_active: project.is_active,
            created_at: project.created_at,
            updated_at: project.updated_at,
        }
    }
}

lazy_static::lazy_static! {
    pub static ref COLOR_REGEX: regex::Regex = regex::Regex::new(r"^#[0-9A-Fa-f]{6}$").unwrap();
}