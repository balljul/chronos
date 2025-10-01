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
use crate::app::services::jwt_service::JwtService;
use crate::app::models::auth::{RegisterRequest, RegisterResponse, AuthError, ForgotPasswordRequest, ForgotPasswordResponse, ResetPasswordRequest, ResetPasswordResponse};
use crate::app::models::jwt::{LoginRequest, LoginResponse, RefreshTokenRequest, RefreshTokenResponse, LogoutRequest, LogoutResponse, JwtError};

#[derive(Clone)]
pub struct AuthAppState {
    pub auth_service: Arc<AuthService>,
    pub jwt_service: Arc<JwtService>,
}

impl AuthAppState {
    pub fn new(auth_service: AuthService, jwt_service: JwtService) -> Self {
        Self {
            auth_service: Arc::new(auth_service),
            jwt_service: Arc::new(jwt_service),
        }
    }
}

pub fn routes() -> Router<AuthAppState> {
    // Configure rate limiting: 10 requests per minute per IP for auth endpoints
    let governor_conf = Box::new(
        GovernorConfigBuilder::default()
            .per_second(10)
            .burst_size(10)
            .finish()
            .unwrap(),
    );

    // Routes that don't require authentication
    let public_routes = Router::new()
        .route("/register", post(register))
        .route("/login", post(login))
        .route("/forgot-password", post(forgot_password))
        .route("/reset-password", post(reset_password))
        .route("/refresh", post(refresh_token));

    // Routes that require authentication
    let protected_routes = Router::new()
        .route("/logout", post(logout));

    Router::new()
        .merge(public_routes)
        .merge(protected_routes)
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

async fn login(
    State(state): State<AuthAppState>,
    Json(request): Json<LoginRequest>,
) -> Result<(StatusCode, Json<LoginResponse>), (StatusCode, Json<AuthError>)> {
    // Find user by email
    let user = match state.auth_service.find_user_by_email(&request.email).await {
        Ok(Some(user)) => user,
        Ok(None) => {
            return Err((StatusCode::UNAUTHORIZED, Json(AuthError::new("Invalid email or password"))));
        }
        Err(_) => {
            return Err((StatusCode::INTERNAL_SERVER_ERROR, Json(AuthError::new("Database error"))));
        }
    };

    // Verify password
    match user.verify_password(&request.password) {
        Ok(true) => {},
        Ok(false) => {
            return Err((StatusCode::UNAUTHORIZED, Json(AuthError::new("Invalid email or password"))));
        }
        Err(_) => {
            return Err((StatusCode::INTERNAL_SERVER_ERROR, Json(AuthError::new("Password verification error"))));
        }
    }

    // Generate JWT tokens
    let token_pair = match state.jwt_service.generate_token_pair(&user).await {
        Ok(tokens) => tokens,
        Err(_) => {
            return Err((StatusCode::INTERNAL_SERVER_ERROR, Json(AuthError::new("Token generation failed"))));
        }
    };

    let response = LoginResponse {
        message: "Login successful".to_string(),
        user: user.to_response(),
        tokens: token_pair,
    };

    Ok((StatusCode::OK, Json(response)))
}

async fn refresh_token(
    State(state): State<AuthAppState>,
    Json(request): Json<RefreshTokenRequest>,
) -> Result<(StatusCode, Json<RefreshTokenResponse>), (StatusCode, String)> {
    match state.jwt_service.refresh_access_token(&request.refresh_token).await {
        Ok(new_access_token) => {
            let response = RefreshTokenResponse {
                access_token: new_access_token,
                token_type: "Bearer".to_string(),
                expires_in: 15 * 60, // 15 minutes
            };
            Ok((StatusCode::OK, Json(response)))
        }
        Err(error) => {
            let status_code = match error {
                JwtError::ExpiredToken => StatusCode::UNAUTHORIZED,
                JwtError::InvalidToken(_) => StatusCode::UNAUTHORIZED,
                JwtError::BlacklistedToken => StatusCode::UNAUTHORIZED,
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            };
            Err((status_code, error.to_string()))
        }
    }
}

async fn logout(
    State(state): State<AuthAppState>,
    headers: axum::http::HeaderMap,
    Json(request): Json<LogoutRequest>,
) -> Result<(StatusCode, Json<LogoutResponse>), (StatusCode, String)> {
    // Extract and validate the access token from Authorization header
    let auth_header = headers
        .get(axum::http::header::AUTHORIZATION)
        .and_then(|header| header.to_str().ok())
        .ok_or_else(|| (StatusCode::UNAUTHORIZED, "Missing authorization header".to_string()))?;

    let access_token = JwtService::extract_token_from_header(auth_header)
        .map_err(|err| (StatusCode::UNAUTHORIZED, err.to_string()))?;

    // Validate the access token
    state.jwt_service
        .validate_token(access_token)
        .await
        .map_err(|err| {
            let status = match err {
                JwtError::ExpiredToken | JwtError::BlacklistedToken | JwtError::InvalidToken(_) => StatusCode::UNAUTHORIZED,
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            };
            (status, err.to_string())
        })?;

    // Blacklist the current access token
    if let Err(error) = state.jwt_service.blacklist_token(access_token).await {
        return Err((StatusCode::INTERNAL_SERVER_ERROR, error.to_string()));
    }

    // If refresh token is provided, blacklist it too
    if let Some(refresh_token) = request.refresh_token {
        if let Err(error) = state.jwt_service.blacklist_token(&refresh_token).await {
            return Err((StatusCode::INTERNAL_SERVER_ERROR, error.to_string()));
        }
    }

    let response = LogoutResponse {
        message: "Logged out successfully".to_string(),
    };

    Ok((StatusCode::OK, Json(response)))
}