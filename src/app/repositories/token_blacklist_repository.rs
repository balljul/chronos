use sqlx::{PgPool, Row};
use uuid::Uuid;
use time::OffsetDateTime;
use crate::app::models::jwt::{BlacklistedToken, TokenType};

type SqlxResult<T> = Result<T, sqlx::Error>;

#[derive(Clone)]
pub struct TokenBlacklistRepository {
    pool: PgPool,
}

impl TokenBlacklistRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    // Blacklist a token
    pub async fn blacklist_token(&self, token: &BlacklistedToken) -> SqlxResult<BlacklistedToken> {
        let token_type_str = match token.token_type {
            TokenType::Access => "access",
            TokenType::Refresh => "refresh",
        };

        let row = sqlx::query(
            r#"
            INSERT INTO blacklisted_tokens (id, jti, user_id, token_type, expires_at, blacklisted_at)
            VALUES ($1, $2, $3, $4, $5, $6)
            RETURNING id, jti, user_id, token_type, expires_at, blacklisted_at
            "#
        )
        .bind(token.id)
        .bind(&token.jti)
        .bind(token.user_id)
        .bind(token_type_str)
        .bind(token.expires_at)
        .bind(token.blacklisted_at)
        .fetch_one(&self.pool)
        .await?;

        let token_type = match row.get::<&str, _>("token_type") {
            "access" => TokenType::Access,
            "refresh" => TokenType::Refresh,
            _ => TokenType::Access, // Default fallback
        };

        Ok(BlacklistedToken {
            id: row.get("id"),
            jti: row.get("jti"),
            user_id: row.get("user_id"),
            token_type,
            expires_at: row.get("expires_at"),
            blacklisted_at: row.get("blacklisted_at"),
        })
    }

    // Check if a token is blacklisted
    pub async fn is_blacklisted(&self, jti: &str) -> SqlxResult<bool> {
        let row = sqlx::query(
            "SELECT COUNT(*) as count FROM blacklisted_tokens WHERE jti = $1"
        )
        .bind(jti)
        .fetch_one(&self.pool)
        .await?;

        let count: i64 = row.get("count");
        Ok(count > 0)
    }

    // Get all blacklisted tokens for a user (admin functionality)
    pub async fn get_blacklisted_tokens_by_user(&self, user_id: Uuid) -> SqlxResult<Vec<BlacklistedToken>> {
        let rows = sqlx::query(
            r#"
            SELECT id, jti, user_id, token_type, expires_at, blacklisted_at
            FROM blacklisted_tokens
            WHERE user_id = $1
            ORDER BY blacklisted_at DESC
            "#
        )
        .bind(user_id)
        .fetch_all(&self.pool)
        .await?;

        let mut tokens = Vec::new();
        for row in rows {
            let token_type = match row.get::<&str, _>("token_type") {
                "access" => TokenType::Access,
                "refresh" => TokenType::Refresh,
                _ => TokenType::Access,
            };

            tokens.push(BlacklistedToken {
                id: row.get("id"),
                jti: row.get("jti"),
                user_id: row.get("user_id"),
                token_type,
                expires_at: row.get("expires_at"),
                blacklisted_at: row.get("blacklisted_at"),
            });
        }

        Ok(tokens)
    }

    // Clean up expired blacklisted tokens (maintenance task)
    pub async fn cleanup_expired_tokens(&self, current_time: OffsetDateTime) -> SqlxResult<usize> {
        let result = sqlx::query(
            "DELETE FROM blacklisted_tokens WHERE expires_at < $1"
        )
        .bind(current_time)
        .execute(&self.pool)
        .await?;

        Ok(result.rows_affected() as usize)
    }

    // Blacklist all tokens for a user (useful for force logout/account security)
    pub async fn blacklist_all_user_tokens(&self, user_id: Uuid) -> SqlxResult<usize> {
        // This is a simplified approach - in practice, you'd need to track active tokens
        // For now, we'll create blacklist entries for potential tokens
        // In a real implementation, you might want to store active tokens or use a different approach

        // For demonstration, let's assume we're invalidating based on issued time
        let now = OffsetDateTime::now_utc();

        // Insert a special blacklist entry that invalidates all tokens issued before this time
        let special_jti = format!("user_logout_{}", user_id);

        let result = sqlx::query(
            r#"
            INSERT INTO blacklisted_tokens (id, jti, user_id, token_type, expires_at, blacklisted_at)
            VALUES ($1, $2, $3, 'access', $4, $5)
            ON CONFLICT (jti) DO NOTHING
            "#
        )
        .bind(Uuid::new_v4())
        .bind(special_jti)
        .bind(user_id)
        .bind(now + time::Duration::days(30)) // Expire in 30 days
        .bind(now)
        .execute(&self.pool)
        .await?;

        Ok(result.rows_affected() as usize)
    }

    // Get blacklisted token by JTI
    pub async fn get_by_jti(&self, jti: &str) -> SqlxResult<Option<BlacklistedToken>> {
        let row = sqlx::query(
            r#"
            SELECT id, jti, user_id, token_type, expires_at, blacklisted_at
            FROM blacklisted_tokens
            WHERE jti = $1
            "#
        )
        .bind(jti)
        .fetch_optional(&self.pool)
        .await?;

        if let Some(row) = row {
            let token_type = match row.get::<&str, _>("token_type") {
                "access" => TokenType::Access,
                "refresh" => TokenType::Refresh,
                _ => TokenType::Access,
            };

            Ok(Some(BlacklistedToken {
                id: row.get("id"),
                jti: row.get("jti"),
                user_id: row.get("user_id"),
                token_type,
                expires_at: row.get("expires_at"),
                blacklisted_at: row.get("blacklisted_at"),
            }))
        } else {
            Ok(None)
        }
    }

    // Get count of blacklisted tokens
    pub async fn count_blacklisted_tokens(&self) -> SqlxResult<i64> {
        let row = sqlx::query("SELECT COUNT(*) as count FROM blacklisted_tokens")
            .fetch_one(&self.pool)
            .await?;

        Ok(row.get("count"))
    }
}