use axum::{
    body::Body,
    extract::connect_info::MockConnectInfo,
    http::{Request, StatusCode},
};
use chronos::app::middleware::security::{SecurityHeadersLayer, get_cors_layer};
use chronos::routes;
use dotenvy::dotenv;
use serde_json::{Value, json};
use sqlx::PgPool;
use std::env;
use std::net::SocketAddr;
use tower::ServiceBuilder;
use tower::ServiceExt;
use tower_http::{
    request_id::{MakeRequestUuid, PropagateRequestIdLayer, SetRequestIdLayer},
    trace::TraceLayer,
};
use uuid::Uuid;

async fn setup_test_pool() -> PgPool {
    dotenv().ok();
    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set for tests");

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
                "192.168.1.1:8080".parse::<SocketAddr>().unwrap(),
            )),
    )
}

async fn register_test_user(app: &axum::Router, email: &str, password: &str) -> (String, String) {
    let register_request = Request::builder()
        .uri("/api/auth/register")
        .method("POST")
        .header("content-type", "application/json")
        .body(Body::from(
            json!({
                "name": "Test User",
                "email": email,
                "password": password
            })
            .to_string(),
        ))
        .unwrap();

    let response = app.clone().oneshot(register_request).await.unwrap();
    assert_eq!(response.status(), StatusCode::CREATED);

    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let response_json: Value = serde_json::from_slice(&body).unwrap();

    let user_id = response_json["user"]["id"].as_str().unwrap().to_string();
    (user_id, email.to_string())
}

