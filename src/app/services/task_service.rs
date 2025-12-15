use crate::app::models::task::{Task, TaskResponse, CreateTaskRequest, UpdateTaskRequest};
use crate::app::repositories::task_repository::TaskRepository;
use crate::app::repositories::project_repository::ProjectRepository;
use uuid::Uuid;
use validator::Validate;

#[derive(Debug)]
pub enum TaskServiceError {
    DatabaseError(sqlx::Error),
    ValidationError(validator::ValidationErrors),
    NotFound,
    Unauthorized,
    InvalidProject,
}

impl std::fmt::Display for TaskServiceError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            TaskServiceError::DatabaseError(e) => write!(f, "Database error: {}", e),
            TaskServiceError::ValidationError(e) => write!(f, "Validation error: {}", e),
            TaskServiceError::NotFound => write!(f, "Task not found"),
            TaskServiceError::Unauthorized => write!(f, "Unauthorized access to task"),
            TaskServiceError::InvalidProject => write!(f, "Project not found or unauthorized"),
        }
    }
}

impl std::error::Error for TaskServiceError {}

impl From<sqlx::Error> for TaskServiceError {
    fn from(e: sqlx::Error) -> Self {
        TaskServiceError::DatabaseError(e)
    }
}

impl From<validator::ValidationErrors> for TaskServiceError {
    fn from(e: validator::ValidationErrors) -> Self {
        TaskServiceError::ValidationError(e)
    }
}

#[derive(Clone)]
pub struct TaskService {
    task_repository: TaskRepository,
    project_repository: ProjectRepository,
}

impl TaskService {
    pub fn new(task_repository: TaskRepository, project_repository: ProjectRepository) -> Self {
        Self { task_repository, project_repository }
    }

    pub async fn create_task(
        &self,
        task_data: CreateTaskRequest,
        user_id: Uuid,
    ) -> Result<TaskResponse, TaskServiceError> {
        task_data.validate()?;

        // Validate that the project belongs to the user if project_id is provided
        if let Some(project_id) = task_data.project_id {
            let project = self
                .project_repository
                .find_by_id(project_id, user_id)
                .await?;
            if project.is_none() {
                return Err(TaskServiceError::InvalidProject);
            }
        }

        let task = self.task_repository.create(&task_data, user_id).await?;
        Ok(TaskResponse::from(task))
    }

    pub async fn get_task_by_id(
        &self,
        task_id: Uuid,
        user_id: Uuid,
    ) -> Result<TaskResponse, TaskServiceError> {
        let task = self
            .task_repository
            .find_by_id(task_id, user_id)
            .await?
            .ok_or(TaskServiceError::NotFound)?;

        Ok(TaskResponse::from(task))
    }

    pub async fn get_tasks_by_user(
        &self,
        user_id: Uuid,
        include_inactive: Option<bool>,
    ) -> Result<Vec<TaskResponse>, TaskServiceError> {
        let tasks = self.task_repository.find_by_user(user_id, include_inactive).await?;
        Ok(tasks.into_iter().map(TaskResponse::from).collect())
    }

    pub async fn get_tasks_by_project(
        &self,
        project_id: Uuid,
        user_id: Uuid,
        include_inactive: Option<bool>,
    ) -> Result<Vec<TaskResponse>, TaskServiceError> {
        // Verify project belongs to user
        let project = self
            .project_repository
            .find_by_id(project_id, user_id)
            .await?;
        if project.is_none() {
            return Err(TaskServiceError::InvalidProject);
        }

        let tasks = self.task_repository.find_by_project(project_id, user_id, include_inactive).await?;
        Ok(tasks.into_iter().map(TaskResponse::from).collect())
    }

    pub async fn get_tasks_without_project(
        &self,
        user_id: Uuid,
        include_inactive: Option<bool>,
    ) -> Result<Vec<TaskResponse>, TaskServiceError> {
        let tasks = self.task_repository.find_without_project(user_id, include_inactive).await?;
        Ok(tasks.into_iter().map(TaskResponse::from).collect())
    }

    pub async fn update_task(
        &self,
        task_id: Uuid,
        user_id: Uuid,
        task_data: UpdateTaskRequest,
    ) -> Result<TaskResponse, TaskServiceError> {
        task_data.validate()?;

        // Validate that the new project belongs to the user if project_id is being updated
        if let Some(project_id) = task_data.project_id {
            let project = self
                .project_repository
                .find_by_id(project_id, user_id)
                .await?;
            if project.is_none() {
                return Err(TaskServiceError::InvalidProject);
            }
        }

        let task = self
            .task_repository
            .update(task_id, user_id, &task_data)
            .await?
            .ok_or(TaskServiceError::NotFound)?;

        Ok(TaskResponse::from(task))
    }

    pub async fn delete_task(
        &self,
        task_id: Uuid,
        user_id: Uuid,
        soft_delete: Option<bool>,
    ) -> Result<bool, TaskServiceError> {
        let soft_delete = soft_delete.unwrap_or(true);

        let success = if soft_delete {
            self.task_repository.soft_delete(task_id, user_id).await?
        } else {
            self.task_repository.delete(task_id, user_id).await?
        };

        Ok(success)
    }

    pub async fn archive_task(
        &self,
        task_id: Uuid,
        user_id: Uuid,
    ) -> Result<TaskResponse, TaskServiceError> {
        let update_data = UpdateTaskRequest {
            name: None,
            description: None,
            project_id: None,
            is_active: Some(false),
        };

        self.update_task(task_id, user_id, update_data).await
    }

    pub async fn restore_task(
        &self,
        task_id: Uuid,
        user_id: Uuid,
    ) -> Result<TaskResponse, TaskServiceError> {
        let update_data = UpdateTaskRequest {
            name: None,
            description: None,
            project_id: None,
            is_active: Some(true),
        };

        self.update_task(task_id, user_id, update_data).await
    }
}