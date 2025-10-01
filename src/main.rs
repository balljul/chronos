#[cfg(feature = "server")]
mod app;
#[cfg(feature = "server")]
mod build;
#[cfg(feature = "server")]
mod routes;

#[cfg(feature = "server")]
#[tokio::main]
async fn main() {
    let _ = dotenvy::dotenv();
    let pool = build::postgres::build()
        .await
        .expect("Failed to connect to Database");
    build::web::build(pool).await;
}
