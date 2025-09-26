use axum::Router;
use sqlx::PgPool;

pub mod users;

pub fn create_router(pool: PgPool) -> Router {
    let state = users::AppState::new(pool);
    Router::new()
        .nest("/api/users", users::routes())
        .with_state(state)
}
