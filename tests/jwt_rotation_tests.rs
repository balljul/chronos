use chronos::app::models::jwt::{Claims, TokenType, RefreshTokenResponse};
use chronos::app::models::user::User;
use chronos::app::services::jwt_service::JwtService;
use chronos::app::repositories::token_blacklist_repository::TokenBlacklistRepository;
use chronos::app::repositories::login_attempt_repository::RefreshTokenRepository;
use sqlx::PgPool;
use uuid::Uuid;
use time::OffsetDateTime;
use std::env;

async fn setup_test_pool() -> PgPool {
    let database_url = env::var("DATABASE_URL")
        .expect("DATABASE_URL must be set for tests");

    sqlx::PgPool::connect(&database_url)
        .await
        .expect("Failed to connect to test database")
}

async fn create_test_user() -> User {
    User {
        id: Uuid::new_v4(),
        name: Some("Test User".to_string()),
        email: "test@example.com".to_string(),
        password_hash: "dummy_hash".to_string(),
        created_at: Some(OffsetDateTime::now_utc()),
        updated_at: Some(OffsetDateTime::now_utc()),
    }
}

#[tokio::test]
async fn test_token_pair_generation() {
    let pool = setup_test_pool().await;
    let blacklist_repo = TokenBlacklistRepository::new(pool.clone());
    let refresh_repo = RefreshTokenRepository::new(pool);

    let jwt_service = JwtService::new("test_secret_key_at_least_256_bits_long", blacklist_repo, refresh_repo);
    let test_user = create_test_user().await;

    let token_pair = jwt_service.generate_token_pair(&test_user)
        .await
        .expect("Should generate token pair successfully");

    // Verify token pair structure
    assert!(!token_pair.access_token.is_empty());
    assert!(!token_pair.refresh_token.is_empty());
    assert_eq!(token_pair.token_type, "Bearer");
    assert_eq!(token_pair.expires_in, 15 * 60); // 15 minutes
    assert_eq!(token_pair.refresh_expires_in, 7 * 24 * 60 * 60); // 7 days

    // Verify tokens have different content
    assert_ne!(token_pair.access_token, token_pair.refresh_token);
}

#[tokio::test]
async fn test_access_token_validation() {
    let pool = setup_test_pool().await;
    let blacklist_repo = TokenBlacklistRepository::new(pool.clone());
    let refresh_repo = RefreshTokenRepository::new(pool);

    let jwt_service = JwtService::new("test_secret_key_at_least_256_bits_long", blacklist_repo, refresh_repo);
    let test_user = create_test_user().await;

    let token_pair = jwt_service.generate_token_pair(&test_user)
        .await
        .expect("Should generate token pair successfully");

    // Validate access token
    let access_claims = jwt_service.validate_token(&token_pair.access_token)
        .await
        .expect("Access token should be valid");

    assert_eq!(access_claims.sub, test_user.id.to_string());
    assert_eq!(access_claims.email, test_user.email);
    assert!(matches!(access_claims.token_type, TokenType::Access));

    // Validate refresh token
    let refresh_claims = jwt_service.validate_token(&token_pair.refresh_token)
        .await
        .expect("Refresh token should be valid");

    assert_eq!(refresh_claims.sub, test_user.id.to_string());
    assert_eq!(refresh_claims.email, test_user.email);
    assert!(matches!(refresh_claims.token_type, TokenType::Refresh));
}

