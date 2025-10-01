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
use uuid::Uuid;
use crate::app::services::auth_service::AuthService;
use crate::app::services::jwt_service::JwtService;
use crate::app::services::secure_login_service::SecureLoginService;
use crate::app::models::auth::{RegisterRequest, RegisterResponse, AuthError, ForgotPasswordRequest, ForgotPasswordResponse, ResetPasswordRequest, ResetPasswordResponse};
use crate::app::models::jwt::{LoginRequest, LoginResponse, RefreshTokenRequest, RefreshTokenResponse, LogoutRequest, LogoutResponse, JwtError};
use crate::app::middleware::security::{SecurityState, check_registration_rate_limit, check_login_rate_limit, check_refresh_rate_limit, check_password_reset_rate_limit, log_security_event};

#[derive(Clone)]
pub struct AuthAppState {
    pub auth_service: Arc<AuthService>,
    pub jwt_service: Arc<JwtService>,
    pub secure_login_service: Arc<SecureLoginService>,
    pub security_state: Arc<SecurityState>,
}

impl AuthAppState {
    pub fn new(auth_service: AuthService, jwt_service: JwtService, secure_login_service: SecureLoginService, security_state: SecurityState) -> Self {
        Self {
            auth_service: Arc::new(auth_service),
            jwt_service: Arc::new(jwt_service),
            secure_login_service: Arc::new(secure_login_service),
            security_state: Arc::new(security_state),
        }
    }
}

pub fn routes() -> Router<AuthAppState> {
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
}

fn extract_real_ip(addr: SocketAddr, headers: &HeaderMap) -> String {
    // Check for forwarded IP headers first (for proxy/load balancer scenarios)
    if let Some(forwarded_for) = headers.get("x-forwarded-for") {
        if let Ok(forwarded_str) = forwarded_for.to_str() {
            // x-forwarded-for can contain multiple IPs, take the first one
            if let Some(first_ip) = forwarded_str.split(',').next() {
                return first_ip.trim().to_string();
            }
        }
    }

    if let Some(real_ip) = headers.get("x-real-ip") {
        if let Ok(real_ip_str) = real_ip.to_str() {
            return real_ip_str.to_string();
        }
    }

    // Fall back to connection IP
    addr.ip().to_string()
}

