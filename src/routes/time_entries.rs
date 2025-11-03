use crate::app::middleware::auth_middleware::AuthUser;
use crate::app::models::time_entry::{
    CreateTimeEntryRequest, UpdateTimeEntryRequest, TimeEntryFilters, 
    TimeEntryResponse, TimeEntriesListResponse, TimeEntryError
};
use crate::app::repositories::time_entry_repository::TimeEntryRepository;
use crate::app::services::time_entry_service::TimeEntryService;
use axum::{
    Json, Router,
    extract::{Path, Query, State},
    http::StatusCode,
    routing::{get, post, patch, delete},
};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use std::sync::Arc;
use uuid::Uuid;
use validator::Validate;

#[derive(Clone)]
pub struct TimeEntriesState {
    service: Arc<TimeEntryService>,
}

impl TimeEntriesState {
    pub fn new(pool: PgPool) -> Self {
        let repository = TimeEntryRepository::new(pool);
        let service = Arc::new(TimeEntryService::new(repository));
        Self { service }
    }
}

#[derive(Serialize)]
struct ErrorResponse {
    error: String,
}

#[derive(Deserialize, Validate)]
struct StartTimerRequest {
    #[validate(length(max = 1000, message = "Description cannot exceed 1000 characters"))]
    pub description: Option<String>,
    pub project_id: Option<Uuid>,
    pub task_id: Option<Uuid>,
}

pub fn routes() -> Router<TimeEntriesState> {
    Router::new()
        .route("/", post(create_time_entry))
        .route("/", get(list_time_entries))
        .route("/start", post(start_timer))
        .route("/current", get(get_current_timer))
        .route("/:id", get(get_time_entry))
        .route("/:id", patch(update_time_entry))
        .route("/:id", delete(delete_time_entry))
        .route("/:id/stop", patch(stop_timer))
}

async fn create_time_entry(
    State(state): State<TimeEntriesState>,
    auth_user: AuthUser,
    Json(request): Json<CreateTimeEntryRequest>,
) -> Result<Json<TimeEntryResponse>, (StatusCode, Json<ErrorResponse>)> {
    match state.service.create_time_entry(auth_user.user_id, request).await {
        Ok(entry) => Ok(Json(entry.into())),
        Err(e) => {
            let (status, message) = match e {
                TimeEntryError::ValidationError(msg) => (StatusCode::BAD_REQUEST, msg),
                TimeEntryError::InvalidTimeRange => (StatusCode::BAD_REQUEST, "End time must be after start time".to_string()),
                TimeEntryError::RunningTimerExists => (StatusCode::CONFLICT, "User already has a running timer. Stop the current timer before starting a new one".to_string()),
                TimeEntryError::DatabaseError(_) => (StatusCode::INTERNAL_SERVER_ERROR, "Database error occurred".to_string()),
                _ => (StatusCode::INTERNAL_SERVER_ERROR, "An unexpected error occurred".to_string()),
            };
            Err((status, Json(ErrorResponse { error: message })))
        }
    }
}

async fn start_timer(
    State(state): State<TimeEntriesState>,
    auth_user: AuthUser,
    Json(request): Json<StartTimerRequest>,
) -> Result<Json<TimeEntryResponse>, (StatusCode, Json<ErrorResponse>)> {
    if let Err(validation_errors) = request.validate() {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse { 
                error: validation_errors.to_string() 
            })
        ));
    }

    match state.service.start_timer(
        auth_user.user_id,
        request.description,
        request.project_id,
        request.task_id,
    ).await {
        Ok(entry) => Ok(Json(entry.into())),
        Err(e) => {
            let (status, message) = match e {
                TimeEntryError::RunningTimerExists => (StatusCode::CONFLICT, "User already has a running timer. Stop the current timer before starting a new one".to_string()),
                TimeEntryError::DatabaseError(_) => (StatusCode::INTERNAL_SERVER_ERROR, "Database error occurred".to_string()),
                _ => (StatusCode::INTERNAL_SERVER_ERROR, "An unexpected error occurred".to_string()),
            };
            Err((status, Json(ErrorResponse { error: message })))
        }
    }
}

async fn get_current_timer(
    State(state): State<TimeEntriesState>,
    auth_user: AuthUser,
) -> Result<Json<TimeEntryResponse>, (StatusCode, Json<ErrorResponse>)> {
    match state.service.get_current_timer(auth_user.user_id).await {
        Ok(entry) => Ok(Json(entry.into())),
        Err(TimeEntryError::NotFound) => Err((
            StatusCode::NOT_FOUND,
            Json(ErrorResponse { 
                error: "No running timer found".to_string() 
            })
        )),
        Err(e) => {
            let message = match e {
                TimeEntryError::DatabaseError(_) => "Database error occurred".to_string(),
                _ => "An unexpected error occurred".to_string(),
            };
            Err((StatusCode::INTERNAL_SERVER_ERROR, Json(ErrorResponse { error: message })))
        }
    }
}

