use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use serde_json::{json, Value};
use tower::ServiceExt;
use chronos::routes;
use sqlx::PgPool;
use std::env;

async fn setup_test_pool() -> PgPool {
    let database_url = env::var("DATABASE_URL")
        .expect("DATABASE_URL must be set for tests");

    sqlx::PgPool::connect(&database_url)
        .await
        .expect("Failed to connect to test database")
}

async fn create_test_app() -> axum::Router {
    let pool = setup_test_pool().await;
    routes::create_router(pool)
}

#[tokio::test]
async fn test_registration_rate_limiting_integration() {
    let app = create_test_app().await;

    let _test_email = "ratelimit@example.com";
    let test_ip = "192.168.100.1";

    // First 5 registration attempts should succeed (or fail with validation, not rate limiting)
    for i in 0..5 {
        let request = Request::builder()
            .uri("/api/auth/register")
            .method("POST")
            .header("content-type", "application/json")
            .header("x-forwarded-for", test_ip)
            .body(Body::from(
                json!({
                    "email": format!("user{}@example.com", i),
                    "password": "StrongP@ssw0rd123",
                    "name": "Test User"
                }).to_string()
            ))
            .unwrap();

        let response = app.clone().oneshot(request).await.unwrap();

        // Should not be rate limited (status should be 201 for success or 400/409 for validation/conflict)
        assert_ne!(response.status(), StatusCode::TOO_MANY_REQUESTS,
                  "Request {} should not be rate limited", i + 1);
    }

    // 6th attempt should be rate limited
    let request = Request::builder()
        .uri("/api/auth/register")
        .method("POST")
        .header("content-type", "application/json")
        .header("x-forwarded-for", test_ip)
        .body(Body::from(
            json!({
                "email": "user6@example.com",
                "password": "StrongP@ssw0rd123",
                "name": "Test User"
            }).to_string()
        ))
        .unwrap();

    let response = app.clone().oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::TOO_MANY_REQUESTS);
}

#[tokio::test]
async fn test_password_reset_rate_limiting_integration() {
    let app = create_test_app().await;

    let test_email = "resettest@example.com";

    // First 3 password reset attempts should succeed
    for _i in 0..3 {
        let request = Request::builder()
            .uri("/api/auth/forgot-password")
            .method("POST")
            .header("content-type", "application/json")
            .body(Body::from(
                json!({
                    "email": test_email
                }).to_string()
            ))
            .unwrap();

        let response = app.clone().oneshot(request).await.unwrap();

        // Should not be rate limited (status should be 200 for success or other non-429 for other errors)
        assert_ne!(response.status(), StatusCode::TOO_MANY_REQUESTS);
    }

    // 4th attempt should be rate limited
    let request = Request::builder()
        .uri("/api/auth/forgot-password")
        .method("POST")
        .header("content-type", "application/json")
        .body(Body::from(
            json!({
                "email": test_email
            }).to_string()
        ))
        .unwrap();

    let response = app.clone().oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::TOO_MANY_REQUESTS);
}

#[tokio::test]
async fn test_security_headers_applied() {
    let app = create_test_app().await;

    let request = Request::builder()
        .uri("/api/auth/register")
        .method("POST")
        .header("content-type", "application/json")
        .body(Body::from(
            json!({
                "email": "security@example.com",
                "password": "StrongP@ssw0rd123",
                "name": "Test User"
            }).to_string()
        ))
        .unwrap();

    let response = app.oneshot(request).await.unwrap();

    // Check that security headers are present
    let headers = response.headers();

    assert!(headers.contains_key("x-content-type-options"));
    assert_eq!(headers.get("x-content-type-options").unwrap(), "nosniff");

    assert!(headers.contains_key("x-frame-options"));
    assert_eq!(headers.get("x-frame-options").unwrap(), "DENY");

    assert!(headers.contains_key("x-xss-protection"));
    assert_eq!(headers.get("x-xss-protection").unwrap(), "1; mode=block");

    assert!(headers.contains_key("strict-transport-security"));
    assert!(headers.get("strict-transport-security").unwrap()
            .to_str().unwrap().contains("max-age=31536000"));

    assert!(headers.contains_key("content-security-policy"));
    assert!(headers.get("content-security-policy").unwrap()
            .to_str().unwrap().contains("default-src 'self'"));

    assert!(headers.contains_key("referrer-policy"));
    assert_eq!(headers.get("referrer-policy").unwrap(), "strict-origin-when-cross-origin");

    assert!(headers.contains_key("permissions-policy"));
    assert!(headers.get("permissions-policy").unwrap()
            .to_str().unwrap().contains("geolocation=()"));
}

