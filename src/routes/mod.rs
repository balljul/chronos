use crate::app::middleware::auth_middleware::jwt_auth_middleware_with_json_errors;
use crate::app::middleware::security::SecurityState;
use crate::app::repositories::login_attempt_repository::{
    AccountLockoutRepository, LoginAttemptRepository, RefreshTokenRepository,
};
use crate::app::repositories::password_reset_repository::PasswordResetRepository;
use crate::app::repositories::token_blacklist_repository::TokenBlacklistRepository;
use crate::app::repositories::user_repository::UserRepository;
use crate::app::services::auth_service::AuthService;
use crate::app::services::email_service::MockEmailService;
use crate::app::services::jwt_service::{JwtService, get_jwt_secret};
use crate::app::services::secure_login_service::SecureLoginService;
use crate::app::services::user_service::UserService;
use axum::{Router, middleware};
use sqlx::PgPool;

pub mod auth;
pub mod users;

pub fn create_router(pool: PgPool) -> Router {
    let users_state = users::AppState::new(pool.clone());

    let user_repository = UserRepository::new(pool.clone());
    let password_reset_repository = PasswordResetRepository::new(pool.clone());
    let token_blacklist_repository = TokenBlacklistRepository::new(pool.clone());
    let login_attempt_repository = LoginAttemptRepository::new(pool.clone());
    let account_lockout_repository = AccountLockoutRepository::new(pool.clone());
    let refresh_token_repository = RefreshTokenRepository::new(pool);

    // Create services
    let email_service = MockEmailService::new();
    let user_service = UserService::new(user_repository.clone());
    let auth_service = AuthService::new(user_repository, password_reset_repository, email_service);

    // Initialize JWT service with refresh token repository
    let jwt_secret = get_jwt_secret();
    let jwt_service = JwtService::new(
        &jwt_secret,
        token_blacklist_repository,
        refresh_token_repository,
    );

    // Create secure login service
    let secure_login_service = SecureLoginService::new(
        auth_service.clone(), // Note: You might need to implement Clone for AuthService
        jwt_service.clone(),  // Note: You might need to implement Clone for JwtService
        login_attempt_repository,
        account_lockout_repository,
    );

    // Create security state for rate limiting
    let security_state = SecurityState::new();

    let auth_state = auth::AuthAppState::new(
        auth_service,
        jwt_service.clone(),
        secure_login_service,
        user_service,
        security_state,
    );

    // Create public auth routes (no middleware)
    let public_auth_routes = auth::routes().with_state(auth_state.clone());

    // Create protected auth routes with middleware
    let protected_auth_routes =
        auth::protected_routes()
            .with_state(auth_state)
            .layer(middleware::from_fn_with_state(
                std::sync::Arc::new(jwt_service),
                jwt_auth_middleware_with_json_errors,
            ));

    Router::new()
        .nest("/api/users", users::routes().with_state(users_state))
        .nest("/api/auth", public_auth_routes)
        .nest("/api/auth", protected_auth_routes)
}
