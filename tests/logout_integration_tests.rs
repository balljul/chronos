use axum::{
    body::Body,
    http::{Request, StatusCode, HeaderValue},
    extract::connect_info::MockConnectInfo,
};
use serde_json::{json, Value};
use tower::ServiceExt;
use tower::ServiceBuilder;
use tower_http::{
    trace::TraceLayer,
    request_id::{MakeRequestUuid, PropagateRequestIdLayer, SetRequestIdLayer},
};
use chronos::routes;
use chronos::app::middleware::security::{SecurityHeadersLayer, get_cors_layer};
use sqlx::PgPool;
use std::env;
use std::net::SocketAddr;
use dotenvy::dotenv;

async fn setup_test_pool() -> PgPool {
    dotenv().ok();
    let database_url = env::var("DATABASE_URL")
        .expect("DATABASE_URL must be set for tests");

    sqlx::PgPool::connect(&database_url)
        .await
        .expect("Failed to connect to test database")
}

async fn create_test_app() -> axum::Router {
    let pool = setup_test_pool().await;
    let app = routes::create_router(pool);

    // Add security middleware layers like in the main app
    app.layer(
        ServiceBuilder::new()
            .layer(SetRequestIdLayer::x_request_id(MakeRequestUuid))
            .layer(PropagateRequestIdLayer::x_request_id())
            .layer(TraceLayer::new_for_http())
            .layer(SecurityHeadersLayer)
            .layer(get_cors_layer())
            .layer(MockConnectInfo(
                "192.168.1.1:8080".parse::<SocketAddr>().unwrap()
            ))
    )
}

async fn register_test_user(app: &axum::Router, email: &str) -> Result<Value, Box<dyn std::error::Error + Send + Sync>> {
    let register_request = Request::builder()
        .uri("/api/auth/register")
        .method("POST")
        .header("content-type", "application/json")
        .body(Body::from(
            json!({
                "email": email,
                "password": "StrongP@ssw0rd123",
                "name": "Test User"
            }).to_string()
        ))?;

    let response = app.clone().oneshot(register_request).await?;

    if response.status() == StatusCode::CREATED {
        let body_bytes = axum::body::to_bytes(response.into_body(), usize::MAX).await?;
        let register_data: Value = serde_json::from_slice(&body_bytes)?;
        Ok(register_data)
    } else {
        Err("Registration failed".into())
    }
}

async fn login_test_user(app: &axum::Router, email: &str) -> Result<Value, Box<dyn std::error::Error + Send + Sync>> {
    let login_request = Request::builder()
        .uri("/api/auth/login")
        .method("POST")
        .header("content-type", "application/json")
        .body(Body::from(
            json!({
                "email": email,
                "password": "StrongP@ssw0rd123"
            }).to_string()
        ))?;

    let response = app.clone().oneshot(login_request).await?;

    if response.status() == StatusCode::OK {
        let body_bytes = axum::body::to_bytes(response.into_body(), usize::MAX).await?;
        let login_data: Value = serde_json::from_slice(&body_bytes)?;
        Ok(login_data)
    } else {
        Err("Login failed".into())
    }
}

#[tokio::test]
async fn test_logout_with_valid_token_integration() {
    let app = create_test_app().await;
    let test_email = "logout_valid@example.com";

    // Register and login to get tokens
    if register_test_user(&app, test_email).await.is_err() {
        // User might already exist, try login directly
    }

    let login_data = login_test_user(&app, test_email).await
        .expect("Should be able to login");

    let access_token = login_data["tokens"]["access_token"]
        .as_str()
        .expect("Should have access token");

    let refresh_token = login_data["tokens"]["refresh_token"]
        .as_str()
        .expect("Should have refresh token");

    // Test logout with valid tokens
    let logout_request = Request::builder()
        .uri("/api/auth/logout")
        .method("POST")
        .header("content-type", "application/json")
        .header("authorization", format!("Bearer {}", access_token))
        .body(Body::from(
            json!({
                "refresh_token": refresh_token,
                "logout_all_devices": false
            }).to_string()
        ))
        .unwrap();

    let response = app.clone().oneshot(logout_request).await.unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body_bytes = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let logout_data: Value = serde_json::from_slice(&body_bytes).unwrap();

    assert_eq!(logout_data["message"], "Logged out successfully");
    // Should have logged out at least 1 device (the current one)
    if let Some(devices) = logout_data["logged_out_devices"].as_u64() {
        assert!(devices >= 1);
    }
}