#[tokio::test]
async fn test_cors_headers_applied() {
    let app = create_test_app().await;

    let request = Request::builder()
        .uri("/api/auth/register")
        .method("OPTIONS")
        .header("origin", "http://localhost:3000")
        .header("access-control-request-method", "POST")
        .header("access-control-request-headers", "content-type")
        .body(Body::empty())
        .unwrap();

    let response = app.oneshot(request).await.unwrap();

    // Check CORS headers
    let headers = response.headers();

    assert!(headers.contains_key("access-control-allow-origin"));
    assert!(headers.contains_key("access-control-allow-methods"));
    assert!(headers.contains_key("access-control-allow-headers"));
    assert!(headers.contains_key("access-control-allow-credentials"));
}

#[tokio::test]
async fn test_invalid_json_handling() {
    let app = create_test_app().await;

    let request = Request::builder()
        .uri("/api/auth/register")
        .method("POST")
        .header("content-type", "application/json")
        .body(Body::from("invalid json"))
        .unwrap();

    let response = app.oneshot(request).await.unwrap();

    // Should return 400 Bad Request for invalid JSON
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_missing_content_type_handling() {
    let app = create_test_app().await;

    let request = Request::builder()
        .uri("/api/auth/register")
        .method("POST")
        .body(Body::from(
            json!({
                "email": "test@example.com",
                "password": "StrongP@ssw0rd123"
            }).to_string()
        ))
        .unwrap();

    let response = app.oneshot(request).await.unwrap();

    // Should handle missing content-type gracefully
    assert_ne!(response.status(), StatusCode::INTERNAL_SERVER_ERROR);
}

#[tokio::test]
async fn test_user_agent_logging() {
    let app = create_test_app().await;

    let request = Request::builder()
        .uri("/api/auth/register")
        .method("POST")
        .header("content-type", "application/json")
        .header("user-agent", "TestAgent/1.0")
        .body(Body::from(
            json!({
                "email": "useragent@example.com",
                "password": "StrongP@ssw0rd123",
                "name": "Test User"
            }).to_string()
        ))
        .unwrap();

    let response = app.oneshot(request).await.unwrap();

    // The request should be processed (user agent is logged but doesn't affect processing)
    assert_ne!(response.status(), StatusCode::INTERNAL_SERVER_ERROR);
}

#[tokio::test]
async fn test_refresh_token_rotation_integration() {
    let app = create_test_app().await;

    // First, create a user and get tokens
    let register_request = Request::builder()
        .uri("/api/auth/register")
        .method("POST")
        .header("content-type", "application/json")
        .body(Body::from(
            json!({
                "email": "refreshtest@example.com",
                "password": "StrongP@ssw0rd123",
                "name": "Test User"
            }).to_string()
        ))
        .unwrap();

    let register_response = app.clone().oneshot(register_request).await.unwrap();

    if register_response.status() != StatusCode::CREATED {
        // User might already exist, try login instead
        let login_request = Request::builder()
            .uri("/api/auth/login")
            .method("POST")
            .header("content-type", "application/json")
            .body(Body::from(
                json!({
                    "email": "refreshtest@example.com",
                    "password": "StrongP@ssw0rd123"
                }).to_string()
            ))
            .unwrap();

        let login_response = app.clone().oneshot(login_request).await.unwrap();

        if login_response.status() == StatusCode::OK {
            let body_bytes = axum::body::to_bytes(login_response.into_body(), usize::MAX).await.unwrap();
            let login_data: Value = serde_json::from_slice(&body_bytes).unwrap();

            let refresh_token = login_data["tokens"]["refresh_token"].as_str().unwrap();

            // Test token refresh with rotation
            let refresh_request = Request::builder()
                .uri("/api/auth/refresh")
                .method("POST")
                .header("content-type", "application/json")
                .body(Body::from(
                    json!({
                        "refresh_token": refresh_token
                    }).to_string()
                ))
                .unwrap();

            let refresh_response = app.clone().oneshot(refresh_request).await.unwrap();

            if refresh_response.status() == StatusCode::OK {
                let body_bytes = axum::body::to_bytes(refresh_response.into_body(), usize::MAX).await.unwrap();
                let refresh_data: Value = serde_json::from_slice(&body_bytes).unwrap();

                // Verify response structure for token rotation
                assert!(refresh_data["access_token"].is_string());
                assert!(refresh_data["refresh_token"].is_string()); // Should have new refresh token
                assert_eq!(refresh_data["token_type"], "Bearer");
                assert!(refresh_data["expires_in"].is_number());
                assert!(refresh_data["refresh_expires_in"].is_number());

                // New refresh token should be different
                let new_refresh_token = refresh_data["refresh_token"].as_str().unwrap();
                assert_ne!(refresh_token, new_refresh_token);
            }
        }
    }
}

#[tokio::test]
async fn test_rate_limiting_with_different_ips() {
    let app = create_test_app().await;

    // Test that different IPs have separate rate limits
    let ip1 = "192.168.1.10";
    let ip2 = "192.168.1.20";

    // Exhaust rate limit for IP1
    for i in 0..5 {
        let request = Request::builder()
            .uri("/api/auth/register")
            .method("POST")
            .header("content-type", "application/json")
            .header("x-forwarded-for", ip1)
            .body(Body::from(
                json!({
                    "email": format!("user{}ip1@example.com", i),
                    "password": "StrongP@ssw0rd123",
                    "name": "Test User"
                }).to_string()
            ))
            .unwrap();

        let _response = app.clone().oneshot(request).await.unwrap();
    }

    // 6th request from IP1 should be rate limited
    let request = Request::builder()
        .uri("/api/auth/register")
        .method("POST")
        .header("content-type", "application/json")
        .header("x-forwarded-for", ip1)
        .body(Body::from(
            json!({
                "email": "user6ip1@example.com",
                "password": "StrongP@ssw0rd123",
                "name": "Test User"
            }).to_string()
        ))
        .unwrap();

    let response = app.clone().oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::TOO_MANY_REQUESTS);

    // But IP2 should still work
    let request = Request::builder()
        .uri("/api/auth/register")
        .method("POST")
        .header("content-type", "application/json")
        .header("x-forwarded-for", ip2)
        .body(Body::from(
            json!({
                "email": "user1ip2@example.com",
                "password": "StrongP@ssw0rd123",
                "name": "Test User"
            }).to_string()
        ))
        .unwrap();

    let response = app.clone().oneshot(request).await.unwrap();
    assert_ne!(response.status(), StatusCode::TOO_MANY_REQUESTS);
}

#[tokio::test]
async fn test_malformed_refresh_token() {
    let app = create_test_app().await;

    let request = Request::builder()
        .uri("/api/auth/refresh")
        .method("POST")
        .header("content-type", "application/json")
        .body(Body::from(
            json!({
                "refresh_token": "invalid.malformed.token"
            }).to_string()
        ))
        .unwrap();

    let response = app.oneshot(request).await.unwrap();

    // Should return 401 Unauthorized for invalid token
    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn test_empty_refresh_token() {
    let app = create_test_app().await;

    let request = Request::builder()
        .uri("/api/auth/refresh")
        .method("POST")
        .header("content-type", "application/json")
        .body(Body::from(
            json!({
                "refresh_token": ""
            }).to_string()
        ))
        .unwrap();

    let response = app.oneshot(request).await.unwrap();

    // Should return 401 Unauthorized for empty token
    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}