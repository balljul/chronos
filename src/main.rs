mod app;
mod build;
mod routes;

#[tokio::main]
async fn main() {
    let _ = dotenvy::dotenv();
    let pool = build::postgres::build().await.expect("Failed to connect to Database");
    build::web::build(pool).await;
}