#[tokio::test]
async fn test_logout_all_devices_integration() {
    let app = create_test_app().await;
    let test_email = "logout_all@example.com";

    // Register and login to get tokens
    if register_test_user(&app, test_email).await.is_err() {
        // User might already exist, try login directly
    }

    let login_data = login_test_user(&app, test_email).await
        .expect("Should be able to login");

    let access_token = login_data["tokens"]["access_token"]
        .as_str()
        .expect("Should have access token");

    let refresh_token = login_data["tokens"]["refresh_token"]
        .as_str()
        .expect("Should have refresh token");

    // Test logout all devices
    let logout_request = Request::builder()
        .uri("/api/auth/logout")
        .method("POST")
        .header("content-type", "application/json")
        .header("authorization", format!("Bearer {}", access_token))
        .body(Body::from(
            json!({
                "refresh_token": refresh_token,
                "logout_all_devices": true
            }).to_string()
        ))
        .unwrap();

    let response = app.clone().oneshot(logout_request).await.unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body_bytes = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let logout_data: Value = serde_json::from_slice(&body_bytes).unwrap();

    assert_eq!(logout_data["message"], "Logged out successfully");
    // For "logout all devices", we don't expose the actual count
    assert!(logout_data["logged_out_devices"].is_null());
}

#[tokio::test]
async fn test_logout_without_token_integration() {
    let app = create_test_app().await;

    // Test logout without authorization header (should still return success)
    let logout_request = Request::builder()
        .uri("/api/auth/logout")
        .method("POST")
        .header("content-type", "application/json")
        .body(Body::from(
            json!({
                "logout_all_devices": false
            }).to_string()
        ))
        .unwrap();

    let response = app.clone().oneshot(logout_request).await.unwrap();

    // Should return success to prevent information leakage
    assert_eq!(response.status(), StatusCode::OK);

    let body_bytes = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let logout_data: Value = serde_json::from_slice(&body_bytes).unwrap();

    assert_eq!(logout_data["message"], "Logged out successfully");
    assert!(logout_data["logged_out_devices"].is_null());
}

#[tokio::test]
async fn test_logout_with_invalid_token_integration() {
    let app = create_test_app().await;

    // Test logout with completely invalid token
    let logout_request = Request::builder()
        .uri("/api/auth/logout")
        .method("POST")
        .header("content-type", "application/json")
        .header("authorization", "Bearer invalid.token.here")
        .body(Body::from(
            json!({
                "logout_all_devices": false
            }).to_string()
        ))
        .unwrap();

    let response = app.clone().oneshot(logout_request).await.unwrap();

    // Should return success even with invalid token (prevents information leakage)
    assert_eq!(response.status(), StatusCode::OK);

    let body_bytes = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let logout_data: Value = serde_json::from_slice(&body_bytes).unwrap();

    assert_eq!(logout_data["message"], "Logged out successfully");
}

