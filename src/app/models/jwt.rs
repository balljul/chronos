use serde::{Deserialize, Serialize};
use uuid::Uuid;
use time::OffsetDateTime;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Claims {
    pub sub: String,      // Subject (user_id)
    pub email: String,    // User email
    pub roles: Vec<String>, // User roles (future extensibility)
    pub exp: usize,       // Expiration time (as UTC timestamp)
    pub iat: usize,       // Issued at (as UTC timestamp)
    pub jti: String,      // JWT ID (unique identifier for this token)
    pub token_type: TokenType, // Access or Refresh token
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum TokenType {
    Access,
    Refresh,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TokenPair {
    pub access_token: String,
    pub refresh_token: String,
    pub token_type: String,      // "Bearer"
    pub expires_in: usize,       // Access token expiry in seconds
    pub refresh_expires_in: usize, // Refresh token expiry in seconds
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LoginRequest {
    pub email: String,
    pub password: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LoginResponse {
    pub message: String,
    pub user: crate::app::models::user::UserResponse,
    pub tokens: TokenPair,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RefreshTokenRequest {
    pub refresh_token: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RefreshTokenResponse {
    pub access_token: String,
    pub token_type: String,
    pub expires_in: usize,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LogoutRequest {
    pub refresh_token: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LogoutResponse {
    pub message: String,
}

// Token blacklist model for database storage
#[derive(Debug, Clone)]
pub struct BlacklistedToken {
    pub id: Uuid,
    pub jti: String,              // JWT ID of the blacklisted token
    pub user_id: Uuid,            // User who owned the token
    pub token_type: TokenType,    // Access or Refresh token
    pub expires_at: OffsetDateTime, // When the original token would have expired
    pub blacklisted_at: OffsetDateTime, // When it was blacklisted
}

impl BlacklistedToken {
    pub fn new(jti: String, user_id: Uuid, token_type: TokenType, expires_at: OffsetDateTime) -> Self {
        Self {
            id: Uuid::new_v4(),
            jti,
            user_id,
            token_type,
            expires_at,
            blacklisted_at: OffsetDateTime::now_utc(),
        }
    }
}

// JWT error types
#[derive(Debug)]
pub enum JwtError {
    InvalidToken(String),
    ExpiredToken,
    TokenCreationError(String),
    MissingToken,
    BlacklistedToken,
    InvalidClaims(String),
}

impl std::fmt::Display for JwtError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            JwtError::InvalidToken(e) => write!(f, "Invalid token: {}", e),
            JwtError::ExpiredToken => write!(f, "Token has expired"),
            JwtError::TokenCreationError(e) => write!(f, "Token creation failed: {}", e),
            JwtError::MissingToken => write!(f, "No token provided"),
            JwtError::BlacklistedToken => write!(f, "Token has been revoked"),
            JwtError::InvalidClaims(e) => write!(f, "Invalid token claims: {}", e),
        }
    }
}

impl std::error::Error for JwtError {}

// Authentication context for middleware
#[derive(Debug, Clone)]
pub struct AuthContext {
    pub user_id: Uuid,
    pub email: String,
    pub roles: Vec<String>,
    pub jti: String,
}

impl From<Claims> for AuthContext {
    fn from(claims: Claims) -> Self {
        Self {
            user_id: Uuid::parse_str(&claims.sub).unwrap_or_default(),
            email: claims.email,
            roles: claims.roles,
            jti: claims.jti,
        }
    }
}