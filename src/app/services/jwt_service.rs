use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation, Algorithm};
use time::OffsetDateTime;
use uuid::Uuid;
use crate::app::models::jwt::{Claims, TokenType, TokenPair, JwtError, BlacklistedToken};
use crate::app::models::user::User;
use crate::app::repositories::token_blacklist_repository::TokenBlacklistRepository;

pub struct JwtService {
    encoding_key: EncodingKey,
    decoding_key: DecodingKey,
    blacklist_repository: TokenBlacklistRepository,
}

impl JwtService {
    pub fn new(secret: &str, blacklist_repository: TokenBlacklistRepository) -> Self {
        Self {
            encoding_key: EncodingKey::from_secret(secret.as_bytes()),
            decoding_key: DecodingKey::from_secret(secret.as_bytes()),
            blacklist_repository,
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
        let refresh_claims = Claims {
            sub: user.id.to_string(),
            email: user.email.clone(),
            roles: vec!["user".to_string()],
            exp: refresh_exp.unix_timestamp() as usize,
            iat: now.unix_timestamp() as usize,
            jti: Uuid::new_v4().to_string(),
            token_type: TokenType::Refresh,
        };

        let header = Header::new(Algorithm::HS256);

        let access_token = encode(&header, &access_claims, &self.encoding_key)
            .map_err(|e| JwtError::TokenCreationError(e.to_string()))?;

        let refresh_token = encode(&header, &refresh_claims, &self.encoding_key)
            .map_err(|e| JwtError::TokenCreationError(e.to_string()))?;

        Ok(TokenPair {
            access_token,
            refresh_token,
            token_type: "Bearer".to_string(),
            expires_in: 15 * 60, // 15 minutes in seconds
            refresh_expires_in: 7 * 24 * 60 * 60, // 7 days in seconds
        })
    }

    // Generate a new access token from a valid refresh token
    pub async fn refresh_access_token(&self, refresh_token: &str) -> Result<String, JwtError> {
        let claims = self.validate_token(refresh_token).await?;

        // Verify it's a refresh token
        match claims.token_type {
            TokenType::Refresh => {},
            TokenType::Access => return Err(JwtError::InvalidToken("Access token provided, refresh token required".to_string())),
        }

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

    // Validate a token and return its claims
    pub async fn validate_token(&self, token: &str) -> Result<Claims, JwtError> {
        let mut validation = Validation::new(Algorithm::HS256);
        validation.validate_exp = true;

        let token_data = decode::<Claims>(token, &self.decoding_key, &validation)
            .map_err(|e| match e.kind() {
                jsonwebtoken::errors::ErrorKind::ExpiredSignature => JwtError::ExpiredToken,
                _ => JwtError::InvalidToken(e.to_string()),
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

        let blacklisted_token = BlacklistedToken::new(
            claims.jti,
            user_id,
            claims.token_type,
            expires_at,
        );

        self.blacklist_repository
            .blacklist_token(&blacklisted_token)
            .await
            .map(|_| ())
            .map_err(|e| JwtError::TokenCreationError(format!("Failed to blacklist token: {}", e)))
    }

    // Decode token without validation (used for blacklisting expired tokens)
    fn decode_token_without_validation(&self, token: &str) -> Result<Claims, JwtError> {
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
            Err(JwtError::InvalidToken("Invalid authorization header format".to_string()))
        }
    }

    // Clean up expired blacklisted tokens (maintenance task)
    pub async fn cleanup_expired_blacklisted_tokens(&self) -> Result<usize, JwtError> {
        let now = OffsetDateTime::now_utc();
        self.blacklist_repository
            .cleanup_expired_tokens(now)
            .await
            .map_err(|e| JwtError::TokenCreationError(format!("Cleanup failed: {}", e)))
    }
}

// Helper function to get JWT secret from environment
pub fn get_jwt_secret() -> String {
    std::env::var("JWT_SECRET")
        .unwrap_or_else(|_| {
            eprintln!("Warning: JWT_SECRET not set, using default (not secure for production!)");
            "your-256-bit-secret-for-development-only-change-in-production".to_string()
        })
}