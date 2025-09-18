pub mod users;

use axum::Router;

pub fn create_router() -> Router {
    Router::new()
        .nest("/api/users", users::routes())
}
