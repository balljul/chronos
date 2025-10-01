use axum::{
    routing::post,
    Router, Json, extract::State,
    http::StatusCode,
};
use std::sync::Arc;
use tower_governor::{
    governor::GovernorConfigBuilder,
    GovernorLayer,
};
use crate::app::services::auth_service::AuthService;
use crate::app::models::auth::{RegisterRequest, RegisterResponse, AuthError, ForgotPasswordRequest, ForgotPasswordResponse, ResetPasswordRequest, ResetPasswordResponse};

#[derive(Clone)]
pub struct AuthAppState {
    pub auth_service: Arc<AuthService>,
}

impl AuthAppState {
    pub fn new(auth_service: AuthService) -> Self {
        Self {
            auth_service: Arc::new(auth_service),
        }
    }
}

pub fn routes() -> Router<AuthAppState> {
    // Configure rate limiting: 5 requests per minute per IP
    let governor_conf = Box::new(
        GovernorConfigBuilder::default()
            .per_second(5)
            .burst_size(5)
            .finish()
            .unwrap(),
    );

    Router::new()
        .route("/register", post(register))
        .route("/forgot-password", post(forgot_password))
        .route("/reset-password", post(reset_password))
        .layer(GovernorLayer::new(governor_conf))
}

async fn register(
    State(state): State<AuthAppState>,
    Json(request): Json<RegisterRequest>,
) -> Result<(StatusCode, Json<RegisterResponse>), (StatusCode, Json<AuthError>)> {
    match state.auth_service.register(request).await {
        Ok(response) => Ok((StatusCode::CREATED, Json(response))),
        Err(error) => {
            let status_code = match error.error.as_str() {
                "Email already registered" => StatusCode::CONFLICT,
                "Validation failed" => StatusCode::BAD_REQUEST,
                _ if error.error.contains("validation") => StatusCode::BAD_REQUEST,
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            };
            Err((status_code, Json(error)))
        }
    }
}

async fn forgot_password(
    State(state): State<AuthAppState>,
    Json(request): Json<ForgotPasswordRequest>,
) -> Result<(StatusCode, Json<ForgotPasswordResponse>), (StatusCode, Json<AuthError>)> {
    match state.auth_service.forgot_password(request).await {
        Ok(response) => Ok((StatusCode::OK, Json(response))),
        Err(error) => {
            let status_code = match error.error.as_str() {
                "Too many password reset requests. Please try again later." => StatusCode::TOO_MANY_REQUESTS,
                "Validation failed" => StatusCode::BAD_REQUEST,
                _ if error.error.contains("validation") => StatusCode::BAD_REQUEST,
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            };
            Err((status_code, Json(error)))
        }
    }
}

async fn reset_password(
    State(state): State<AuthAppState>,
    Json(request): Json<ResetPasswordRequest>,
) -> Result<(StatusCode, Json<ResetPasswordResponse>), (StatusCode, Json<AuthError>)> {
    match state.auth_service.reset_password(request).await {
        Ok(response) => Ok((StatusCode::OK, Json(response))),
        Err(error) => {
            let status_code = match error.error.as_str() {
                "Invalid or expired reset token" => StatusCode::BAD_REQUEST,
                "Validation failed" => StatusCode::BAD_REQUEST,
                _ if error.error.contains("validation") => StatusCode::BAD_REQUEST,
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            };
            Err((status_code, Json(error)))
        }
    }
}