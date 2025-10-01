use crate::app::models::jwt::{BlacklistedToken, Claims, JwtError, TokenPair, TokenType};
use crate::app::models::login_attempt::RefreshTokenStorage;
use crate::app::models::user::User;
use crate::app::repositories::login_attempt_repository::RefreshTokenRepository;
use crate::app::repositories::token_blacklist_repository::TokenBlacklistRepository;
use argon2::{
    Argon2, PasswordHasher,
    password_hash::{SaltString, rand_core::OsRng},
};
use jsonwebtoken::{Algorithm, DecodingKey, EncodingKey, Header, Validation, decode, encode};
use time::OffsetDateTime;
use uuid::Uuid;

#[derive(Clone)]
pub struct JwtService {
    encoding_key: EncodingKey,
    decoding_key: DecodingKey,
    blacklist_repository: TokenBlacklistRepository,
    refresh_token_repository: RefreshTokenRepository,
}

impl JwtService {
    pub fn new(
        secret: &str,
        blacklist_repository: TokenBlacklistRepository,
        refresh_token_repository: RefreshTokenRepository,
    ) -> Self {
        Self {
            encoding_key: EncodingKey::from_secret(secret.as_bytes()),
            decoding_key: DecodingKey::from_secret(secret.as_bytes()),
            blacklist_repository,
            refresh_token_repository,
        }
    }

    // Generate a pair of access and refresh tokens
    pub async fn generate_token_pair(&self, user: &User) -> Result<TokenPair, JwtError> {
        let now = OffsetDateTime::now_utc();

        // Access token: 15 minutes
        let access_exp = now + time::Duration::minutes(15);
        let access_claims = Claims {
            sub: user.id.to_string(),
            email: user.email.clone(),
            roles: vec!["user".to_string()], // Default role, can be expanded
            exp: access_exp.unix_timestamp() as usize,
            iat: now.unix_timestamp() as usize,
            jti: Uuid::new_v4().to_string(),
            token_type: TokenType::Access,
        };

        // Refresh token: 7 days
        let refresh_exp = now + time::Duration::days(7);
        let refresh_jti = Uuid::new_v4().to_string();
        let refresh_claims = Claims {
            sub: user.id.to_string(),
            email: user.email.clone(),
            roles: vec!["user".to_string()],
            exp: refresh_exp.unix_timestamp() as usize,
            iat: now.unix_timestamp() as usize,
            jti: refresh_jti.clone(),
            token_type: TokenType::Refresh,
        };

        let header = Header::new(Algorithm::HS256);

        let access_token = encode(&header, &access_claims, &self.encoding_key)
            .map_err(|e| JwtError::TokenCreationError(e.to_string()))?;

        let refresh_token = encode(&header, &refresh_claims, &self.encoding_key)
            .map_err(|e| JwtError::TokenCreationError(e.to_string()))?;

        // Hash the refresh token for secure storage
        let token_hash = self.hash_token(&refresh_token)?;

        // Store refresh token in database
        let refresh_token_storage =
            RefreshTokenStorage::new(refresh_jti, user.id, token_hash, refresh_exp);

        self.refresh_token_repository
            .store_token(&refresh_token_storage)
            .await
            .map_err(|e| {
                JwtError::TokenCreationError(format!("Failed to store refresh token: {}", e))
            })?;

        Ok(TokenPair {
            access_token,
            refresh_token,
            token_type: "Bearer".to_string(),
            expires_in: 15 * 60,                  // 15 minutes in seconds
            refresh_expires_in: 7 * 24 * 60 * 60, // 7 days in seconds
        })
    }

