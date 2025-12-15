use crate::app::models::project::{ProjectResponse, CreateProjectRequest, UpdateProjectRequest};
use crate::app::repositories::project_repository::ProjectRepository;
use crate::app::services::project_service::{ProjectService, ProjectServiceError};
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
pub struct ProjectsState {
    project_service: Arc<ProjectService>,
}

impl ProjectsState {
    pub fn new(pool: PgPool) -> Self {
        let project_repository = ProjectRepository::new(pool);
        let project_service = Arc::new(ProjectService::new(project_repository));
        Self { project_service }
    }
}

#[derive(Serialize)]
struct ErrorResponse {
    error: String,
}

#[derive(Deserialize)]
struct ProjectQueryParams {
    include_inactive: Option<bool>,
}

#[derive(Deserialize)]
struct DeleteQueryParams {
    soft: Option<bool>,
}


pub fn routes() -> Router<ProjectsState> {
    Router::new()
        .route("/", get(list_projects).post(create_project))
        .route("/{id}", get(get_project).put(update_project).delete(delete_project))
        .route("/{id}/archive", put(archive_project))
        .route("/{id}/restore", put(restore_project))
}

async fn create_project(
    State(state): State<ProjectsState>,
    auth_user: AuthUser,
    Json(request): Json<CreateProjectRequest>,
) -> Result<Json<ProjectResponse>, (StatusCode, Json<ErrorResponse>)> {
    let user_id = auth_user.user_id;

    match state.project_service.create_project(request, user_id).await {
        Ok(project) => Ok(Json(project)),
        Err(ProjectServiceError::ValidationError(e)) => Err((
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse {
                error: format!("Validation error: {}", e),
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

async fn get_project(
    State(state): State<ProjectsState>,
    auth_user: AuthUser,
    Path(id): Path<Uuid>,
) -> Result<Json<ProjectResponse>, (StatusCode, Json<ErrorResponse>)> {
    let user_id = auth_user.user_id;

    match state.project_service.get_project_by_id(id, user_id).await {
        Ok(project) => Ok(Json(project)),
        Err(ProjectServiceError::NotFound) => Err((
            StatusCode::NOT_FOUND,
            Json(ErrorResponse {
                error: "Project not found".to_string(),
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

async fn list_projects(
    State(state): State<ProjectsState>,
    auth_user: AuthUser,
    Query(params): Query<ProjectQueryParams>,
) -> Result<Json<Vec<ProjectResponse>>, (StatusCode, Json<ErrorResponse>)> {
    let user_id = auth_user.user_id;

    match state.project_service.get_projects_by_user(user_id, params.include_inactive).await {
        Ok(projects) => Ok(Json(projects)),
        Err(e) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse {
                error: e.to_string(),
            }),
        )),
    }
}

async fn update_project(
    State(state): State<ProjectsState>,
    auth_user: AuthUser,
    Path(id): Path<Uuid>,
    Json(request): Json<UpdateProjectRequest>,
) -> Result<Json<ProjectResponse>, (StatusCode, Json<ErrorResponse>)> {
    let user_id = auth_user.user_id;

    match state.project_service.update_project(id, user_id, request).await {
        Ok(project) => Ok(Json(project)),
        Err(ProjectServiceError::NotFound) => Err((
            StatusCode::NOT_FOUND,
            Json(ErrorResponse {
                error: "Project not found".to_string(),
            }),
        )),
        Err(ProjectServiceError::ValidationError(e)) => Err((
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse {
                error: format!("Validation error: {}", e),
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

async fn delete_project(
    State(state): State<ProjectsState>,
    auth_user: AuthUser,
    Path(id): Path<Uuid>,
    Query(params): Query<DeleteQueryParams>,
) -> Result<StatusCode, (StatusCode, Json<ErrorResponse>)> {
    let user_id = auth_user.user_id;

    match state.project_service.delete_project(id, user_id, params.soft).await {
        Ok(true) => Ok(StatusCode::NO_CONTENT),
        Ok(false) => Err((
            StatusCode::NOT_FOUND,
            Json(ErrorResponse {
                error: "Project not found".to_string(),
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

async fn archive_project(
    State(state): State<ProjectsState>,
    auth_user: AuthUser,
    Path(id): Path<Uuid>,
) -> Result<Json<ProjectResponse>, (StatusCode, Json<ErrorResponse>)> {
    let user_id = auth_user.user_id;

    match state.project_service.archive_project(id, user_id).await {
        Ok(project) => Ok(Json(project)),
        Err(ProjectServiceError::NotFound) => Err((
            StatusCode::NOT_FOUND,
            Json(ErrorResponse {
                error: "Project not found".to_string(),
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

async fn restore_project(
    State(state): State<ProjectsState>,
    auth_user: AuthUser,
    Path(id): Path<Uuid>,
) -> Result<Json<ProjectResponse>, (StatusCode, Json<ErrorResponse>)> {
    let user_id = auth_user.user_id;

    match state.project_service.restore_project(id, user_id).await {
        Ok(project) => Ok(Json(project)),
        Err(ProjectServiceError::NotFound) => Err((
            StatusCode::NOT_FOUND,
            Json(ErrorResponse {
                error: "Project not found".to_string(),
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