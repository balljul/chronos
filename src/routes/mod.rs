use axum::Router;
use sqlx::PgPool;
use crate::app::repositories::user_repository::UserRepository;
use crate::app::repositories::password_reset_repository::PasswordResetRepository;
use crate::app::repositories::token_blacklist_repository::TokenBlacklistRepository;
use crate::app::services::auth_service::AuthService;
use crate::app::services::email_service::MockEmailService;
use crate::app::services::jwt_service::{JwtService, get_jwt_secret};

pub mod users;
pub mod auth;

pub fn create_router(pool: PgPool) -> Router {
    let users_state = users::AppState::new(pool.clone());

    // Create auth state with JWT service
    let user_repository = UserRepository::new(pool.clone());
    let password_reset_repository = PasswordResetRepository::new(pool.clone());
    let token_blacklist_repository = TokenBlacklistRepository::new(pool);
    let email_service = MockEmailService::new();
    let auth_service = AuthService::new(user_repository, password_reset_repository, email_service);

    // Initialize JWT service
    let jwt_secret = get_jwt_secret();
    let jwt_service = JwtService::new(&jwt_secret, token_blacklist_repository);

    let auth_state = auth::AuthAppState::new(auth_service, jwt_service);

    Router::new()
        .nest("/api/users", users::routes().with_state(users_state))
        .nest("/api/auth", auth::routes().with_state(auth_state))
}