#[tokio::test]
async fn test_token_rotation() {
    let pool = setup_test_pool().await;
    let blacklist_repo = TokenBlacklistRepository::new(pool.clone());
    let refresh_repo = RefreshTokenRepository::new(pool);

    let jwt_service = JwtService::new("test_secret_key_at_least_256_bits_long", blacklist_repo, refresh_repo);
    let test_user = create_test_user().await;

    // Generate initial token pair
    let initial_tokens = jwt_service.generate_token_pair(&test_user)
        .await
        .expect("Should generate initial token pair");

    // Use refresh token to get new tokens with rotation
    let new_tokens = jwt_service.refresh_with_rotation(&initial_tokens.refresh_token, test_user.id)
        .await
        .expect("Should refresh tokens with rotation");

    // Verify new tokens are different from initial tokens
    assert_ne!(initial_tokens.access_token, new_tokens.access_token);
    assert_ne!(initial_tokens.refresh_token, new_tokens.refresh_token);

    // Verify new access token is valid
    let new_access_claims = jwt_service.validate_token(&new_tokens.access_token)
        .await
        .expect("New access token should be valid");

    assert_eq!(new_access_claims.sub, test_user.id.to_string());
    assert_eq!(new_access_claims.email, test_user.email);

    // Verify new refresh token is valid
    let new_refresh_claims = jwt_service.validate_token(&new_tokens.refresh_token)
        .await
        .expect("New refresh token should be valid");

    assert_eq!(new_refresh_claims.sub, test_user.id.to_string());
    assert_eq!(new_refresh_claims.email, test_user.email);
}

#[tokio::test]
async fn test_old_refresh_token_invalidated_after_rotation() {
    let pool = setup_test_pool().await;
    let blacklist_repo = TokenBlacklistRepository::new(pool.clone());
    let refresh_repo = RefreshTokenRepository::new(pool);

    let jwt_service = JwtService::new("test_secret_key_at_least_256_bits_long", blacklist_repo, refresh_repo);
    let test_user = create_test_user().await;

    // Generate initial token pair
    let initial_tokens = jwt_service.generate_token_pair(&test_user)
        .await
        .expect("Should generate initial token pair");

    // Use refresh token to get new tokens with rotation
    let _new_tokens = jwt_service.refresh_with_rotation(&initial_tokens.refresh_token, test_user.id)
        .await
        .expect("Should refresh tokens with rotation");

    // Try to use the old refresh token again - should fail
    let result = jwt_service.refresh_with_rotation(&initial_tokens.refresh_token, test_user.id).await;
    assert!(result.is_err(), "Old refresh token should be invalidated");
}

#[tokio::test]
async fn test_refresh_token_without_rotation() {
    let pool = setup_test_pool().await;
    let blacklist_repo = TokenBlacklistRepository::new(pool.clone());
    let refresh_repo = RefreshTokenRepository::new(pool);

    let jwt_service = JwtService::new("test_secret_key_at_least_256_bits_long", blacklist_repo, refresh_repo);
    let test_user = create_test_user().await;

    // Generate initial token pair
    let initial_tokens = jwt_service.generate_token_pair(&test_user)
        .await
        .expect("Should generate initial token pair");

    // Use basic refresh (without rotation)
    let new_access_token = jwt_service.refresh_access_token(&initial_tokens.refresh_token)
        .await
        .expect("Should refresh access token");

    // Verify new access token is different from initial
    assert_ne!(initial_tokens.access_token, new_access_token);

    // Verify new access token is valid
    let new_access_claims = jwt_service.validate_token(&new_access_token)
        .await
        .expect("New access token should be valid");

    assert_eq!(new_access_claims.sub, test_user.id.to_string());
    assert_eq!(new_access_claims.email, test_user.email);
    assert!(matches!(new_access_claims.token_type, TokenType::Access));

    // Original refresh token should still be usable (no rotation)
    let another_access_token = jwt_service.refresh_access_token(&initial_tokens.refresh_token)
        .await
        .expect("Original refresh token should still work");

    assert_ne!(new_access_token, another_access_token);
}

#[tokio::test]
async fn test_invalid_refresh_token() {
    let pool = setup_test_pool().await;
    let blacklist_repo = TokenBlacklistRepository::new(pool.clone());
    let refresh_repo = RefreshTokenRepository::new(pool);

    let jwt_service = JwtService::new("test_secret_key_at_least_256_bits_long", blacklist_repo, refresh_repo);

    // Try to refresh with invalid token
    let result = jwt_service.refresh_access_token("invalid.token.here").await;
    assert!(result.is_err(), "Invalid token should fail validation");

    // Try rotation with invalid token
    let result = jwt_service.refresh_with_rotation("invalid.token.here", Uuid::new_v4()).await;
    assert!(result.is_err(), "Invalid token should fail rotation");
}

