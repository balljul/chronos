use chronos::app::models::jwt::{Claims, TokenType, JwtError, BlacklistedToken};
use chronos::app::models::user::User;
use chronos::app::services::jwt_service::JwtService;
use chronos::app::repositories::token_blacklist_repository::TokenBlacklistRepository;
use chronos::app::repositories::login_attempt_repository::RefreshTokenRepository;
use chronos::app::repositories::user_repository::UserRepository;
use sqlx::PgPool;
use uuid::Uuid;
use time::OffsetDateTime;

#[tokio::test]
async fn test_jwt_token_generation() {
    let pool = get_test_pool().await;
    let blacklist_repo = TokenBlacklistRepository::new(pool.clone());
    let refresh_token_repo = RefreshTokenRepository::new(pool.clone());
    let jwt_service = JwtService::new("test-secret-key", blacklist_repo, refresh_token_repo);

    let user = User::new(
        Some("Test User".to_string()),
        "test@example.com".to_string(),
        "TestPassword123!",
    ).unwrap();

    let token_pair = jwt_service.generate_token_pair(&user).await.unwrap();

    // Verify token structure
    assert_eq!(token_pair.token_type, "Bearer");
    assert_eq!(token_pair.expires_in, 15 * 60); // 15 minutes
    assert_eq!(token_pair.refresh_expires_in, 7 * 24 * 60 * 60); // 7 days

    // Verify tokens are different
    assert_ne!(token_pair.access_token, token_pair.refresh_token);

    // Verify tokens are not empty
    assert!(!token_pair.access_token.is_empty());
    assert!(!token_pair.refresh_token.is_empty());
}

#[tokio::test]
async fn test_jwt_token_validation() {
    let pool = get_test_pool().await;
    let blacklist_repo = TokenBlacklistRepository::new(pool.clone());
    let jwt_service = JwtService::new("test-secret-key", blacklist_repo, RefreshTokenRepository::new(pool.clone()));

    let user = User::new(
        Some("Test User".to_string()),
        "test@example.com".to_string(),
        "TestPassword123!",
    ).unwrap();

    let token_pair = jwt_service.generate_token_pair(&user).await.unwrap();

    // Validate access token
    let access_claims = jwt_service.validate_token(&token_pair.access_token).await.unwrap();
    assert_eq!(access_claims.sub, user.id.to_string());
    assert_eq!(access_claims.email, user.email);
    assert_eq!(access_claims.roles, vec!["user"]);
    assert!(matches!(access_claims.token_type, TokenType::Access));

    // Validate refresh token
    let refresh_claims = jwt_service.validate_token(&token_pair.refresh_token).await.unwrap();
    assert_eq!(refresh_claims.sub, user.id.to_string());
    assert_eq!(refresh_claims.email, user.email);
    assert!(matches!(refresh_claims.token_type, TokenType::Refresh));
}

