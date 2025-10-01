use chronos::app::models::jwt::{Claims, JwtError, LogoutRequest, LogoutResponse, TokenType};
use chronos::app::models::user::User;
use chronos::app::repositories::login_attempt_repository::RefreshTokenRepository;
use chronos::app::repositories::token_blacklist_repository::TokenBlacklistRepository;
use chronos::app::repositories::user_repository::UserRepository;
use chronos::app::services::jwt_service::JwtService;
use dotenvy::dotenv;
use sqlx::PgPool;
use std::env;
use time::OffsetDateTime;
use uuid::Uuid;

async fn setup_test_pool() -> PgPool {
    dotenv().ok();
    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set for tests");

    sqlx::PgPool::connect(&database_url)
        .await
        .expect("Failed to connect to test database")
}

async fn create_test_user(pool: &PgPool, email_suffix: &str) -> User {
    let user = User::new(
        Some("Test User".to_string()),
        format!("test{}@example.com", email_suffix),
        "TestPassword123!",
    )
    .expect("Failed to create test user");

    let user_repository = UserRepository::new(pool.clone());
    user_repository
        .create(&user)
        .await
        .expect("Failed to save test user")
}

#[tokio::test]
async fn test_logout_request_serialization() {
    // Test normal logout request
    let request = LogoutRequest {
        refresh_token: Some("test_refresh_token".to_string()),
        logout_all_devices: Some(false),
    };

    let json = serde_json::to_string(&request).expect("Should serialize");
    let deserialized: LogoutRequest = serde_json::from_str(&json).expect("Should deserialize");

    assert_eq!(request.refresh_token, deserialized.refresh_token);
    assert_eq!(request.logout_all_devices, deserialized.logout_all_devices);

    // Test logout all devices request
    let request_all = LogoutRequest {
        refresh_token: Some("test_refresh_token".to_string()),
        logout_all_devices: Some(true),
    };

    let json_all = serde_json::to_string(&request_all).expect("Should serialize");
    let deserialized_all: LogoutRequest =
        serde_json::from_str(&json_all).expect("Should deserialize");

    assert_eq!(request_all.logout_all_devices, Some(true));
    assert_eq!(deserialized_all.logout_all_devices, Some(true));

    // Test minimal request
    let minimal_request = LogoutRequest {
        refresh_token: None,
        logout_all_devices: None,
    };

    let json_minimal = serde_json::to_string(&minimal_request).expect("Should serialize");
    let deserialized_minimal: LogoutRequest =
        serde_json::from_str(&json_minimal).expect("Should deserialize");

    assert_eq!(minimal_request.refresh_token, None);
    assert_eq!(deserialized_minimal.logout_all_devices, None);
}

#[tokio::test]
async fn test_logout_response_serialization() {
    // Test normal logout response
    let response = LogoutResponse {
        message: "Logged out successfully".to_string(),
        logged_out_devices: Some(1),
    };

    let json = serde_json::to_string(&response).expect("Should serialize");
    let deserialized: LogoutResponse = serde_json::from_str(&json).expect("Should deserialize");

    assert_eq!(response.message, deserialized.message);
    assert_eq!(response.logged_out_devices, deserialized.logged_out_devices);

    // Test response without device count
    let response_no_count = LogoutResponse {
        message: "Logged out successfully".to_string(),
        logged_out_devices: None,
    };

    let json_no_count = serde_json::to_string(&response_no_count).expect("Should serialize");
    let deserialized_no_count: LogoutResponse =
        serde_json::from_str(&json_no_count).expect("Should deserialize");

    assert_eq!(response_no_count.logged_out_devices, None);
    assert_eq!(deserialized_no_count.logged_out_devices, None);
}