#[tokio::test]
async fn test_access_token_used_as_refresh_token() {
    let pool = setup_test_pool().await;
    let blacklist_repo = TokenBlacklistRepository::new(pool.clone());
    let refresh_repo = RefreshTokenRepository::new(pool);

    let jwt_service = JwtService::new("test_secret_key_at_least_256_bits_long", blacklist_repo, refresh_repo);
    let test_user = create_test_user().await;

    // Generate token pair
    let tokens = jwt_service.generate_token_pair(&test_user)
        .await
        .expect("Should generate token pair");

    // Try to use access token as refresh token - should fail
    let result = jwt_service.refresh_access_token(&tokens.access_token).await;
    assert!(result.is_err(), "Access token should not work as refresh token");

    // Try rotation with access token - should also fail
    let result = jwt_service.refresh_with_rotation(&tokens.access_token, test_user.id).await;
    assert!(result.is_err(), "Access token should not work for rotation");
}

#[tokio::test]
async fn test_token_blacklisting() {
    let pool = setup_test_pool().await;
    let blacklist_repo = TokenBlacklistRepository::new(pool.clone());
    let refresh_repo = RefreshTokenRepository::new(pool);

    let jwt_service = JwtService::new("test_secret_key_at_least_256_bits_long", blacklist_repo, refresh_repo);
    let test_user = create_test_user().await;

    // Generate token pair
    let tokens = jwt_service.generate_token_pair(&test_user)
        .await
        .expect("Should generate token pair");

    // Blacklist the access token
    jwt_service.blacklist_token(&tokens.access_token)
        .await
        .expect("Should blacklist token successfully");

    // Try to validate blacklisted token - should fail
    let result = jwt_service.validate_token(&tokens.access_token).await;
    assert!(result.is_err(), "Blacklisted token should fail validation");
}

#[tokio::test]
async fn test_refresh_token_revocation() {
    let pool = setup_test_pool().await;
    let blacklist_repo = TokenBlacklistRepository::new(pool.clone());
    let refresh_repo = RefreshTokenRepository::new(pool);

    let jwt_service = JwtService::new("test_secret_key_at_least_256_bits_long", blacklist_repo, refresh_repo);
    let test_user = create_test_user().await;

    // Generate token pair
    let tokens = jwt_service.generate_token_pair(&test_user)
        .await
        .expect("Should generate token pair");

    // Revoke the refresh token
    jwt_service.revoke_refresh_token(&tokens.refresh_token)
        .await
        .expect("Should revoke refresh token successfully");

    // Try to use revoked refresh token - should fail
    let result = jwt_service.refresh_access_token(&tokens.refresh_token).await;
    assert!(result.is_err(), "Revoked refresh token should fail");
}

#[tokio::test]
async fn test_token_decode_without_validation() {
    let pool = setup_test_pool().await;
    let blacklist_repo = TokenBlacklistRepository::new(pool.clone());
    let refresh_repo = RefreshTokenRepository::new(pool);

    let jwt_service = JwtService::new("test_secret_key_at_least_256_bits_long", blacklist_repo, refresh_repo);
    let test_user = create_test_user().await;

    // Generate token pair
    let tokens = jwt_service.generate_token_pair(&test_user)
        .await
        .expect("Should generate token pair");

    // Decode token without validation
    let claims = jwt_service.decode_token_without_validation(&tokens.access_token)
        .expect("Should decode token without validation");

    assert_eq!(claims.sub, test_user.id.to_string());
    assert_eq!(claims.email, test_user.email);
    assert!(matches!(claims.token_type, TokenType::Access));

    // This method should work even for expired tokens (not tested here due to timing)
    // and blacklisted tokens (the validation bypass is the point)
}