    // Generate a new access token from a valid refresh token
    pub async fn refresh_access_token(&self, refresh_token: &str) -> Result<String, JwtError> {
        let claims = self.validate_token(refresh_token).await?;

        // Verify it's a refresh token
        match claims.token_type {
            TokenType::Refresh => {}
            TokenType::Access => {
                return Err(JwtError::InvalidToken(
                    "Access token provided, refresh token required".to_string(),
                ));
            }
        }

        // Validate refresh token against database storage
        let stored_token = self
            .refresh_token_repository
            .find_by_jti(&claims.jti)
            .await
            .map_err(|e| JwtError::InvalidToken(format!("Database error: {}", e)))?;

        let stored_token = stored_token.ok_or(JwtError::InvalidToken(
            "Refresh token not found".to_string(),
        ))?;

        if !stored_token.is_valid() {
            return Err(JwtError::InvalidToken(
                "Refresh token is revoked or expired".to_string(),
            ));
        }

        // Verify the token hash matches
        let token_hash = self.hash_token(refresh_token)?;
        if stored_token.token_hash != token_hash {
            return Err(JwtError::InvalidToken(
                "Refresh token hash mismatch".to_string(),
            ));
        }

        // Update last used time
        self.refresh_token_repository
            .update_last_used(&claims.jti)
            .await
            .map_err(|e| {
                JwtError::TokenCreationError(format!("Failed to update token usage: {}", e))
            })?;

        // Generate new access token
        let now = OffsetDateTime::now_utc();
        let exp = now + time::Duration::minutes(15);

        let new_claims = Claims {
            sub: claims.sub,
            email: claims.email,
            roles: claims.roles,
            exp: exp.unix_timestamp() as usize,
            iat: now.unix_timestamp() as usize,
            jti: Uuid::new_v4().to_string(),
            token_type: TokenType::Access,
        };

        let header = Header::new(Algorithm::HS256);
        encode(&header, &new_claims, &self.encoding_key)
            .map_err(|e| JwtError::TokenCreationError(e.to_string()))
    }

    // Refresh with token rotation for enhanced security
    pub async fn refresh_with_rotation(
        &self,
        refresh_token: &str,
        user_id: Uuid,
    ) -> Result<TokenPair, JwtError> {
        let claims = self.validate_token(refresh_token).await?;

        // Verify it's a refresh token
        match claims.token_type {
            TokenType::Refresh => {}
            TokenType::Access => {
                return Err(JwtError::InvalidToken(
                    "Access token provided, refresh token required".to_string(),
                ));
            }
        }

        // Validate refresh token against database storage
        let stored_token = self
            .refresh_token_repository
            .find_by_jti(&claims.jti)
            .await
            .map_err(|e| JwtError::InvalidToken(format!("Database error: {}", e)))?;

        let stored_token = stored_token.ok_or(JwtError::InvalidToken(
            "Refresh token not found".to_string(),
        ))?;

        if !stored_token.is_valid() {
            return Err(JwtError::InvalidToken(
                "Refresh token is revoked or expired".to_string(),
            ));
        }

        // Verify the token hash matches
        let token_hash = self.hash_token(refresh_token)?;
        if stored_token.token_hash != token_hash {
            return Err(JwtError::InvalidToken(
                "Refresh token hash mismatch".to_string(),
            ));
        }

        // Revoke the old refresh token
        self.refresh_token_repository
            .revoke_token(&claims.jti)
            .await
            .map_err(|e| {
                JwtError::TokenCreationError(format!("Failed to revoke old refresh token: {}", e))
            })?;

        // Create a mock user object for token generation
        let user = User {
            id: user_id,
            name: None,
            email: claims.email.clone(),
            password_hash: String::new(), // Not used in token generation
            created_at: Some(OffsetDateTime::now_utc()), // Not used in token generation
            updated_at: Some(OffsetDateTime::now_utc()), // Not used in token generation
        };

        // Generate new token pair
        self.generate_token_pair(&user).await
    }

    // Validate a token and return its claims
    pub async fn validate_token(&self, token: &str) -> Result<Claims, JwtError> {
        let mut validation = Validation::new(Algorithm::HS256);
        validation.validate_exp = true;

        let token_data = decode::<Claims>(token, &self.decoding_key, &validation).map_err(|e| {
            match e.kind() {
                jsonwebtoken::errors::ErrorKind::ExpiredSignature => JwtError::ExpiredToken,
                _ => JwtError::InvalidToken(e.to_string()),
            }
        })?;

        // Check if token is blacklisted
        if self.is_token_blacklisted(&token_data.claims.jti).await? {
            return Err(JwtError::BlacklistedToken);
        }

        Ok(token_data.claims)
    }

    // Check if a token is blacklisted
    pub async fn is_token_blacklisted(&self, jti: &str) -> Result<bool, JwtError> {
        self.blacklist_repository
            .is_blacklisted(jti)
            .await
            .map_err(|e| JwtError::InvalidToken(format!("Database error: {}", e)))
    }

