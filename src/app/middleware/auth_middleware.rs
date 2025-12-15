use crate::app::models::jwt::{AuthContext, Claims, JwtError};
use crate::app::services::jwt_service::JwtService;
use axum::{
    extract::{Request, State},
    http::{StatusCode, header::AUTHORIZATION},
    middleware::Next,
    response::Response,
};
use std::sync::Arc;

// Auth middleware that validates JWT tokens
pub async fn jwt_auth_middleware(
    State(jwt_service): State<Arc<JwtService>>,
    mut request: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    // Extract authorization header
    let auth_header = request
        .headers()
        .get(AUTHORIZATION)
        .and_then(|header| header.to_str().ok())
        .ok_or(StatusCode::UNAUTHORIZED)?;

    // Extract token from header
    let token =
        JwtService::extract_token_from_header(auth_header).map_err(|_| StatusCode::UNAUTHORIZED)?;

    // Validate token
    let claims = jwt_service
        .validate_token(token)
        .await
        .map_err(|err| match err {
            JwtError::ExpiredToken => StatusCode::UNAUTHORIZED,
            JwtError::BlacklistedToken => StatusCode::UNAUTHORIZED,
            JwtError::InvalidToken(_) => StatusCode::UNAUTHORIZED,
            _ => StatusCode::INTERNAL_SERVER_ERROR,
        })?;

    // Create auth context and add to request extensions
    let auth_context = AuthContext::from(claims);
    request.extensions_mut().insert(auth_context);

    Ok(next.run(request).await)
}

// Optional middleware for handling JWT auth errors with custom responses
pub async fn jwt_auth_middleware_with_json_errors(
    State(jwt_service): State<Arc<JwtService>>,
    mut request: Request,
    next: Next,
) -> Response {
    // Extract authorization header
    let auth_header = match request
        .headers()
        .get(AUTHORIZATION)
        .and_then(|header| header.to_str().ok())
    {
        Some(header) => header,
        None => {
            return create_auth_error_response(
                StatusCode::UNAUTHORIZED,
                "Missing authorization header",
            );
        }
    };

    // Extract token from header
    let token = match JwtService::extract_token_from_header(auth_header) {
        Ok(token) => token,
        Err(_) => {
            return create_auth_error_response(
                StatusCode::UNAUTHORIZED,
                "Invalid authorization header format",
            );
        }
    };

    // Validate token
    let claims = match jwt_service.validate_token(token).await {
        Ok(claims) => claims,
        Err(err) => {
            let (status, message) = match err {
                JwtError::ExpiredToken => (StatusCode::UNAUTHORIZED, "Token has expired"),
                JwtError::BlacklistedToken => (StatusCode::UNAUTHORIZED, "Token has been revoked"),
                JwtError::InvalidToken(_) => (StatusCode::UNAUTHORIZED, "Invalid token"),
                JwtError::MissingToken => (StatusCode::UNAUTHORIZED, "Missing token"),
                _ => (StatusCode::INTERNAL_SERVER_ERROR, "Authentication error"),
            };
            return create_auth_error_response(status, message);
        }
    };

    // Create auth context and add to request extensions
    let auth_context = AuthContext::from(claims);
    request.extensions_mut().insert(auth_context);

    next.run(request).await
}

// Helper function to create JSON error responses
fn create_auth_error_response(status: StatusCode, message: &str) -> Response {
    let error_json = format!(r#"{{"error": "{}"}}"#, message);

    Response::builder()
        .status(status)
        .header("content-type", "application/json")
        .body(error_json.into())
        .unwrap()
}

// Extension trait to easily extract auth context from request
pub trait AuthContextExt {
    fn auth_context(&self) -> Option<&AuthContext>;
    fn require_auth_context(&self) -> Result<&AuthContext, StatusCode>;
}

impl<T> AuthContextExt for axum::extract::Request<T> {
    fn auth_context(&self) -> Option<&AuthContext> {
        self.extensions().get::<AuthContext>()
    }

    fn require_auth_context(&self) -> Result<&AuthContext, StatusCode> {
        self.auth_context().ok_or(StatusCode::UNAUTHORIZED)
    }
}

// Extension trait for extracting auth context in handlers
impl AuthContextExt for axum::http::Extensions {
    fn auth_context(&self) -> Option<&AuthContext> {
        self.get::<AuthContext>()
    }

    fn require_auth_context(&self) -> Result<&AuthContext, StatusCode> {
        self.auth_context().ok_or(StatusCode::UNAUTHORIZED)
    }
}

// Extractor for auth context in handlers
#[derive(Debug, Clone)]
pub struct AuthUser(pub AuthContext);

impl<S> axum::extract::FromRequestParts<S> for AuthUser
where
    S: Send + Sync,
{
    type Rejection = StatusCode;

    async fn from_request_parts(
        parts: &mut axum::http::request::Parts,
        _state: &S,
    ) -> Result<Self, Self::Rejection> {
        parts
            .extensions
            .get::<AuthContext>()
            .map(|ctx| AuthUser(ctx.clone()))
            .ok_or(StatusCode::UNAUTHORIZED)
    }
}

impl std::ops::Deref for AuthUser {
    type Target = AuthContext;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
