mod app;
mod build;

#[tokio::main]
async fn main() {
    let _ = dotenvy::dotenv();
    build::web::build().await;
}
