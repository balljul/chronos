use axum::{
    body::Body,
    extract::{ConnectInfo, Request},
    http::{HeaderMap, HeaderName, HeaderValue, StatusCode},
    response::{IntoResponse, Response},
    Json,
};
use dashmap::DashMap;
use serde_json::json;
use std::{
    net::SocketAddr,
    sync::Arc,
    time::{Duration, Instant},
};
use tower::{Layer, Service};
use tower_http::cors::{Any, CorsLayer};
use tracing::{error, info, warn};

pub struct SecurityState {
    pub ip_rate_limiter: Arc<DashMap<String, IpRateLimit>>,
    pub user_rate_limiter: Arc<DashMap<String, UserRateLimit>>,
    pub email_rate_limiter: Arc<DashMap<String, EmailRateLimit>>,
}

impl Default for SecurityState {
    fn default() -> Self {
        Self::new()
    }
}

impl SecurityState {
    pub fn new() -> Self {
        Self {
            ip_rate_limiter: Arc::new(DashMap::new()),
            user_rate_limiter: Arc::new(DashMap::new()),
            email_rate_limiter: Arc::new(DashMap::new()),
        }
    }
}

#[derive(Debug, Clone)]
pub struct IpRateLimit {
    pub registration_attempts: Vec<Instant>,
    pub login_attempts: Vec<Instant>,
}

