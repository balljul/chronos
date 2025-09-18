mod app;
mod build;
mod routes;

#[tokio::main]
async fn main() {
    let _ = dotenvy::dotenv();
    build::postgres::build().await;
    build::web::build().await;
}
