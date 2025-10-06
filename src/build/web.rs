use crate::app::middleware::security::{SecurityHeadersLayer, get_cors_layer};
use crate::routes;
use axum::extract::connect_info::ConnectInfo;
use sqlx::PgPool;
use std::env;
use std::net::SocketAddr;
use tokio::net::TcpListener;
use tower::ServiceBuilder;
use tower_http::{
    request_id::{MakeRequestUuid, PropagateRequestIdLayer, SetRequestIdLayer},
    trace::TraceLayer,
};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

pub async fn build(pool: PgPool) {
    // Initialize tracing
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "chronos=debug,tower_http=debug,axum::rejection=trace".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    let domain: String = env::var("APP_URL").expect("APP_URL must be set!");
    let port: String = env::var("APP_PORT").expect("APP_PORT must be set!");
    let url = format!("{}:{}", domain, port);

    let listener = TcpListener::bind(&url).await.unwrap();

    // Create the router
    let app = routes::create_router(pool);

    // Add security middleware layers
    let app = app.layer(
        ServiceBuilder::new()
            .layer(SetRequestIdLayer::x_request_id(MakeRequestUuid))
            .layer(PropagateRequestIdLayer::x_request_id())
            .layer(TraceLayer::new_for_http())
            .layer(SecurityHeadersLayer)
            .layer(get_cors_layer()),
    );

    println!("Server running on {}", &url);

    axum::serve(
        listener,
        app.into_make_service_with_connect_info::<SocketAddr>()
    ).await.unwrap();
}
