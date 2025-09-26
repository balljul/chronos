use std::env;
use sqlx::PgPool;
use tokio::net::TcpListener;
use crate::routes;

pub async fn build(pool: PgPool) {
    let domain: String = env::var("APP_URL").expect("APP_URL must be set!");
    let port: String = env::var("APP_PORT").expect("APP_PORT must be set!");
    let url = format!("{}:{}", domain, port);

    let listener = TcpListener::bind(&url).await.unwrap();
    let app = routes::create_router(pool);
  let service = tower::make::Shared::new(app.into_service());
    
    println!("Server running on {}", &url);

    axum::serve(listener, service).await.unwrap();
}
