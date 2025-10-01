use chronos::app::models::jwt::{LoginRequest, LoginResponse};
use chronos::app::models::auth::AuthError;
use chronos::app::models::user::User;
use chronos::app::models::login_attempt::{LoginAttempt, AccountLockout, RefreshTokenStorage};
use chronos::app::services::secure_login_service::SecureLoginService;
use chronos::app::services::auth_service::AuthService;
use chronos::app::services::jwt_service::JwtService;
use chronos::app::repositories::user_repository::UserRepository;
use chronos::app::repositories::password_reset_repository::PasswordResetRepository;
use chronos::app::repositories::token_blacklist_repository::TokenBlacklistRepository;
use chronos::app::repositories::login_attempt_repository::{LoginAttemptRepository, AccountLockoutRepository, RefreshTokenRepository};
use chronos::app::services::email_service::MockEmailService;
use sqlx::{PgPool, postgres::PgPoolOptions};
use std::env;
use time::OffsetDateTime;
use uuid::Uuid;

async fn setup_test_pool() -> PgPool {
    let database_url = env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgresql://postgres:password@localhost:5432/chronos_test".to_string());

    PgPoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
        .await
        .expect("Failed to connect to test database")
}

async fn setup_secure_login_service(pool: PgPool) -> SecureLoginService {
    let user_repository = UserRepository::new(pool.clone());
    let password_reset_repository = PasswordResetRepository::new(pool.clone());
    let token_blacklist_repository = TokenBlacklistRepository::new(pool.clone());
    let login_attempt_repository = LoginAttemptRepository::new(pool.clone());
    let account_lockout_repository = AccountLockoutRepository::new(pool.clone());
    let refresh_token_repository = RefreshTokenRepository::new(pool.clone());
    let email_service = MockEmailService::new();

    let auth_service = AuthService::new(user_repository, password_reset_repository, email_service);
    let jwt_service = JwtService::new("test-secret-key", token_blacklist_repository, refresh_token_repository);

    SecureLoginService::new(
        auth_service,
        jwt_service,
        login_attempt_repository,
        account_lockout_repository,
    )
}

async fn create_test_user(pool: &PgPool) -> User {
    let user = User::new(
        Some("Test User".to_string()),
        "test@example.com".to_string(),
        "SecurePassword123!",
    ).expect("Failed to create test user");

    let user_repository = UserRepository::new(pool.clone());
    user_repository.create(&user).await.expect("Failed to save test user")
}

#[tokio::test]
async fn test_successful_login() {
    let pool = setup_test_pool().await;
    let service = setup_secure_login_service(pool.clone()).await;
    let user = create_test_user(&pool).await;

    let request = LoginRequest {
        email: user.email.clone(),
        password: "SecurePassword123!".to_string(),
    };

    let result = service.secure_login(
        request,
        "192.168.1.1".to_string(),
        Some("Test User Agent".to_string()),
    ).await;

    assert!(result.is_ok());
    let response = result.unwrap();
    assert_eq!(response.message, "Login successful");
    assert_eq!(response.user.email, user.email);
    assert!(!response.tokens.access_token.is_empty());
    assert!(!response.tokens.refresh_token.is_empty());

    // Cleanup
    let user_repository = UserRepository::new(pool);
    let _ = user_repository.delete(user.id).await;
}

#[tokio::test]
async fn test_invalid_credentials() {
    let pool = setup_test_pool().await;
    let service = setup_secure_login_service(pool.clone()).await;
    let user = create_test_user(&pool).await;

    let request = LoginRequest {
        email: user.email.clone(),
        password: "WrongPassword".to_string(),
    };

    let result = service.secure_login(
        request,
        "192.168.1.1".to_string(),
        Some("Test User Agent".to_string()),
    ).await;

    assert!(result.is_err());
    let error = result.unwrap_err();
    assert_eq!(error.error, "Invalid email or password");

    // Cleanup
    let user_repository = UserRepository::new(pool);
    let _ = user_repository.delete(user.id).await;
}

#[tokio::test]
async fn test_nonexistent_user() {
    let pool = setup_test_pool().await;
    let service = setup_secure_login_service(pool).await;

    let request = LoginRequest {
        email: "nonexistent@example.com".to_string(),
        password: "AnyPassword".to_string(),
    };

    let result = service.secure_login(
        request,
        "192.168.1.1".to_string(),
        Some("Test User Agent".to_string()),
    ).await;

    assert!(result.is_err());
    let error = result.unwrap_err();
    assert_eq!(error.error, "Invalid email or password");
}

#[tokio::test]
async fn test_ip_rate_limiting() {
    let pool = setup_test_pool().await;
    let service = setup_secure_login_service(pool.clone()).await;
    let user = create_test_user(&pool).await;

    let ip_address = "192.168.1.2".to_string();
    let user_agent = Some("Test User Agent".to_string());

    // Make 5 failed login attempts to trigger rate limiting
    for _ in 0..5 {
        let request = LoginRequest {
            email: user.email.clone(),
            password: "WrongPassword".to_string(),
        };

        let _ = service.secure_login(
            request,
            ip_address.clone(),
            user_agent.clone(),
        ).await;
    }

    // 6th attempt should be rate limited
    let request = LoginRequest {
        email: user.email.clone(),
        password: "WrongPassword".to_string(),
    };

    let result = service.secure_login(
        request,
        ip_address,
        user_agent,
    ).await;

    assert!(result.is_err());
    let error = result.unwrap_err();
    assert_eq!(error.error, "Too many failed login attempts. Please try again later.");

    // Cleanup
    let user_repository = UserRepository::new(pool);
    let _ = user_repository.delete(user.id).await;
}