async fn login_test_user(app: &axum::Router, email: &str, password: &str) -> String {
    let login_request = Request::builder()
        .uri("/api/auth/login")
        .method("POST")
        .header("content-type", "application/json")
        .body(Body::from(
            json!({
                "email": email,
                "password": password
            })
            .to_string(),
        ))
        .unwrap();

    let response = app.clone().oneshot(login_request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let response_json: Value = serde_json::from_slice(&body).unwrap();

    response_json["tokens"]["access_token"]
        .as_str()
        .unwrap()
        .to_string()
}

#[tokio::test]
async fn test_get_profile_success() {
    let app = create_test_app().await;
    let test_email = "profileget@example.com";
    let test_password = "TestPass123!";

    // Register and login user
    let (_user_id, _email) = register_test_user(&app, test_email, test_password).await;
    let access_token = login_test_user(&app, test_email, test_password).await;

    // Get profile
    let profile_request = Request::builder()
        .uri("/api/auth/profile")
        .method("GET")
        .header("content-type", "application/json")
        .header("authorization", format!("Bearer {}", access_token))
        .body(Body::empty())
        .unwrap();

    let response = app.oneshot(profile_request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let response_json: Value = serde_json::from_slice(&body).unwrap();

    assert_eq!(response_json["email"].as_str().unwrap(), test_email);
    assert_eq!(response_json["name"].as_str().unwrap(), "Test User");
    assert!(response_json["id"].is_string());
    assert!(response_json["created_at"].is_string());
    // Ensure password_hash is not included
    assert!(response_json.get("password_hash").is_none());
}

#[tokio::test]
async fn test_get_profile_unauthorized() {
    let app = create_test_app().await;

    // Try to get profile without token
    let profile_request = Request::builder()
        .uri("/api/auth/profile")
        .method("GET")
        .header("content-type", "application/json")
        .body(Body::empty())
        .unwrap();

    let response = app.oneshot(profile_request).await.unwrap();
    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn test_update_profile_name_only() {
    let app = create_test_app().await;
    let test_email = "profileupdate1@example.com";
    let test_password = "TestPass123!";

    // Register and login user
    register_test_user(&app, test_email, test_password).await;
    let access_token = login_test_user(&app, test_email, test_password).await;

    // Update profile - name only
    let update_request = Request::builder()
        .uri("/api/auth/profile")
        .method("PUT")
        .header("content-type", "application/json")
        .header("authorization", format!("Bearer {}", access_token))
        .body(Body::from(
            json!({
                "name": "Updated Name"
            })
            .to_string(),
        ))
        .unwrap();

    let response = app.oneshot(update_request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let response_json: Value = serde_json::from_slice(&body).unwrap();

    assert_eq!(response_json["name"].as_str().unwrap(), "Updated Name");
    assert_eq!(response_json["email"].as_str().unwrap(), test_email);
}

#[tokio::test]
async fn test_update_profile_email_with_password() {
    let app = create_test_app().await;
    let test_email = "profileupdate2@example.com";
    let new_email = "updated2@example.com";
    let test_password = "TestPass123!";

    // Register and login user
    register_test_user(&app, test_email, test_password).await;
    let access_token = login_test_user(&app, test_email, test_password).await;

    // Update profile - email change with current password
    let update_request = Request::builder()
        .uri("/api/auth/profile")
        .method("PUT")
        .header("content-type", "application/json")
        .header("authorization", format!("Bearer {}", access_token))
        .body(Body::from(
            json!({
                "email": new_email,
                "current_password": test_password
            })
            .to_string(),
        ))
        .unwrap();

    let response = app.oneshot(update_request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let response_json: Value = serde_json::from_slice(&body).unwrap();

    assert_eq!(response_json["email"].as_str().unwrap(), new_email);
}

#[tokio::test]
async fn test_update_profile_email_without_password() {
    let app = create_test_app().await;
    let test_email = "profileupdate3@example.com";
    let new_email = "updated3@example.com";
    let test_password = "TestPass123!";

    // Register and login user
    register_test_user(&app, test_email, test_password).await;
    let access_token = login_test_user(&app, test_email, test_password).await;

    // Update profile - email change without current password (should fail)
    let update_request = Request::builder()
        .uri("/api/auth/profile")
        .method("PUT")
        .header("content-type", "application/json")
        .header("authorization", format!("Bearer {}", access_token))
        .body(Body::from(
            json!({
                "email": new_email
            })
            .to_string(),
        ))
        .unwrap();

    let response = app.oneshot(update_request).await.unwrap();
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);

    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let response_json: Value = serde_json::from_slice(&body).unwrap();

    assert!(
        response_json["error"]
            .as_str()
            .unwrap()
            .contains("Current password is required")
    );
}

#[tokio::test]
async fn test_update_profile_email_wrong_password() {
    let app = create_test_app().await;
    let test_email = "profileupdate4@example.com";
    let new_email = "updated4@example.com";
    let test_password = "TestPass123!";
    let wrong_password = "WrongPass123!";

    // Register and login user
    register_test_user(&app, test_email, test_password).await;
    let access_token = login_test_user(&app, test_email, test_password).await;

    // Update profile - email change with wrong password
    let update_request = Request::builder()
        .uri("/api/auth/profile")
        .method("PUT")
        .header("content-type", "application/json")
        .header("authorization", format!("Bearer {}", access_token))
        .body(Body::from(
            json!({
                "email": new_email,
                "current_password": wrong_password
            })
            .to_string(),
        ))
        .unwrap();

    let response = app.oneshot(update_request).await.unwrap();
    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);

    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let response_json: Value = serde_json::from_slice(&body).unwrap();

    assert!(
        response_json["error"]
            .as_str()
            .unwrap()
            .contains("Current password is incorrect")
    );
}

#[tokio::test]
async fn test_update_profile_invalid_email() {
    let app = create_test_app().await;
    let test_email = "profileupdate5@example.com";
    let test_password = "TestPass123!";

    // Register and login user
    register_test_user(&app, test_email, test_password).await;
    let access_token = login_test_user(&app, test_email, test_password).await;

    // Update profile - invalid email format
    let update_request = Request::builder()
        .uri("/api/auth/profile")
        .method("PUT")
        .header("content-type", "application/json")
        .header("authorization", format!("Bearer {}", access_token))
        .body(Body::from(
            json!({
                "email": "invalid-email-format"
            })
            .to_string(),
        ))
        .unwrap();

    let response = app.oneshot(update_request).await.unwrap();
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);

    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let response_json: Value = serde_json::from_slice(&body).unwrap();

    assert!(
        response_json["error"]
            .as_str()
            .unwrap()
            .contains("Validation failed")
    );
}

#[tokio::test]
async fn test_change_password_success() {
    let app = create_test_app().await;
    let test_email = "passwordchange1@example.com";
    let old_password = "OldPass123!";
    let new_password = "NewPass123!";

    // Register and login user
    register_test_user(&app, test_email, old_password).await;
    let access_token = login_test_user(&app, test_email, old_password).await;

    // Change password
    let change_request = Request::builder()
        .uri("/api/auth/change-password")
        .method("POST")
        .header("content-type", "application/json")
        .header("authorization", format!("Bearer {}", access_token))
        .body(Body::from(
            json!({
                "current_password": old_password,
                "new_password": new_password
            })
            .to_string(),
        ))
        .unwrap();

    let response = app.oneshot(change_request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let response_json: Value = serde_json::from_slice(&body).unwrap();

    assert!(
        response_json["message"]
            .as_str()
            .unwrap()
            .contains("Password changed successfully")
    );
}

#[tokio::test]
async fn test_change_password_wrong_current() {
    let app = create_test_app().await;
    let test_email = "passwordchange2@example.com";
    let correct_password = "CorrectPass123!";
    let wrong_password = "WrongPass123!";
    let new_password = "NewPass123!";

    // Register and login user
    register_test_user(&app, test_email, correct_password).await;
    let access_token = login_test_user(&app, test_email, correct_password).await;

    // Try to change password with wrong current password
    let change_request = Request::builder()
        .uri("/api/auth/change-password")
        .method("POST")
        .header("content-type", "application/json")
        .header("authorization", format!("Bearer {}", access_token))
        .body(Body::from(
            json!({
                "current_password": wrong_password,
                "new_password": new_password
            })
            .to_string(),
        ))
        .unwrap();

    let response = app.oneshot(change_request).await.unwrap();
    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);

    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let response_json: Value = serde_json::from_slice(&body).unwrap();

    assert!(
        response_json["error"]
            .as_str()
            .unwrap()
            .contains("Current password is incorrect")
    );
}

#[tokio::test]
async fn test_change_password_same_password() {
    let app = create_test_app().await;
    let test_email = "passwordchange3@example.com";
    let password = "SamePass123!";

    // Register and login user
    register_test_user(&app, test_email, password).await;
    let access_token = login_test_user(&app, test_email, password).await;

    // Try to change password to same password
    let change_request = Request::builder()
        .uri("/api/auth/change-password")
        .method("POST")
        .header("content-type", "application/json")
        .header("authorization", format!("Bearer {}", access_token))
        .body(Body::from(
            json!({
                "current_password": password,
                "new_password": password
            })
            .to_string(),
        ))
        .unwrap();

    let response = app.oneshot(change_request).await.unwrap();
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);

    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let response_json: Value = serde_json::from_slice(&body).unwrap();

    assert!(
        response_json["error"]
            .as_str()
            .unwrap()
            .contains("New password must be different")
    );
}

#[tokio::test]
async fn test_change_password_weak_new_password() {
    let app = create_test_app().await;
    let test_email = "passwordchange4@example.com";
    let current_password = "CurrentPass123!";
    let weak_password = "weak";

    // Register and login user
    register_test_user(&app, test_email, current_password).await;
    let access_token = login_test_user(&app, test_email, current_password).await;

    // Try to change to weak password
    let change_request = Request::builder()
        .uri("/api/auth/change-password")
        .method("POST")
        .header("content-type", "application/json")
        .header("authorization", format!("Bearer {}", access_token))
        .body(Body::from(
            json!({
                "current_password": current_password,
                "new_password": weak_password
            })
            .to_string(),
        ))
        .unwrap();

    let response = app.oneshot(change_request).await.unwrap();
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);

    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let response_json: Value = serde_json::from_slice(&body).unwrap();

    assert!(
        response_json["error"]
            .as_str()
            .unwrap()
            .contains("Validation failed")
    );
}

#[tokio::test]
async fn test_profile_endpoints_unauthorized() {
    let app = create_test_app().await;

    let requests = vec![
        ("GET", "/api/auth/profile", json!({})),
        ("PUT", "/api/auth/profile", json!({"name": "Test"})),
        (
            "POST",
            "/api/auth/change-password",
            json!({"current_password": "test", "new_password": "test"}),
        ),
    ];

    for (method, uri, body) in requests {
        let request = Request::builder()
            .uri(uri)
            .method(method)
            .header("content-type", "application/json")
            .body(Body::from(body.to_string()))
            .unwrap();

        let response = app.clone().oneshot(request).await.unwrap();
        assert_eq!(
            response.status(),
            StatusCode::UNAUTHORIZED,
            "Endpoint {} {} should require authentication",
            method,
            uri
        );
    }
}
