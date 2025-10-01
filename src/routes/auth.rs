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
use crate::app::models::auth::{RegisterRequest, RegisterResponse, AuthError};

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