#[tokio::test]
async fn test_jwt_token_validation_with_invalid_token() {
    let pool = get_test_pool().await;
    let blacklist_repo = TokenBlacklistRepository::new(pool.clone());
    let jwt_service = JwtService::new("test-secret-key", blacklist_repo, RefreshTokenRepository::new(pool.clone()));

    // Test with completely invalid token
    let result = jwt_service.validate_token("invalid.token.here").await;
    assert!(result.is_err());
    assert!(matches!(result.unwrap_err(), JwtError::InvalidToken(_)));

    // Test with empty token
    let result = jwt_service.validate_token("").await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_jwt_token_validation_with_wrong_secret() {
    let pool = get_test_pool().await;
    let blacklist_repo = TokenBlacklistRepository::new(pool.clone());
    let jwt_service1 = JwtService::new("secret1", blacklist_repo, RefreshTokenRepository::new(pool.clone()));

    let blacklist_repo2 = TokenBlacklistRepository::new(pool.clone());
    let jwt_service2 = JwtService::new("secret2", blacklist_repo2, RefreshTokenRepository::new(pool.clone()));

    let user = User::new(
        Some("Test User".to_string()),
        "test@example.com".to_string(),
        "TestPassword123!",
    ).unwrap();

    // Generate token with first service
    let token_pair = jwt_service1.generate_token_pair(&user).await.unwrap();

    // Try to validate with second service (different secret)
    let result = jwt_service2.validate_token(&token_pair.access_token).await;
    assert!(result.is_err());
    assert!(matches!(result.unwrap_err(), JwtError::InvalidToken(_)));
}

#[tokio::test]
async fn test_refresh_token_functionality() {
    let pool = get_test_pool().await;
    let blacklist_repo = TokenBlacklistRepository::new(pool.clone());
    let jwt_service = JwtService::new("test-secret-key", blacklist_repo, RefreshTokenRepository::new(pool.clone()));

    let user = User::new(
        Some("Test User".to_string()),
        "test@example.com".to_string(),
        "TestPassword123!",
    ).unwrap();

    let token_pair = jwt_service.generate_token_pair(&user).await.unwrap();

    // Use refresh token to get new access token
    let new_access_token = jwt_service
        .refresh_access_token(&token_pair.refresh_token)
        .await
        .unwrap();

    // Verify new access token is valid
    let claims = jwt_service.validate_token(&new_access_token).await.unwrap();
    assert_eq!(claims.sub, user.id.to_string());
    assert_eq!(claims.email, user.email);
    assert!(matches!(claims.token_type, TokenType::Access));

    // Verify new token is different from original
    assert_ne!(new_access_token, token_pair.access_token);
}

#[tokio::test]
async fn test_refresh_token_with_access_token_should_fail() {
    let pool = get_test_pool().await;
    let blacklist_repo = TokenBlacklistRepository::new(pool.clone());
    let jwt_service = JwtService::new("test-secret-key", blacklist_repo, RefreshTokenRepository::new(pool.clone()));

    let user = User::new(
        Some("Test User".to_string()),
        "test@example.com".to_string(),
        "TestPassword123!",
    ).unwrap();

    let token_pair = jwt_service.generate_token_pair(&user).await.unwrap();

    // Try to refresh using access token instead of refresh token
    let result = jwt_service.refresh_access_token(&token_pair.access_token).await;
    assert!(result.is_err());
    assert!(matches!(result.unwrap_err(), JwtError::InvalidToken(_)));
}

#[tokio::test]
async fn test_token_blacklisting() {
    let pool = get_test_pool().await;
    let blacklist_repo = TokenBlacklistRepository::new(pool.clone());
    let jwt_service = JwtService::new("test-secret-key", blacklist_repo, RefreshTokenRepository::new(pool.clone()));

    let user = User::new(
        Some("Test User".to_string()),
        format!("test-blacklist-{}@example.com", Uuid::new_v4()),
        "TestPassword123!",
    ).unwrap();

    // Create user in database first
    let user_repo = UserRepository::new(pool.clone());
    let user = user_repo.create(&user).await.unwrap();

    let token_pair = jwt_service.generate_token_pair(&user).await.unwrap();

    // Verify token is valid before blacklisting
    assert!(jwt_service.validate_token(&token_pair.access_token).await.is_ok());

    // Blacklist the token
    jwt_service.blacklist_token(&token_pair.access_token).await.unwrap();

    // Verify token is now invalid
    let result = jwt_service.validate_token(&token_pair.access_token).await;
    assert!(result.is_err());
    assert!(matches!(result.unwrap_err(), JwtError::BlacklistedToken));
}

#[tokio::test]
async fn test_blacklisted_token_model() {
    let user_id = Uuid::new_v4();
    let jti = "test-jti-123".to_string();
    let expires_at = OffsetDateTime::now_utc() + time::Duration::hours(1);

    let blacklisted_token = BlacklistedToken::new(
        jti.clone(),
        user_id,
        TokenType::Access,
        expires_at,
    );

    assert_eq!(blacklisted_token.jti, jti);
    assert_eq!(blacklisted_token.user_id, user_id);
    assert!(matches!(blacklisted_token.token_type, TokenType::Access));
    assert_eq!(blacklisted_token.expires_at, expires_at);
    assert!(!blacklisted_token.id.is_nil());
}

#[tokio::test]
async fn test_token_header_extraction() {
    // Valid Bearer token
    let header = "Bearer eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.test.token";
    let token = JwtService::extract_token_from_header(header).unwrap();
    assert_eq!(token, "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.test.token");

    // Invalid header format
    let header = "Basic dGVzdDp0ZXN0";
    let result = JwtService::extract_token_from_header(header);
    assert!(result.is_err());

    // Missing Bearer prefix
    let header = "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.test.token";
    let result = JwtService::extract_token_from_header(header);
    assert!(result.is_err());

    // Empty header
    let result = JwtService::extract_token_from_header("");
    assert!(result.is_err());
}

#[tokio::test]
async fn test_token_claims_structure() {
    let claims = Claims {
        sub: "user-123".to_string(),
        email: "test@example.com".to_string(),
        roles: vec!["user".to_string(), "admin".to_string()],
        exp: 1234567890,
        iat: 1234567800,
        jti: "jwt-123".to_string(),
        token_type: TokenType::Access,
    };

    assert_eq!(claims.sub, "user-123");
    assert_eq!(claims.email, "test@example.com");
    assert_eq!(claims.roles.len(), 2);
    assert!(claims.roles.contains(&"user".to_string()));
    assert!(claims.roles.contains(&"admin".to_string()));
    assert!(matches!(claims.token_type, TokenType::Access));
}

#[tokio::test]
async fn test_jwt_error_display() {
    let error = JwtError::InvalidToken("Test error".to_string());
    assert_eq!(error.to_string(), "Invalid token: Test error");

    let error = JwtError::ExpiredToken;
    assert_eq!(error.to_string(), "Token has expired");

    let error = JwtError::MissingToken;
    assert_eq!(error.to_string(), "No token provided");

    let error = JwtError::BlacklistedToken;
    assert_eq!(error.to_string(), "Token has been revoked");
}

#[tokio::test]
async fn test_cleanup_expired_blacklisted_tokens() {
    let pool = get_test_pool().await;
    let blacklist_repo = TokenBlacklistRepository::new(pool.clone());
    let jwt_service = JwtService::new("test-secret-key", blacklist_repo, RefreshTokenRepository::new(pool.clone()));

    let user = User::new(
        Some("Test User".to_string()),
        format!("test-cleanup-{}@example.com", Uuid::new_v4()),
        "TestPassword123!",
    ).unwrap();

    // Create user in database first
    let user_repo = UserRepository::new(pool.clone());
    let user = user_repo.create(&user).await.unwrap();

    // Create an expired token for blacklisting
    let expired_token = BlacklistedToken {
        id: Uuid::new_v4(),
        jti: "expired-token".to_string(),
        user_id: user.id,
        token_type: TokenType::Access,
        expires_at: OffsetDateTime::now_utc() - time::Duration::hours(1), // Expired 1 hour ago
        blacklisted_at: OffsetDateTime::now_utc() - time::Duration::hours(2),
    };

    // Add expired token to blacklist
    // For testing purposes, we'll create a blacklist repository directly
    let blacklist_repo = TokenBlacklistRepository::new(pool.clone());
    blacklist_repo.blacklist_token(&expired_token).await.unwrap();

    // Run cleanup
    let cleaned_count = jwt_service.cleanup_expired_blacklisted_tokens().await.unwrap();
    assert!(cleaned_count >= 1);
}

#[tokio::test]
async fn test_token_uniqueness() {
    let pool = get_test_pool().await;
    let blacklist_repo = TokenBlacklistRepository::new(pool.clone());
    let jwt_service = JwtService::new("test-secret-key", blacklist_repo, RefreshTokenRepository::new(pool.clone()));

    let user = User::new(
        Some("Test User".to_string()),
        "test@example.com".to_string(),
        "TestPassword123!",
    ).unwrap();

    // Generate multiple token pairs
    let mut access_tokens = std::collections::HashSet::new();
    let mut refresh_tokens = std::collections::HashSet::new();

    for _ in 0..10 {
        let token_pair = jwt_service.generate_token_pair(&user).await.unwrap();

        // Ensure tokens are unique
        assert!(access_tokens.insert(token_pair.access_token.clone()));
        assert!(refresh_tokens.insert(token_pair.refresh_token.clone()));

        // Ensure access and refresh tokens are different from each other
        assert_ne!(token_pair.access_token, token_pair.refresh_token);
    }

    assert_eq!(access_tokens.len(), 10);
    assert_eq!(refresh_tokens.len(), 10);
}

// Helper function to get test database pool
async fn get_test_pool() -> PgPool {
    dotenvy::dotenv().ok();
    let database_url = std::env::var("DATABASE_URL")
        .expect("DATABASE_URL must be set for testing");

    PgPool::connect(&database_url)
        .await
        .expect("Failed to connect to test database")
}