#[tokio::test]
async fn test_single_device_logout_flow() {
    let pool = setup_test_pool().await;
    let blacklist_repo = TokenBlacklistRepository::new(pool.clone());
    let refresh_repo = RefreshTokenRepository::new(pool.clone());

    let jwt_service = JwtService::new(
        "test_secret_key_at_least_256_bits_long",
        blacklist_repo,
        refresh_repo,
    );
    let test_user = create_test_user(&pool, "_single_logout").await;

    // Generate token pair
    let tokens = jwt_service
        .generate_token_pair(&test_user)
        .await
        .expect("Should generate token pair");

    // Verify tokens are initially valid
    let access_claims = jwt_service
        .validate_token(&tokens.access_token)
        .await
        .expect("Access token should be valid initially");
    assert_eq!(access_claims.sub, test_user.id.to_string());

    let refresh_claims = jwt_service
        .validate_token(&tokens.refresh_token)
        .await
        .expect("Refresh token should be valid initially");
    assert_eq!(refresh_claims.sub, test_user.id.to_string());

    // Simulate logout: blacklist access token and revoke refresh token
    jwt_service
        .blacklist_token(&tokens.access_token)
        .await
        .expect("Should blacklist access token");

    jwt_service
        .revoke_refresh_token(&tokens.refresh_token)
        .await
        .expect("Should revoke refresh token");

    jwt_service
        .blacklist_token(&tokens.refresh_token)
        .await
        .expect("Should blacklist refresh token");

    // Verify access token is now blacklisted
    let access_result = jwt_service.validate_token(&tokens.access_token).await;
    assert!(access_result.is_err());
    assert!(matches!(
        access_result.unwrap_err(),
        JwtError::BlacklistedToken
    ));

    // Verify refresh token is also blacklisted
    let refresh_result = jwt_service.validate_token(&tokens.refresh_token).await;
    assert!(refresh_result.is_err());
    assert!(matches!(
        refresh_result.unwrap_err(),
        JwtError::BlacklistedToken
    ));

    // Verify refresh token can't be used for new tokens
    let refresh_attempt = jwt_service
        .refresh_access_token(&tokens.refresh_token)
        .await;
    assert!(refresh_attempt.is_err());

    // Cleanup
    let user_repository = UserRepository::new(pool);
    let _ = user_repository.delete(test_user.id).await;
}

#[tokio::test]
async fn test_logout_all_devices_flow() {
    let pool = setup_test_pool().await;
    let blacklist_repo = TokenBlacklistRepository::new(pool.clone());
    let refresh_repo = RefreshTokenRepository::new(pool.clone());

    let jwt_service = JwtService::new(
        "test_secret_key_at_least_256_bits_long",
        blacklist_repo,
        refresh_repo,
    );
    let test_user = create_test_user(&pool, "_all_devices_logout").await;

    // Generate multiple token pairs to simulate multiple devices
    let tokens1 = jwt_service
        .generate_token_pair(&test_user)
        .await
        .expect("Should generate first token pair");

    let tokens2 = jwt_service
        .generate_token_pair(&test_user)
        .await
        .expect("Should generate second token pair");

    let tokens3 = jwt_service
        .generate_token_pair(&test_user)
        .await
        .expect("Should generate third token pair");

    // Verify all refresh tokens are initially valid
    assert!(
        jwt_service
            .validate_token(&tokens1.refresh_token)
            .await
            .is_ok()
    );
    assert!(
        jwt_service
            .validate_token(&tokens2.refresh_token)
            .await
            .is_ok()
    );
    assert!(
        jwt_service
            .validate_token(&tokens3.refresh_token)
            .await
            .is_ok()
    );

    // Logout all devices - revoke all refresh tokens for the user
    jwt_service
        .revoke_all_user_refresh_tokens(test_user.id)
        .await
        .expect("Should revoke all user refresh tokens");

    // Verify all refresh tokens can no longer be used
    let refresh1_result = jwt_service
        .refresh_access_token(&tokens1.refresh_token)
        .await;
    assert!(
        refresh1_result.is_err(),
        "First refresh token should be revoked"
    );

    let refresh2_result = jwt_service
        .refresh_access_token(&tokens2.refresh_token)
        .await;
    assert!(
        refresh2_result.is_err(),
        "Second refresh token should be revoked"
    );

    let refresh3_result = jwt_service
        .refresh_access_token(&tokens3.refresh_token)
        .await;
    assert!(
        refresh3_result.is_err(),
        "Third refresh token should be revoked"
    );

    // Cleanup
    let user_repository = UserRepository::new(pool);
    let _ = user_repository.delete(test_user.id).await;
}