async fn list_time_entries(
    State(state): State<TimeEntriesState>,
    auth_user: AuthUser,
    Query(filters): Query<TimeEntryFilters>,
) -> Result<Json<TimeEntriesListResponse>, (StatusCode, Json<ErrorResponse>)> {
    match state.service.list_time_entries(auth_user.user_id, filters.clone()).await {
        Ok((entries, total_count, total_duration)) => {
            let response = TimeEntriesListResponse {
                entries: entries.into_iter().map(|e| e.into()).collect(),
                total_count,
                total_duration,
                page: filters.page.unwrap_or(1),
                per_page: filters.limit.unwrap_or(20),
            };
            Ok(Json(response))
        }
        Err(e) => {
            let (status, message) = match e {
                TimeEntryError::ValidationError(msg) => (StatusCode::BAD_REQUEST, msg),
                TimeEntryError::DatabaseError(_) => (StatusCode::INTERNAL_SERVER_ERROR, "Database error occurred".to_string()),
                _ => (StatusCode::INTERNAL_SERVER_ERROR, "An unexpected error occurred".to_string()),
            };
            Err((status, Json(ErrorResponse { error: message })))
        }
    }
}

async fn get_time_entry(
    State(state): State<TimeEntriesState>,
    auth_user: AuthUser,
    Path(id): Path<Uuid>,
) -> Result<Json<TimeEntryResponse>, (StatusCode, Json<ErrorResponse>)> {
    match state.service.get_time_entry(id, auth_user.user_id).await {
        Ok(entry) => Ok(Json(entry.into())),
        Err(e) => {
            let (status, message) = match e {
                TimeEntryError::NotFound => (StatusCode::NOT_FOUND, "Time entry not found".to_string()),
                TimeEntryError::Forbidden => (StatusCode::FORBIDDEN, "Access denied".to_string()),
                TimeEntryError::DatabaseError(_) => (StatusCode::INTERNAL_SERVER_ERROR, "Database error occurred".to_string()),
                _ => (StatusCode::INTERNAL_SERVER_ERROR, "An unexpected error occurred".to_string()),
            };
            Err((status, Json(ErrorResponse { error: message })))
        }
    }
}

async fn update_time_entry(
    State(state): State<TimeEntriesState>,
    auth_user: AuthUser,
    Path(id): Path<Uuid>,
    Json(request): Json<UpdateTimeEntryRequest>,
) -> Result<Json<TimeEntryResponse>, (StatusCode, Json<ErrorResponse>)> {
    match state.service.update_time_entry(id, auth_user.user_id, request).await {
        Ok(entry) => Ok(Json(entry.into())),
        Err(e) => {
            let (status, message) = match e {
                TimeEntryError::NotFound => (StatusCode::NOT_FOUND, "Time entry not found".to_string()),
                TimeEntryError::Forbidden => (StatusCode::FORBIDDEN, "Access denied".to_string()),
                TimeEntryError::ValidationError(msg) => (StatusCode::BAD_REQUEST, msg),
                TimeEntryError::InvalidTimeRange => (StatusCode::BAD_REQUEST, "End time must be after start time".to_string()),
                TimeEntryError::DatabaseError(_) => (StatusCode::INTERNAL_SERVER_ERROR, "Database error occurred".to_string()),
                _ => (StatusCode::INTERNAL_SERVER_ERROR, "An unexpected error occurred".to_string()),
            };
            Err((status, Json(ErrorResponse { error: message })))
        }
    }
}

async fn delete_time_entry(
    State(state): State<TimeEntriesState>,
    auth_user: AuthUser,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, (StatusCode, Json<ErrorResponse>)> {
    match state.service.delete_time_entry(id, auth_user.user_id).await {
        Ok(()) => Ok(StatusCode::NO_CONTENT),
        Err(e) => {
            let (status, message) = match e {
                TimeEntryError::NotFound => (StatusCode::NOT_FOUND, "Time entry not found".to_string()),
                TimeEntryError::Forbidden => (StatusCode::FORBIDDEN, "Access denied".to_string()),
                TimeEntryError::DatabaseError(_) => (StatusCode::INTERNAL_SERVER_ERROR, "Database error occurred".to_string()),
                _ => (StatusCode::INTERNAL_SERVER_ERROR, "An unexpected error occurred".to_string()),
            };
            Err((status, Json(ErrorResponse { error: message })))
        }
    }
}

async fn stop_timer(
    State(state): State<TimeEntriesState>,
    auth_user: AuthUser,
    Path(id): Path<Uuid>,
) -> Result<Json<TimeEntryResponse>, (StatusCode, Json<ErrorResponse>)> {
    match state.service.stop_timer(id, auth_user.user_id).await {
        Ok(entry) => Ok(Json(entry.into())),
        Err(e) => {
            let (status, message) = match e {
                TimeEntryError::NotFound => (StatusCode::NOT_FOUND, "Time entry not found".to_string()),
                TimeEntryError::Forbidden => (StatusCode::FORBIDDEN, "Access denied".to_string()),
                TimeEntryError::TimerNotRunning => (StatusCode::BAD_REQUEST, "Timer is not currently running".to_string()),
                TimeEntryError::DatabaseError(_) => (StatusCode::INTERNAL_SERVER_ERROR, "Database error occurred".to_string()),
                _ => (StatusCode::INTERNAL_SERVER_ERROR, "An unexpected error occurred".to_string()),
            };
            Err((status, Json(ErrorResponse { error: message })))
        }
    }
}