    // Blacklist a token (for logout)
    pub async fn blacklist_token(&self, token: &str) -> Result<(), JwtError> {
        let claims = self.decode_token_without_validation(token)?;

        let expires_at = OffsetDateTime::from_unix_timestamp(claims.exp as i64)
            .map_err(|_| JwtError::InvalidClaims("Invalid expiration time".to_string()))?;

        let user_id = Uuid::parse_str(&claims.sub)
            .map_err(|_| JwtError::InvalidClaims("Invalid user ID".to_string()))?;

        let blacklisted_token =
            BlacklistedToken::new(claims.jti, user_id, claims.token_type, expires_at);

        self.blacklist_repository
            .blacklist_token(&blacklisted_token)
            .await
            .map(|_| ())
            .map_err(|e| JwtError::TokenCreationError(format!("Failed to blacklist token: {}", e)))
    }

    // Decode token without validation (used for blacklisting expired tokens)
    pub fn decode_token_without_validation(&self, token: &str) -> Result<Claims, JwtError> {
        let mut validation = Validation::new(Algorithm::HS256);
        validation.validate_exp = false; // Don't validate expiration
        validation.validate_nbf = false;
        validation.validate_aud = false;

        let token_data = decode::<Claims>(token, &self.decoding_key, &validation)
            .map_err(|e| JwtError::InvalidToken(e.to_string()))?;

        Ok(token_data.claims)
    }

    // Extract token from Authorization header
    pub fn extract_token_from_header(auth_header: &str) -> Result<&str, JwtError> {
        if auth_header.starts_with("Bearer ") {
            Ok(&auth_header[7..])
        } else {
            Err(JwtError::InvalidToken(
                "Invalid authorization header format".to_string(),
            ))
        }
    }

    // Hash a token for secure storage (used for refresh tokens)
    fn hash_token(&self, token: &str) -> Result<String, JwtError> {
        let salt = SaltString::generate(&mut OsRng);
        let argon2 = Argon2::default();
        let hash = argon2
            .hash_password(token.as_bytes(), &salt)
            .map_err(|e| JwtError::TokenCreationError(format!("Token hashing failed: {}", e)))?;
        Ok(hash.to_string())
    }

    // Revoke refresh token on logout
    pub async fn revoke_refresh_token(&self, refresh_token: &str) -> Result<(), JwtError> {
        let claims = self.decode_token_without_validation(refresh_token)?;

        if matches!(claims.token_type, TokenType::Refresh) {
            self.refresh_token_repository
                .revoke_token(&claims.jti)
                .await
                .map_err(|e| {
                    JwtError::TokenCreationError(format!("Failed to revoke refresh token: {}", e))
                })?;
        }

        Ok(())
    }

    // Revoke all refresh tokens for a user
    pub async fn revoke_all_user_refresh_tokens(&self, user_id: Uuid) -> Result<(), JwtError> {
        self.refresh_token_repository
            .revoke_all_user_tokens(user_id)
            .await
            .map_err(|e| {
                JwtError::TokenCreationError(format!("Failed to revoke user tokens: {}", e))
            })
    }

    // Clean up expired blacklisted tokens (maintenance task)
    pub async fn cleanup_expired_blacklisted_tokens(&self) -> Result<usize, JwtError> {
        let now = OffsetDateTime::now_utc();
        self.blacklist_repository
            .cleanup_expired_tokens(now)
            .await
            .map_err(|e| JwtError::TokenCreationError(format!("Cleanup failed: {}", e)))
    }

    // Clean up expired refresh tokens (maintenance task)
    pub async fn cleanup_expired_refresh_tokens(&self) -> Result<u64, JwtError> {
        self.refresh_token_repository
            .cleanup_expired_tokens()
            .await
            .map_err(|e| {
                JwtError::TokenCreationError(format!("Refresh token cleanup failed: {}", e))
            })
    }
}

// Helper function to get JWT secret from environment
pub fn get_jwt_secret() -> String {
    std::env::var("JWT_SECRET").unwrap_or_else(|_| {
        eprintln!("Warning: JWT_SECRET not set, using default (not secure for production!)");
        "your-256-bit-secret-for-development-only-change-in-production".to_string()
    })
}