#[tokio::test]
async fn test_logout_with_invalid_token() {
    let pool = setup_test_pool().await;
    let blacklist_repo = TokenBlacklistRepository::new(pool.clone());
    let refresh_repo = RefreshTokenRepository::new(pool.clone());

    let jwt_service = JwtService::new(
        "test_secret_key_at_least_256_bits_long",
        blacklist_repo,
        refresh_repo,
    );

    // Test with completely invalid token
    let invalid_token = "invalid.token.here";

    // This should not panic - logout should handle invalid tokens gracefully
    let decode_result = jwt_service.decode_token_without_validation(invalid_token);
    assert!(decode_result.is_err());

    // Attempting to blacklist invalid token should handle gracefully
    let blacklist_result = jwt_service.blacklist_token(invalid_token).await;
    // This might succeed or fail depending on implementation, but shouldn't panic

    // Attempting to revoke invalid refresh token should handle gracefully
    let revoke_result = jwt_service.revoke_refresh_token(invalid_token).await;
    // This should fail but handle gracefully
    assert!(revoke_result.is_err());
}

#[tokio::test]
async fn test_logout_with_expired_token() {
    let pool = setup_test_pool().await;
    let blacklist_repo = TokenBlacklistRepository::new(pool.clone());
    let refresh_repo = RefreshTokenRepository::new(pool.clone());

    let jwt_service = JwtService::new(
        "test_secret_key_at_least_256_bits_long",
        blacklist_repo,
        refresh_repo,
    );
    let test_user = create_test_user(&pool, "_expired_logout").await;

    // Generate token pair
    let tokens = jwt_service
        .generate_token_pair(&test_user)
        .await
        .expect("Should generate token pair");

    // We can't easily create an expired token for testing, so we'll simulate
    // the logout flow with a valid token and then verify the behavior

    // Even if token is expired, decode_token_without_validation should work
    let decode_result = jwt_service.decode_token_without_validation(&tokens.access_token);
    assert!(decode_result.is_ok());

    let claims = decode_result.unwrap();
    assert_eq!(claims.sub, test_user.id.to_string());

    // Logout operations should still work (blacklisting and revoking)
    let blacklist_result = jwt_service.blacklist_token(&tokens.access_token).await;
    assert!(blacklist_result.is_ok());

    let revoke_result = jwt_service
        .revoke_refresh_token(&tokens.refresh_token)
        .await;
    assert!(revoke_result.is_ok());

    // Cleanup
    let user_repository = UserRepository::new(pool);
    let _ = user_repository.delete(test_user.id).await;
}

#[tokio::test]
async fn test_token_cleanup_during_logout() {
    let pool = setup_test_pool().await;
    let blacklist_repo = TokenBlacklistRepository::new(pool.clone());
    let refresh_repo = RefreshTokenRepository::new(pool.clone());

    let jwt_service = JwtService::new(
        "test_secret_key_at_least_256_bits_long",
        blacklist_repo,
        refresh_repo,
    );
    let test_user = create_test_user(&pool, "_cleanup_logout").await;

    // Generate some tokens and blacklist them
    let tokens = jwt_service
        .generate_token_pair(&test_user)
        .await
        .expect("Should generate token pair");

    jwt_service
        .blacklist_token(&tokens.access_token)
        .await
        .expect("Should blacklist access token");

    jwt_service
        .blacklist_token(&tokens.refresh_token)
        .await
        .expect("Should blacklist refresh token");

    // Test cleanup functions
    let blacklist_cleanup_result = jwt_service.cleanup_expired_blacklisted_tokens().await;
    assert!(blacklist_cleanup_result.is_ok());

    let refresh_cleanup_result = jwt_service.cleanup_expired_refresh_tokens().await;
    assert!(refresh_cleanup_result.is_ok());

    // Cleanup should not affect the functionality
    // (Since our tokens aren't actually expired, they shouldn't be cleaned up)

    // Cleanup
    let user_repository = UserRepository::new(pool);
    let _ = user_repository.delete(test_user.id).await;
}

