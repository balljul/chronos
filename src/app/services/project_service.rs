use crate::app::models::project::{Project, ProjectResponse, CreateProjectRequest, UpdateProjectRequest};
use crate::app::repositories::project_repository::ProjectRepository;
use uuid::Uuid;
use validator::Validate;

#[derive(Debug)]
pub enum ProjectServiceError {
    DatabaseError(sqlx::Error),
    ValidationError(validator::ValidationErrors),
    NotFound,
    Unauthorized,
}

impl std::fmt::Display for ProjectServiceError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            ProjectServiceError::DatabaseError(e) => write!(f, "Database error: {}", e),
            ProjectServiceError::ValidationError(e) => write!(f, "Validation error: {}", e),
            ProjectServiceError::NotFound => write!(f, "Project not found"),
            ProjectServiceError::Unauthorized => write!(f, "Unauthorized access to project"),
        }
    }
}

impl std::error::Error for ProjectServiceError {}

impl From<sqlx::Error> for ProjectServiceError {
    fn from(e: sqlx::Error) -> Self {
        ProjectServiceError::DatabaseError(e)
    }
}

impl From<validator::ValidationErrors> for ProjectServiceError {
    fn from(e: validator::ValidationErrors) -> Self {
        ProjectServiceError::ValidationError(e)
    }
}

#[derive(Clone)]
pub struct ProjectService {
    repository: ProjectRepository,
}

impl ProjectService {
    pub fn new(repository: ProjectRepository) -> Self {
        Self { repository }
    }

    pub async fn create_project(
        &self,
        project_data: CreateProjectRequest,
        user_id: Uuid,
    ) -> Result<ProjectResponse, ProjectServiceError> {
        project_data.validate()?;

        let project = self.repository.create(&project_data, user_id).await?;
        Ok(ProjectResponse::from(project))
    }

    pub async fn get_project_by_id(
        &self,
        project_id: Uuid,
        user_id: Uuid,
    ) -> Result<ProjectResponse, ProjectServiceError> {
        let project = self
            .repository
            .find_by_id(project_id, user_id)
            .await?
            .ok_or(ProjectServiceError::NotFound)?;

        Ok(ProjectResponse::from(project))
    }

    pub async fn get_projects_by_user(
        &self,
        user_id: Uuid,
        include_inactive: Option<bool>,
    ) -> Result<Vec<ProjectResponse>, ProjectServiceError> {
        let projects = self.repository.find_by_user(user_id, include_inactive).await?;
        Ok(projects.into_iter().map(ProjectResponse::from).collect())
    }

    pub async fn update_project(
        &self,
        project_id: Uuid,
        user_id: Uuid,
        project_data: UpdateProjectRequest,
    ) -> Result<ProjectResponse, ProjectServiceError> {
        project_data.validate()?;

        let project = self
            .repository
            .update(project_id, user_id, &project_data)
            .await?
            .ok_or(ProjectServiceError::NotFound)?;

        Ok(ProjectResponse::from(project))
    }

    pub async fn delete_project(
        &self,
        project_id: Uuid,
        user_id: Uuid,
        soft_delete: Option<bool>,
    ) -> Result<bool, ProjectServiceError> {
        let soft_delete = soft_delete.unwrap_or(true);

        let success = if soft_delete {
            self.repository.soft_delete(project_id, user_id).await?
        } else {
            self.repository.delete(project_id, user_id).await?
        };

        Ok(success)
    }

    pub async fn archive_project(
        &self,
        project_id: Uuid,
        user_id: Uuid,
    ) -> Result<ProjectResponse, ProjectServiceError> {
        let update_data = UpdateProjectRequest {
            name: None,
            description: None,
            color: None,
            is_active: Some(false),
        };

        self.update_project(project_id, user_id, update_data).await
    }

    pub async fn restore_project(
        &self,
        project_id: Uuid,
        user_id: Uuid,
    ) -> Result<ProjectResponse, ProjectServiceError> {
        let update_data = UpdateProjectRequest {
            name: None,
            description: None,
            color: None,
            is_active: Some(true),
        };

        self.update_project(project_id, user_id, update_data).await
    }
}