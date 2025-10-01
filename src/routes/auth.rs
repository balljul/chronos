use axum::{
    routing::post,
    Router, Json, extract::State,
    http::{StatusCode, HeaderMap},
    extract::ConnectInfo,
};
use std::net::SocketAddr;
use std::sync::Arc;
use tower_governor::{
    governor::GovernorConfigBuilder,
    GovernorLayer,
};
use crate::app::services::auth_service::AuthService;
use crate::app::services::jwt_service::JwtService;
use crate::app::services::secure_login_service::SecureLoginService;
use crate::app::models::auth::{RegisterRequest, RegisterResponse, AuthError, ForgotPasswordRequest, ForgotPasswordResponse, ResetPasswordRequest, ResetPasswordResponse};
use crate::app::models::jwt::{LoginRequest, LoginResponse, RefreshTokenRequest, RefreshTokenResponse, LogoutRequest, LogoutResponse, JwtError};

#[derive(Clone)]
pub struct AuthAppState {
    pub auth_service: Arc<AuthService>,
    pub jwt_service: Arc<JwtService>,
    pub secure_login_service: Arc<SecureLoginService>,
}

impl AuthAppState {
    pub fn new(auth_service: AuthService, jwt_service: JwtService, secure_login_service: SecureLoginService) -> Self {
        Self {
            auth_service: Arc::new(auth_service),
            jwt_service: Arc::new(jwt_service),
            secure_login_service: Arc::new(secure_login_service),
        }
    }
}

pub fn routes() -> Router<AuthAppState> {
    // Configure rate limiting: general auth endpoints limited to prevent DoS
    // Note: Login has specific rate limiting (5 failed attempts per 15 minutes)
    // implemented in SecureLoginService, this is additional protection
    let governor_conf = Box::new(
        GovernorConfigBuilder::default()
            .per_second(5)  // 5 requests per second max
            .burst_size(10) // Allow bursts of 10
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
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    headers: HeaderMap,
    Json(request): Json<LoginRequest>,
) -> Result<(StatusCode, Json<LoginResponse>), (StatusCode, Json<AuthError>)> {
    // Extract IP address
    let ip_address = addr.ip().to_string();

    // Extract User-Agent
    let user_agent = headers
        .get("user-agent")
        .and_then(|header| header.to_str().ok())
        .map(|s| s.to_string());

    // Use secure login service with enhanced security features
    match state.secure_login_service.secure_login(request, ip_address, user_agent).await {
        Ok(response) => Ok((StatusCode::OK, Json(response))),
        Err(error) => {
            let status_code = match error.error.as_str() {
                "Too many failed login attempts. Please try again later." => StatusCode::TOO_MANY_REQUESTS,
                msg if msg.contains("Account is temporarily locked") => StatusCode::LOCKED,
                msg if msg.contains("Account has been temporarily locked") => StatusCode::LOCKED,
                "Invalid email or password" => StatusCode::UNAUTHORIZED,
                "Authentication failed" => StatusCode::UNAUTHORIZED,
                _ if error.error.contains("Database error") => StatusCode::INTERNAL_SERVER_ERROR,
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            };
            Err((status_code, Json(error)))
        }
    }
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

    // If refresh token is provided, revoke it properly
    if let Some(refresh_token) = request.refresh_token {
        if let Err(error) = state.jwt_service.revoke_refresh_token(&refresh_token).await {
            return Err((StatusCode::INTERNAL_SERVER_ERROR, error.to_string()));
        }
        // Also blacklist the refresh token
        if let Err(error) = state.jwt_service.blacklist_token(&refresh_token).await {
            return Err((StatusCode::INTERNAL_SERVER_ERROR, error.to_string()));
        }
    }

    let response = LogoutResponse {
        message: "Logged out successfully".to_string(),
    };

    Ok((StatusCode::OK, Json(response)))
}