impl IpRateLimit {
    pub fn new() -> Self {
        Self {
            registration_attempts: Vec::new(),
            login_attempts: Vec::new(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct UserRateLimit {
    pub refresh_attempts: Vec<Instant>,
}

impl UserRateLimit {
    pub fn new() -> Self {
        Self {
            refresh_attempts: Vec::new(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct EmailRateLimit {
    pub password_reset_attempts: Vec<Instant>,
}

impl EmailRateLimit {
    pub fn new() -> Self {
        Self {
            password_reset_attempts: Vec::new(),
        }
    }
}

pub fn check_registration_rate_limit(
    security_state: &SecurityState,
    ip: &str,
) -> Result<(), Response> {
    let mut ip_limit = security_state.ip_rate_limiter
        .entry(ip.to_string())
        .or_insert_with(IpRateLimit::new);

    let now = Instant::now();
    let one_hour_ago = now - Duration::from_secs(3600);

    ip_limit.registration_attempts.retain(|&time| time > one_hour_ago);

    if ip_limit.registration_attempts.len() >= 5 {
        warn!("Registration rate limit exceeded for IP: {}", ip);
        let response = Json(json!({
            "error": "Rate limit exceeded",
            "message": "Too many registration attempts. Please try again later.",
            "retry_after": 3600
        }));
        return Err((StatusCode::TOO_MANY_REQUESTS, response).into_response());
    }

    ip_limit.registration_attempts.push(now);
    Ok(())
}

pub fn check_login_rate_limit(
    security_state: &SecurityState,
    ip: &str,
) -> Result<(), Response> {
    let mut ip_limit = security_state.ip_rate_limiter
        .entry(ip.to_string())
        .or_insert_with(IpRateLimit::new);

    let now = Instant::now();
    let fifteen_minutes_ago = now - Duration::from_secs(900);

    ip_limit.login_attempts.retain(|&time| time > fifteen_minutes_ago);

    if ip_limit.login_attempts.len() >= 5 {
        warn!("Login rate limit exceeded for IP: {}", ip);
        let response = Json(json!({
            "error": "Rate limit exceeded",
            "message": "Too many login attempts. Please try again in 15 minutes.",
            "retry_after": 900
        }));
        return Err((StatusCode::TOO_MANY_REQUESTS, response).into_response());
    }

    ip_limit.login_attempts.push(now);
    Ok(())
}

pub fn check_refresh_rate_limit(
    security_state: &SecurityState,
    user_id: &str,
) -> Result<(), Response> {
    let mut user_limit = security_state.user_rate_limiter
        .entry(user_id.to_string())
        .or_insert_with(UserRateLimit::new);

    let now = Instant::now();
    let one_minute_ago = now - Duration::from_secs(60);

    user_limit.refresh_attempts.retain(|&time| time > one_minute_ago);

    if user_limit.refresh_attempts.len() >= 10 {
        warn!("Token refresh rate limit exceeded for user: {}", user_id);
        let response = Json(json!({
            "error": "Rate limit exceeded",
            "message": "Too many token refresh attempts. Please wait before retrying.",
            "retry_after": 60
        }));
        return Err((StatusCode::TOO_MANY_REQUESTS, response).into_response());
    }

    user_limit.refresh_attempts.push(now);
    Ok(())
}

pub fn check_password_reset_rate_limit(
    security_state: &SecurityState,
    email: &str,
) -> Result<(), Response> {
    let mut email_limit = security_state.email_rate_limiter
        .entry(email.to_string())
        .or_insert_with(EmailRateLimit::new);

    let now = Instant::now();
    let one_hour_ago = now - Duration::from_secs(3600);

    email_limit.password_reset_attempts.retain(|&time| time > one_hour_ago);

    if email_limit.password_reset_attempts.len() >= 3 {
        warn!("Password reset rate limit exceeded for email: {}", email);
        let response = Json(json!({
            "error": "Rate limit exceeded",
            "message": "Too many password reset requests. Please try again later.",
            "retry_after": 3600
        }));
        return Err((StatusCode::TOO_MANY_REQUESTS, response).into_response());
    }

    email_limit.password_reset_attempts.push(now);
    Ok(())
}

pub fn log_security_event(
    event_type: &str,
    ip: &str,
    user_agent: Option<&str>,
    user_id: Option<&str>,
    email: Option<&str>,
    success: bool,
    details: Option<&str>,
) {
    if success {
        info!(
            event_type = event_type,
            ip_address = ip,
            user_agent = user_agent,
            user_id = user_id,
            email = email,
            details = details,
            "Security event logged"
        );
    } else {
        warn!(
            event_type = event_type,
            ip_address = ip,
            user_agent = user_agent,
            user_id = user_id,
            email = email,
            details = details,
            "Failed security event logged"
        );
    }
}

pub fn get_cors_layer() -> CorsLayer {
    CorsLayer::new()
        .allow_origin([
            "http://localhost:3000".parse().unwrap(),
            "http://127.0.0.1:3000".parse().unwrap(),
            "https://localhost:3443".parse().unwrap(),
        ])
        .allow_methods([
            axum::http::Method::GET,
            axum::http::Method::POST,
            axum::http::Method::PUT,
            axum::http::Method::DELETE,
            axum::http::Method::OPTIONS,
        ])
        .allow_headers([
            axum::http::header::AUTHORIZATION,
            axum::http::header::CONTENT_TYPE,
            axum::http::header::ACCEPT,
        ])
        .allow_credentials(true)
        .max_age(Duration::from_secs(3600))
}

#[derive(Clone)]
pub struct SecurityHeadersLayer;

impl<S> Layer<S> for SecurityHeadersLayer {
    type Service = SecurityHeadersService<S>;

    fn layer(&self, inner: S) -> Self::Service {
        SecurityHeadersService { inner }
    }
}

#[derive(Clone)]
pub struct SecurityHeadersService<S> {
    inner: S,
}

impl<S> Service<Request> for SecurityHeadersService<S>
where
    S: Service<Request, Response = Response<Body>> + Clone + Send + 'static,
    S::Future: Send + 'static,
{
    type Response = S::Response;
    type Error = S::Error;
    type Future = std::pin::Pin<Box<dyn std::future::Future<Output = Result<Self::Response, Self::Error>> + Send>>;

    fn poll_ready(&mut self, cx: &mut std::task::Context<'_>) -> std::task::Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx)
    }

    fn call(&mut self, req: Request) -> Self::Future {
        let mut inner = self.inner.clone();

        Box::pin(async move {
            let mut response = inner.call(req).await?;

            let headers = response.headers_mut();

            headers.insert(
                HeaderName::from_static("x-content-type-options"),
                HeaderValue::from_static("nosniff"),
            );

            headers.insert(
                HeaderName::from_static("x-frame-options"),
                HeaderValue::from_static("DENY"),
            );

            headers.insert(
                HeaderName::from_static("x-xss-protection"),
                HeaderValue::from_static("1; mode=block"),
            );

            headers.insert(
                HeaderName::from_static("strict-transport-security"),
                HeaderValue::from_static("max-age=31536000; includeSubDomains; preload"),
            );

            headers.insert(
                HeaderName::from_static("content-security-policy"),
                HeaderValue::from_static("default-src 'self'; script-src 'self' 'unsafe-inline'; style-src 'self' 'unsafe-inline'; img-src 'self' data: https:; font-src 'self'; connect-src 'self'; frame-ancestors 'none';"),
            );

            headers.insert(
                HeaderName::from_static("referrer-policy"),
                HeaderValue::from_static("strict-origin-when-cross-origin"),
            );

            headers.insert(
                HeaderName::from_static("permissions-policy"),
                HeaderValue::from_static("geolocation=(), microphone=(), camera=()"),
            );

            Ok(response)
        })
    }
}

pub fn cleanup_rate_limiters(security_state: &SecurityState) {
    let now = Instant::now();
    let cleanup_threshold = Duration::from_secs(7200); // 2 hours

    security_state.ip_rate_limiter.retain(|_, limit| {
        limit.registration_attempts.retain(|&time| now.duration_since(time) < cleanup_threshold);
        limit.login_attempts.retain(|&time| now.duration_since(time) < cleanup_threshold);
        !limit.registration_attempts.is_empty() || !limit.login_attempts.is_empty()
    });

    security_state.user_rate_limiter.retain(|_, limit| {
        limit.refresh_attempts.retain(|&time| now.duration_since(time) < cleanup_threshold);
        !limit.refresh_attempts.is_empty()
    });

    security_state.email_rate_limiter.retain(|_, limit| {
        limit.password_reset_attempts.retain(|&time| now.duration_since(time) < cleanup_threshold);
        !limit.password_reset_attempts.is_empty()
    });
}