#[tokio::test]
async fn test_logout_with_malformed_authorization_header_integration() {
    let app = create_test_app().await;

    // Test logout with malformed authorization header
    let logout_request = Request::builder()
        .uri("/api/auth/logout")
        .method("POST")
        .header("content-type", "application/json")
        .header("authorization", "InvalidFormat token")
        .body(Body::from(
            json!({
                "logout_all_devices": false
            }).to_string()
        ))
        .unwrap();

    let response = app.clone().oneshot(logout_request).await.unwrap();

    // Should handle malformed header gracefully and return error
    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn test_logout_token_becomes_unusable_integration() {
    let app = create_test_app().await;
    let test_email = "logout_unusable@example.com";

    // Register and login to get tokens
    if register_test_user(&app, test_email).await.is_err() {
        // User might already exist, try login directly
    }

    let login_data = login_test_user(&app, test_email).await
        .expect("Should be able to login");

    let access_token = login_data["tokens"]["access_token"]
        .as_str()
        .expect("Should have access token");

    let refresh_token = login_data["tokens"]["refresh_token"]
        .as_str()
        .expect("Should have refresh token");

    // First, verify the access token works (if we had a protected endpoint)
    // For now, we'll just verify it can be used for logout

    // Perform logout
    let logout_request = Request::builder()
        .uri("/api/auth/logout")
        .method("POST")
        .header("content-type", "application/json")
        .header("authorization", format!("Bearer {}", access_token))
        .body(Body::from(
            json!({
                "refresh_token": refresh_token,
                "logout_all_devices": false
            }).to_string()
        ))
        .unwrap();

    let response = app.clone().oneshot(logout_request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    // Try to use the refresh token again (should fail)
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

    // Refresh should fail since the token was revoked during logout
    assert_eq!(refresh_response.status(), StatusCode::UNAUTHORIZED);

    // Try to logout again with the same access token (should still succeed for security)
    let logout_again_request = Request::builder()
        .uri("/api/auth/logout")
        .method("POST")
        .header("content-type", "application/json")
        .header("authorization", format!("Bearer {}", access_token))
        .body(Body::from(
            json!({
                "logout_all_devices": false
            }).to_string()
        ))
        .unwrap();

    let logout_again_response = app.clone().oneshot(logout_again_request).await.unwrap();

    // Should still return success (idempotent and secure)
    assert_eq!(logout_again_response.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_logout_with_different_user_agents_integration() {
    let app = create_test_app().await;
    let test_email = "logout_agents@example.com";

    // Register and login to get tokens
    if register_test_user(&app, test_email).await.is_err() {
        // User might already exist, try login directly
    }

    let login_data = login_test_user(&app, test_email).await
        .expect("Should be able to login");

    let access_token = login_data["tokens"]["access_token"]
        .as_str()
        .expect("Should have access token");

    // Test logout with different user agents (should all work)
    let user_agents = vec![
        "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36",
        "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36",
        "Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36",
        "MyApp/1.0 (iOS)",
        "MyApp/1.0 (Android)",
    ];

    for (i, user_agent) in user_agents.iter().enumerate() {
        let logout_request = Request::builder()
            .uri("/api/auth/logout")
            .method("POST")
            .header("content-type", "application/json")
            .header("authorization", format!("Bearer {}", access_token))
            .header("user-agent", *user_agent)
            .body(Body::from(
                json!({
                    "logout_all_devices": false
                }).to_string()
            ))
            .unwrap();

        let response = app.clone().oneshot(logout_request).await.unwrap();

        // All should succeed (first one actually logs out, subsequent ones are idempotent)
        assert_eq!(response.status(), StatusCode::OK, "Failed for user agent {}: {}", i, user_agent);
    }
}

#[tokio::test]
async fn test_logout_minimal_request_integration() {
    let app = create_test_app().await;
    let test_email = "logout_minimal@example.com";

    // Register and login to get tokens
    if register_test_user(&app, test_email).await.is_err() {
        // User might already exist, try login directly
    }

    let login_data = login_test_user(&app, test_email).await
        .expect("Should be able to login");

    let access_token = login_data["tokens"]["access_token"]
        .as_str()
        .expect("Should have access token");

    // Test logout with minimal request (no refresh token, no logout_all_devices)
    let logout_request = Request::builder()
        .uri("/api/auth/logout")
        .method("POST")
        .header("content-type", "application/json")
        .header("authorization", format!("Bearer {}", access_token))
        .body(Body::from("{}"))
        .unwrap();

    let response = app.clone().oneshot(logout_request).await.unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body_bytes = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let logout_data: Value = serde_json::from_slice(&body_bytes).unwrap();

    assert_eq!(logout_data["message"], "Logged out successfully");
    // Should have logged out at least the current device
    if let Some(devices) = logout_data["logged_out_devices"].as_u64() {
        assert!(devices >= 1);
    }
}

#[tokio::test]
async fn test_logout_response_headers_integration() {
    let app = create_test_app().await;

    // Test logout request to verify security headers are applied
    let logout_request = Request::builder()
        .uri("/api/auth/logout")
        .method("POST")
        .header("content-type", "application/json")
        .body(Body::from("{}"))
        .unwrap();

    let response = app.oneshot(logout_request).await.unwrap();

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
}

#[tokio::test]
async fn test_logout_with_invalid_json_integration() {
    let app = create_test_app().await;
    let test_email = "logout_invalid_json@example.com";

    // Register and login to get tokens
    if register_test_user(&app, test_email).await.is_err() {
        // User might already exist, try login directly
    }

    let login_data = login_test_user(&app, test_email).await
        .expect("Should be able to login");

    let access_token = login_data["tokens"]["access_token"]
        .as_str()
        .expect("Should have access token");

    // Test logout with invalid JSON
    let logout_request = Request::builder()
        .uri("/api/auth/logout")
        .method("POST")
        .header("content-type", "application/json")
        .header("authorization", format!("Bearer {}", access_token))
        .body(Body::from("invalid json"))
        .unwrap();

    let response = app.oneshot(logout_request).await.unwrap();

    // Should return 400 Bad Request for invalid JSON
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_logout_performance_integration() {
    let app = create_test_app().await;
    let test_email = "logout_perf@example.com";

    // Register and login to get tokens
    if register_test_user(&app, test_email).await.is_err() {
        // User might already exist, try login directly
    }

    let login_data = login_test_user(&app, test_email).await
        .expect("Should be able to login");

    let access_token = login_data["tokens"]["access_token"]
        .as_str()
        .expect("Should have access token");

    // Measure logout performance
    let start = std::time::Instant::now();

    let logout_request = Request::builder()
        .uri("/api/auth/logout")
        .method("POST")
        .header("content-type", "application/json")
        .header("authorization", format!("Bearer {}", access_token))
        .body(Body::from(
            json!({
                "logout_all_devices": true
            }).to_string()
        ))
        .unwrap();

    let response = app.oneshot(logout_request).await.unwrap();

    let duration = start.elapsed();

    assert_eq!(response.status(), StatusCode::OK);

    // Logout should complete within reasonable time (5 seconds is generous for tests)
    assert!(duration.as_secs() < 5, "Logout took too long: {:?}", duration);
}