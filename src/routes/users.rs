use axum::{
    routing::get,
    Router, Json, extract::{State, Path},
    http::StatusCode,
};
use uuid::Uuid;
use serde::{Serialize};
use crate::app::services::user_service::UserService;
use crate::app::repositories::user_repository::UserRepository;
use crate::app::models::user::User;
use sqlx::PgPool;
use std::sync::Arc;

#[derive(Clone)]
pub struct AppState {
    user_service: Arc<UserService>,
}

impl AppState {
    pub fn new(pool: PgPool) -> Self {
        let user_repository = UserRepository::new(pool);
        let user_service = Arc::new(UserService::new(user_repository));
        Self { user_service }
    }
}


#[derive(Serialize)]
struct ErrorResponse {
    error: String,
}

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/", get(list_users))
        .route("/email/{email}", get(get_user_by_email))
        .route("/{id}", get(get_user))
}

async fn list_users(
    State(state): State<AppState>,
) -> Result<Json<Vec<User>>, (StatusCode, Json<ErrorResponse>)> {
    match state.user_service.get_all_users().await {
        Ok(users) => Ok(Json(users)),
        Err(e) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse { error: e.to_string() }),
        )),
    }
}

async fn get_user(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<User>, (StatusCode, Json<ErrorResponse>)> {
    match state.user_service.get_user_by_id(id).await {
        Ok(Some(user)) => Ok(Json(user)),
        Ok(None) => Err((
            StatusCode::NOT_FOUND,
            Json(ErrorResponse { error: "User not found".to_string() }),
        )),
        Err(e) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse { error: e.to_string() }),
        )),
    }
}

async fn get_user_by_email(
    State(state): State<AppState>,
    Path(email): Path<String>,
) -> Result<Json<User>, (StatusCode, Json<ErrorResponse>)> {
    match state.user_service.get_user_by_email(&email).await {
        Ok(Some(user)) => Ok(Json(user)),
        Ok(None) => Err((
            StatusCode::NOT_FOUND,
            Json(ErrorResponse { error: "User not found".to_string() }),
        )),
        Err(e) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse { error: e.to_string() }),
        )),
    }
}