#[tokio::test]
async fn test_concurrent_logout_operations() {
    let pool = setup_test_pool().await;
    let blacklist_repo = TokenBlacklistRepository::new(pool.clone());
    let refresh_repo = RefreshTokenRepository::new(pool.clone());

    let jwt_service = JwtService::new(
        "test_secret_key_at_least_256_bits_long",
        blacklist_repo,
        refresh_repo,
    );
    let test_user = create_test_user(&pool, "_concurrent_logout").await;

    // Generate token pair
    let tokens = jwt_service
        .generate_token_pair(&test_user)
        .await
        .expect("Should generate token pair");

    // Simulate concurrent logout operations
    let jwt_service1 = jwt_service.clone();
    let jwt_service2 = jwt_service.clone();
    let access_token1 = tokens.access_token.clone();
    let access_token2 = tokens.access_token.clone();
    let refresh_token1 = tokens.refresh_token.clone();
    let refresh_token2 = tokens.refresh_token.clone();

    let task1 = tokio::spawn(async move { jwt_service1.blacklist_token(&access_token1).await });

    let task2 = tokio::spawn(async move { jwt_service2.blacklist_token(&access_token2).await });

    let task3 =
        tokio::spawn(async move { jwt_service.revoke_refresh_token(&refresh_token1).await });

    // Wait for all tasks to complete
    let results = futures::future::join_all(vec![task1, task2, task3]).await;

    // At least one operation should succeed (they might conflict, but system should handle it)
    let successful_operations = results
        .into_iter()
        .filter_map(|r| r.ok())
        .filter(|r| r.is_ok())
        .count();

    assert!(
        successful_operations > 0,
        "At least one concurrent operation should succeed"
    );

    // Cleanup
    let user_repository = UserRepository::new(pool);
    let _ = user_repository.delete(test_user.id).await;
}

#[tokio::test]
async fn test_logout_idempotency() {
    let pool = setup_test_pool().await;
    let blacklist_repo = TokenBlacklistRepository::new(pool.clone());
    let refresh_repo = RefreshTokenRepository::new(pool.clone());

    let jwt_service = JwtService::new(
        "test_secret_key_at_least_256_bits_long",
        blacklist_repo,
        refresh_repo,
    );
    let test_user = create_test_user(&pool, "_idempotent_logout").await;

    // Generate token pair
    let tokens = jwt_service
        .generate_token_pair(&test_user)
        .await
        .expect("Should generate token pair");

    // Perform logout operations multiple times
    for i in 1..=3 {
        let blacklist_result = jwt_service.blacklist_token(&tokens.access_token).await;
        // First time should succeed, subsequent times should not fail the system
        if i == 1 {
            assert!(blacklist_result.is_ok(), "First blacklist should succeed");
        }
        // Subsequent calls should either succeed (idempotent) or fail gracefully

        let revoke_result = jwt_service
            .revoke_refresh_token(&tokens.refresh_token)
            .await;
        // Similar expectation for revoke operations
        if i == 1 {
            assert!(revoke_result.is_ok(), "First revoke should succeed");
        }
    }

    // Token should still be blacklisted after multiple operations
    let validation_result = jwt_service.validate_token(&tokens.access_token).await;
    assert!(validation_result.is_err());
    assert!(matches!(
        validation_result.unwrap_err(),
        JwtError::BlacklistedToken
    ));

    // Cleanup
    let user_repository = UserRepository::new(pool);
    let _ = user_repository.delete(test_user.id).await;
}

#[tokio::test]
async fn test_logout_edge_cases() {
    let pool = setup_test_pool().await;
    let blacklist_repo = TokenBlacklistRepository::new(pool.clone());
    let refresh_repo = RefreshTokenRepository::new(pool);

    let jwt_service = JwtService::new(
        "test_secret_key_at_least_256_bits_long",
        blacklist_repo,
        refresh_repo,
    );

    // Test with empty string token
    let empty_result = jwt_service.blacklist_token("").await;
    // Should handle gracefully (might succeed or fail, but shouldn't panic)

    // Test with very long invalid token
    let long_invalid_token = "a".repeat(10000);
    let long_result = jwt_service.blacklist_token(&long_invalid_token).await;
    // Should handle gracefully

    // Test with special characters
    let special_token = "token.with.special.chars!@#$%^&*()";
    let special_result = jwt_service.blacklist_token(special_token).await;
    // Should handle gracefully

    // Test revoking non-existent refresh token
    let uuid = Uuid::new_v4();
    let nonexistent_result = jwt_service.revoke_all_user_refresh_tokens(uuid).await;
    // Should succeed (no tokens to revoke for this user)
    assert!(nonexistent_result.is_ok());
}
