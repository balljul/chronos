use axum::{routing::get, Router, Json};
use crate::app::services::user_service;

pub fn routes() -> Router {
    Router::new()
        .route("/", get(list_users))
}

async fn list_users() -> Json<serde_json::Value> {
    let result = user_service::test().expect("Failed the test method");
    Json(result)
}