async fn register(
    State(state): State<AuthAppState>,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    headers: HeaderMap,
    Json(request): Json<RegisterRequest>,
) -> Result<(StatusCode, Json<RegisterResponse>), (StatusCode, Json<AuthError>)> {
    let ip_address = extract_real_ip(addr, &headers);
    let user_agent = headers.get("user-agent")
        .and_then(|h| h.to_str().ok());

    // Check rate limiting for registration attempts
    if let Err(response) = check_registration_rate_limit(&state.security_state, &ip_address) {
        log_security_event("registration_rate_limit_exceeded", &ip_address, user_agent, None, Some(&request.email), false, Some("Rate limit exceeded"));
        return Err((StatusCode::TOO_MANY_REQUESTS, Json(AuthError::new("Too many registration attempts. Please try again later."))));
    }

    match state.auth_service.register(request.clone()).await {
        Ok(response) => {
            log_security_event("user_registration", &ip_address, user_agent, Some(&response.user.id.to_string()), Some(&request.email), true, None);
            Ok((StatusCode::CREATED, Json(response)))
        }
        Err(error) => {
            log_security_event("user_registration_failed", &ip_address, user_agent, None, Some(&request.email), false, Some(&error.error));
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
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    headers: HeaderMap,
    Json(request): Json<ForgotPasswordRequest>,
) -> Result<(StatusCode, Json<ForgotPasswordResponse>), (StatusCode, Json<AuthError>)> {
    let ip_address = extract_real_ip(addr, &headers);
    let user_agent = headers.get("user-agent")
        .and_then(|h| h.to_str().ok());

    // Check rate limiting for password reset attempts
    if let Err(response) = check_password_reset_rate_limit(&state.security_state, &request.email) {
        log_security_event("password_reset_rate_limit_exceeded", &ip_address, user_agent, None, Some(&request.email), false, Some("Rate limit exceeded"));
        return Err((StatusCode::TOO_MANY_REQUESTS, Json(AuthError::new("Too many password reset requests. Please try again later."))));
    }

    match state.auth_service.forgot_password(request.clone()).await {
        Ok(response) => {
            log_security_event("password_reset_requested", &ip_address, user_agent, None, Some(&request.email), true, None);
            Ok((StatusCode::OK, Json(response)))
        }
        Err(error) => {
            log_security_event("password_reset_request_failed", &ip_address, user_agent, None, Some(&request.email), false, Some(&error.error));
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
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    headers: HeaderMap,
    Json(request): Json<RefreshTokenRequest>,
) -> Result<(StatusCode, Json<RefreshTokenResponse>), (StatusCode, String)> {
    let ip_address = addr.ip().to_string();
    let user_agent = headers.get("user-agent")
        .and_then(|h| h.to_str().ok());

    // First, validate the refresh token to get user info for rate limiting
    let claims = match state.jwt_service.decode_token_without_validation(&request.refresh_token) {
        Ok(claims) => claims,
        Err(error) => {
            log_security_event("refresh_token_validation_failed", &ip_address, user_agent, None, None, false, Some(&error.to_string()));
            let status_code = match error {
                JwtError::ExpiredToken => StatusCode::UNAUTHORIZED,
                JwtError::InvalidToken(_) => StatusCode::UNAUTHORIZED,
                JwtError::BlacklistedToken => StatusCode::UNAUTHORIZED,
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            };
            return Err((status_code, error.to_string()));
        }
    };

    // Check rate limiting for token refresh attempts
    if let Err(_) = check_refresh_rate_limit(&state.security_state, &claims.sub) {
        log_security_event("refresh_rate_limit_exceeded", &ip_address, user_agent, Some(&claims.sub), Some(&claims.email), false, Some("Rate limit exceeded"));
        return Err((StatusCode::TOO_MANY_REQUESTS, "Too many token refresh attempts. Please wait before retrying.".to_string()));
    }

    let user_id = match Uuid::parse_str(&claims.sub) {
        Ok(id) => id,
        Err(_) => {
            log_security_event("refresh_token_invalid_user_id", &ip_address, user_agent, Some(&claims.sub), Some(&claims.email), false, Some("Invalid user ID in token"));
            return Err((StatusCode::UNAUTHORIZED, "Invalid token".to_string()));
        }
    };

    // Use token rotation for enhanced security
    match state.jwt_service.refresh_with_rotation(&request.refresh_token, user_id).await {
        Ok(tokens) => {
            let response = RefreshTokenResponse {
                access_token: tokens.access_token,
                refresh_token: Some(tokens.refresh_token),
                token_type: "Bearer".to_string(),
                expires_in: 15 * 60, // 15 minutes
                refresh_expires_in: Some(7 * 24 * 60 * 60), // 7 days
            };
            log_security_event("token_refreshed", &ip_address, user_agent, Some(&claims.sub), Some(&claims.email), true, Some("Token rotated"));
            Ok((StatusCode::OK, Json(response)))
        }
        Err(error) => {
            log_security_event("token_refresh_failed", &ip_address, user_agent, Some(&claims.sub), Some(&claims.email), false, Some(&error.to_string()));
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
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    headers: axum::http::HeaderMap,
    Json(request): Json<LogoutRequest>,
) -> Result<(StatusCode, Json<LogoutResponse>), (StatusCode, String)> {
    let ip_address = extract_real_ip(addr, &headers);
    let user_agent = headers.get("user-agent")
        .and_then(|h| h.to_str().ok());

    // Extract and validate the access token from Authorization header
    let auth_header = headers
        .get(axum::http::header::AUTHORIZATION)
        .and_then(|header| header.to_str().ok());

    let access_token = if let Some(header) = auth_header {
        JwtService::extract_token_from_header(header)
            .map_err(|err| (StatusCode::UNAUTHORIZED, err.to_string()))?
    } else {
        // Return success even if no token provided (prevents information leakage)
        log_security_event("logout_no_token", &ip_address, user_agent, None, None, true, Some("Logout attempted without token"));

        let response = LogoutResponse {
            message: "Logged out successfully".to_string(),
            logged_out_devices: None,
        };
        return Ok((StatusCode::OK, Json(response)));
    };

    // Get user info from token (even if expired/invalid for logging)
    let claims = match state.jwt_service.decode_token_without_validation(access_token) {
        Ok(claims) => Some(claims),
        Err(_) => None,
    };

    let user_id = claims.as_ref()
        .and_then(|c| Uuid::parse_str(&c.sub).ok());
    let user_email = claims.as_ref()
        .map(|c| &c.email);

    // Always try to perform logout operations, but don't fail if token is already invalid
    let mut logged_out_devices = 0u32;

    // Blacklist the current access token (ignore errors for invalid/expired tokens)
    let _ = state.jwt_service.blacklist_token(access_token).await;
    logged_out_devices += 1;

    if let Some(user_id) = user_id {
        // Handle "logout all devices" functionality
        if request.logout_all_devices.unwrap_or(false) {
            // Revoke all refresh tokens for the user
            match state.jwt_service.revoke_all_user_refresh_tokens(user_id).await {
                Ok(()) => {
                    // Note: We can't easily count revoked devices, so we use a placeholder
                    logged_out_devices = 999; // Indicates "all devices"
                    log_security_event("logout_all_devices", &ip_address, user_agent,
                                     Some(&user_id.to_string()), user_email.map(|x| x.as_str()), true,
                                     Some("All devices logged out"));
                },
                Err(error) => {
                    log_security_event("logout_all_devices_failed", &ip_address, user_agent,
                                     Some(&user_id.to_string()), user_email.map(|x| x.as_str()), false,
                                     Some(&error.to_string()));
                }
            }
        } else {
            // Handle single device logout
            if let Some(refresh_token) = &request.refresh_token {
                // Try to revoke the specific refresh token (ignore errors)
                let _ = state.jwt_service.revoke_refresh_token(refresh_token).await;
                let _ = state.jwt_service.blacklist_token(refresh_token).await;
            }
        }

        // Cleanup expired tokens during logout
        let _ = state.jwt_service.cleanup_expired_blacklisted_tokens().await;
        let _ = state.jwt_service.cleanup_expired_refresh_tokens().await;

        log_security_event("logout_successful", &ip_address, user_agent,
                         Some(&user_id.to_string()), user_email.map(|x| x.as_str()), true,
                         Some(&format!("Devices logged out: {}", logged_out_devices)));
    } else {
        log_security_event("logout_invalid_token", &ip_address, user_agent, None, None, true,
                         Some("Logout with invalid token"));
    }

    let response = LogoutResponse {
        message: "Logged out successfully".to_string(),
        logged_out_devices: if logged_out_devices == 999 {
            None // Don't expose the actual count for "all devices"
        } else if logged_out_devices > 0 {
            Some(logged_out_devices)
        } else {
            None
        },
    };

    Ok((StatusCode::OK, Json(response)))
}