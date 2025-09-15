use axum::{
    routing::get,
    Router
};
use std::env;

pub async fn build() {
    let app = Router::new().route("/", get(|| async { "Hello, World!" }));
    
    let domain: String = env::var("APP_URL").expect("APP_URL must be set!");
    let port: String = env::var("APP_PORT").expect("APP_PORT must be set!");
    let url = format!("{}:{}", domain, port);

    let listener = tokio::net::TcpListener::bind(&url).await.unwrap();
    println!("Server running on {}", &url);

    axum::serve(listener, app).await.unwrap();
}