#[tokio::test]
async fn test_account_lockout() {
    let pool = setup_test_pool().await;
    let service = setup_secure_login_service(pool.clone()).await;
    let user = create_test_user(&pool).await;

    let user_agent = Some("Test User Agent".to_string());

    // Make 10 failed login attempts to trigger account lockout
    for i in 0..10 {
        let request = LoginRequest {
            email: user.email.clone(),
            password: "WrongPassword".to_string(),
        };

        let result = service.secure_login(
            request,
            format!("192.168.1.{}", i + 10), // Different IPs to avoid IP rate limiting
            user_agent.clone(),
        ).await;

        // Should fail but not be locked until the 10th attempt
        assert!(result.is_err());
    }

    // 11th attempt should indicate account lockout
    let request = LoginRequest {
        email: user.email.clone(),
        password: "WrongPassword".to_string(),
    };

    let result = service.secure_login(
        request,
        "192.168.1.100".to_string(),
        user_agent,
    ).await;

    assert!(result.is_err());
    let error = result.unwrap_err();
    assert!(error.error.contains("Account has been temporarily locked"));

    // Cleanup
    let user_repository = UserRepository::new(pool);
    let _ = user_repository.delete(user.id).await;
}

#[tokio::test]
async fn test_locked_account_rejects_valid_credentials() {
    let pool = setup_test_pool().await;
    let service = setup_secure_login_service(pool.clone()).await;
    let user = create_test_user(&pool).await;

    // Create an account lockout
    let lockout = AccountLockout::new(user.id, 10, 30);
    let lockout_repository = AccountLockoutRepository::new(pool.clone());
    lockout_repository.create_lockout(&lockout).await.expect("Failed to create lockout");

    let request = LoginRequest {
        email: user.email.clone(),
        password: "SecurePassword123!".to_string(), // Correct password
    };

    let result = service.secure_login(
        request,
        "192.168.1.1".to_string(),
        Some("Test User Agent".to_string()),
    ).await;

    assert!(result.is_err());
    let error = result.unwrap_err();
    assert!(error.error.contains("Account is temporarily locked"));

    // Cleanup
    let user_repository = UserRepository::new(pool);
    let _ = user_repository.delete(user.id).await;
}

#[tokio::test]
async fn test_successful_login_unlocks_account() {
    let pool = setup_test_pool().await;
    let service = setup_secure_login_service(pool.clone()).await;
    let user = create_test_user(&pool).await;

    // Create an expired lockout (should be automatically unlocked)
    let past_time = OffsetDateTime::now_utc() - time::Duration::minutes(1);
    let mut lockout = AccountLockout::new(user.id, 10, 30);
    lockout.locked_until = past_time; // Already expired

    let lockout_repository = AccountLockoutRepository::new(pool.clone());
    lockout_repository.create_lockout(&lockout).await.expect("Failed to create lockout");

    let request = LoginRequest {
        email: user.email.clone(),
        password: "SecurePassword123!".to_string(),
    };

    let result = service.secure_login(
        request,
        "192.168.1.1".to_string(),
        Some("Test User Agent".to_string()),
    ).await;

    // Should succeed because lockout has expired
    assert!(result.is_ok());

    // Cleanup
    let user_repository = UserRepository::new(pool);
    let _ = user_repository.delete(user.id).await;
}

#[tokio::test]
async fn test_login_statistics_tracking() {
    let pool = setup_test_pool().await;
    let service = setup_secure_login_service(pool.clone()).await;
    let user = create_test_user(&pool).await;

    // Make a successful login
    let request = LoginRequest {
        email: user.email.clone(),
        password: "SecurePassword123!".to_string(),
    };

    let result = service.secure_login(
        request,
        "192.168.1.1".to_string(),
        Some("Test User Agent".to_string()),
    ).await;
    assert!(result.is_ok());

    // Make a failed login
    let request = LoginRequest {
        email: user.email.clone(),
        password: "WrongPassword".to_string(),
    };

    let _ = service.secure_login(
        request,
        "192.168.1.2".to_string(),
        Some("Test User Agent".to_string()),
    ).await;

    // Check statistics
    let stats = service.get_login_statistics(&user.email).await;
    assert!(stats.is_ok());

    let attempts = stats.unwrap();
    assert!(attempts.len() >= 2); // At least one success and one failure

    // Should have both successful and failed attempts
    let has_success = attempts.iter().any(|a| a.success);
    let has_failure = attempts.iter().any(|a| !a.success);
    assert!(has_success);
    assert!(has_failure);

    // Cleanup
    let user_repository = UserRepository::new(pool);
    let _ = user_repository.delete(user.id).await;
}

#[tokio::test]
async fn test_manual_account_unlock() {
    let pool = setup_test_pool().await;
    let service = setup_secure_login_service(pool.clone()).await;
    let user = create_test_user(&pool).await;

    // Create an active lockout
    let lockout = AccountLockout::new(user.id, 10, 30);
    let lockout_repository = AccountLockoutRepository::new(pool.clone());
    lockout_repository.create_lockout(&lockout).await.expect("Failed to create lockout");

    // Unlock the account manually
    let unlock_result = service.unlock_account(user.id).await;
    assert!(unlock_result.is_ok());

    // Should now be able to login with correct credentials
    let request = LoginRequest {
        email: user.email.clone(),
        password: "SecurePassword123!".to_string(),
    };

    let result = service.secure_login(
        request,
        "192.168.1.1".to_string(),
        Some("Test User Agent".to_string()),
    ).await;

    assert!(result.is_ok());

    // Cleanup
    let user_repository = UserRepository::new(pool);
    let _ = user_repository.delete(user.id).await;
}