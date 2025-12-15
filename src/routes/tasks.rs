use crate::app::models::task::{TaskResponse, CreateTaskRequest, UpdateTaskRequest};
use crate::app::repositories::task_repository::TaskRepository;
use crate::app::repositories::project_repository::ProjectRepository;
use crate::app::services::task_service::{TaskService, TaskServiceError};
use crate::app::middleware::auth_middleware::AuthUser;
use axum::{
    Json, Router,
    extract::{Path, Query, State},
    http::StatusCode,
    routing::{get, post, put, delete},
};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use std::sync::Arc;
use uuid::Uuid;


#[derive(Clone)]
pub struct TasksState {
    task_service: Arc<TaskService>,
}

impl TasksState {
    pub fn new(pool: PgPool) -> Self {
        let task_repository = TaskRepository::new(pool.clone());
        let project_repository = ProjectRepository::new(pool);
        let task_service = Arc::new(TaskService::new(task_repository, project_repository));
        Self { task_service }
    }
}

#[derive(Serialize)]
struct ErrorResponse {
    error: String,
}

#[derive(Deserialize)]
struct TaskQueryParams {
    include_inactive: Option<bool>,
    project_id: Option<Uuid>,
    without_project: Option<bool>,
}

#[derive(Deserialize)]
struct DeleteQueryParams {
    soft: Option<bool>,
}

pub fn routes() -> Router<TasksState> {
    Router::new()
        .route("/", get(list_tasks).post(create_task))
        .route("/{id}", get(get_task).put(update_task).delete(delete_task))
        .route("/{id}/archive", put(archive_task))
        .route("/{id}/restore", put(restore_task))
}

async fn create_task(
    State(state): State<TasksState>,
    auth_user: AuthUser,
    Json(request): Json<CreateTaskRequest>,
) -> Result<Json<TaskResponse>, (StatusCode, Json<ErrorResponse>)> {
    let user_id = auth_user.user_id;

    match state.task_service.create_task(request, user_id).await {
        Ok(task) => Ok(Json(task)),
        Err(TaskServiceError::ValidationError(e)) => Err((
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse {
                error: format!("Validation error: {}", e),
            }),
        )),
        Err(TaskServiceError::InvalidProject) => Err((
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse {
                error: "Project not found or unauthorized".to_string(),
            }),
        )),
        Err(e) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse {
                error: e.to_string(),
            }),
        )),
    }
}

async fn get_task(
    State(state): State<TasksState>,
    auth_user: AuthUser,
    Path(id): Path<Uuid>,
) -> Result<Json<TaskResponse>, (StatusCode, Json<ErrorResponse>)> {
    let user_id = auth_user.user_id;

    match state.task_service.get_task_by_id(id, user_id).await {
        Ok(task) => Ok(Json(task)),
        Err(TaskServiceError::NotFound) => Err((
            StatusCode::NOT_FOUND,
            Json(ErrorResponse {
                error: "Task not found".to_string(),
            }),
        )),
        Err(e) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse {
                error: e.to_string(),
            }),
        )),
    }
}

async fn list_tasks(
    State(state): State<TasksState>,
    auth_user: AuthUser,
    Query(params): Query<TaskQueryParams>,
) -> Result<Json<Vec<TaskResponse>>, (StatusCode, Json<ErrorResponse>)> {
    let user_id = auth_user.user_id;

    let result = if let Some(project_id) = params.project_id {
        state.task_service.get_tasks_by_project(project_id, user_id, params.include_inactive).await
    } else if params.without_project.unwrap_or(false) {
        state.task_service.get_tasks_without_project(user_id, params.include_inactive).await
    } else {
        state.task_service.get_tasks_by_user(user_id, params.include_inactive).await
    };

    match result {
        Ok(tasks) => Ok(Json(tasks)),
        Err(TaskServiceError::InvalidProject) => Err((
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse {
                error: "Project not found or unauthorized".to_string(),
            }),
        )),
        Err(e) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse {
                error: e.to_string(),
            }),
        )),
    }
}

async fn update_task(
    State(state): State<TasksState>,
    auth_user: AuthUser,
    Path(id): Path<Uuid>,
    Json(request): Json<UpdateTaskRequest>,
) -> Result<Json<TaskResponse>, (StatusCode, Json<ErrorResponse>)> {
    let user_id = auth_user.user_id;

    match state.task_service.update_task(id, user_id, request).await {
        Ok(task) => Ok(Json(task)),
        Err(TaskServiceError::NotFound) => Err((
            StatusCode::NOT_FOUND,
            Json(ErrorResponse {
                error: "Task not found".to_string(),
            }),
        )),
        Err(TaskServiceError::ValidationError(e)) => Err((
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse {
                error: format!("Validation error: {}", e),
            }),
        )),
        Err(TaskServiceError::InvalidProject) => Err((
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse {
                error: "Project not found or unauthorized".to_string(),
            }),
        )),
        Err(e) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse {
                error: e.to_string(),
            }),
        )),
    }
}

async fn delete_task(
    State(state): State<TasksState>,
    auth_user: AuthUser,
    Path(id): Path<Uuid>,
    Query(params): Query<DeleteQueryParams>,
) -> Result<StatusCode, (StatusCode, Json<ErrorResponse>)> {
    let user_id = auth_user.user_id;

    match state.task_service.delete_task(id, user_id, params.soft).await {
        Ok(true) => Ok(StatusCode::NO_CONTENT),
        Ok(false) => Err((
            StatusCode::NOT_FOUND,
            Json(ErrorResponse {
                error: "Task not found".to_string(),
            }),
        )),
        Err(e) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse {
                error: e.to_string(),
            }),
        )),
    }
}

async fn archive_task(
    State(state): State<TasksState>,
    auth_user: AuthUser,
    Path(id): Path<Uuid>,
) -> Result<Json<TaskResponse>, (StatusCode, Json<ErrorResponse>)> {
    let user_id = auth_user.user_id;

    match state.task_service.archive_task(id, user_id).await {
        Ok(task) => Ok(Json(task)),
        Err(TaskServiceError::NotFound) => Err((
            StatusCode::NOT_FOUND,
            Json(ErrorResponse {
                error: "Task not found".to_string(),
            }),
        )),
        Err(e) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse {
                error: e.to_string(),
            }),
        )),
    }
}

async fn restore_task(
    State(state): State<TasksState>,
    auth_user: AuthUser,
    Path(id): Path<Uuid>,
) -> Result<Json<TaskResponse>, (StatusCode, Json<ErrorResponse>)> {
    let user_id = auth_user.user_id;

    match state.task_service.restore_task(id, user_id).await {
        Ok(task) => Ok(Json(task)),
        Err(TaskServiceError::NotFound) => Err((
            StatusCode::NOT_FOUND,
            Json(ErrorResponse {
                error: "Task not found".to_string(),
            }),
        )),
        Err(e) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse {
                error: e.to_string(),
            }),
        )),
    }
}