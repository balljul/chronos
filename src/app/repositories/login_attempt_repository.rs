use crate::app::models::login_attempt::{AccountLockout, LoginAttempt, RefreshTokenStorage};
use sqlx::{PgPool, Result as SqlxResult};
use time::OffsetDateTime;
use uuid::Uuid;

#[derive(Clone)]
pub struct LoginAttemptRepository {
    pool: PgPool,
}

impl LoginAttemptRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    // Record a login attempt
    pub async fn create_attempt(&self, attempt: &LoginAttempt) -> SqlxResult<()> {
        sqlx::query!(
            r#"
            INSERT INTO login_attempts (id, ip_address, email, user_id, success, failure_reason, user_agent, created_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
            "#,
            attempt.id,
            attempt.ip_address,
            attempt.email,
            attempt.user_id,
            attempt.success,
            attempt.failure_reason,
            attempt.user_agent,
            attempt.created_at
        )
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    // Count failed attempts for an IP address within a time window
    pub async fn count_failed_attempts_by_ip(
        &self,
        ip_address: &str,
        since: OffsetDateTime,
    ) -> SqlxResult<i64> {
        let count = sqlx::query_scalar!(
            r#"
            SELECT COUNT(*) as count
            FROM login_attempts
            WHERE ip_address = $1 AND success = false AND created_at >= $2
            "#,
            ip_address,
            since
        )
        .fetch_one(&self.pool)
        .await?;

        Ok(count.unwrap_or(0))
    }

    // Count failed attempts for a user (by email) within a time window
    pub async fn count_failed_attempts_by_email(
        &self,
        email: &str,
        since: OffsetDateTime,
    ) -> SqlxResult<i64> {
        let count = sqlx::query_scalar!(
            r#"
            SELECT COUNT(*) as count
            FROM login_attempts
            WHERE email = $1 AND success = false AND created_at >= $2
            "#,
            email,
            since
        )
        .fetch_one(&self.pool)
        .await?;

        Ok(count.unwrap_or(0))
    }

    // Get recent login attempts for analysis
    pub async fn get_recent_attempts_by_email(
        &self,
        email: &str,
        limit: i32,
    ) -> SqlxResult<Vec<LoginAttempt>> {
        let attempts = sqlx::query_as!(
            LoginAttempt,
            r#"
            SELECT id, ip_address, email, user_id, success, failure_reason, user_agent, created_at
            FROM login_attempts
            WHERE email = $1
            ORDER BY created_at DESC
            LIMIT $2
            "#,
            email,
            limit as i64
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(attempts)
    }

    // Clean up old login attempts (maintenance)
    pub async fn cleanup_old_attempts(&self, older_than: OffsetDateTime) -> SqlxResult<u64> {
        let result = sqlx::query!(
            "DELETE FROM login_attempts WHERE created_at < $1",
            older_than
        )
        .execute(&self.pool)
        .await?;

        Ok(result.rows_affected())
    }
}

#[derive(Clone)]
pub struct AccountLockoutRepository {
    pool: PgPool,
}

impl AccountLockoutRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    // Create an account lockout
    pub async fn create_lockout(&self, lockout: &AccountLockout) -> SqlxResult<()> {
        sqlx::query!(
            r#"
            INSERT INTO account_lockouts (id, user_id, locked_until, failed_attempts, locked_at, unlocked_at)
            VALUES ($1, $2, $3, $4, $5, $6)
            "#,
            lockout.id,
            lockout.user_id,
            lockout.locked_until,
            lockout.failed_attempts,
            lockout.locked_at,
            lockout.unlocked_at
        )
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    // Get active lockout for a user
    pub async fn get_active_lockout(&self, user_id: Uuid) -> SqlxResult<Option<AccountLockout>> {
        let lockout = sqlx::query_as!(
            AccountLockout,
            r#"
            SELECT id, user_id, locked_until, failed_attempts, locked_at, unlocked_at
            FROM account_lockouts
            WHERE user_id = $1 AND unlocked_at IS NULL AND locked_until > NOW()
            ORDER BY locked_at DESC
            LIMIT 1
            "#,
            user_id
        )
        .fetch_optional(&self.pool)
        .await?;

        Ok(lockout)
    }

    // Unlock an account (set unlocked_at)
    pub async fn unlock_account(&self, user_id: Uuid) -> SqlxResult<()> {
        sqlx::query!(
            r#"
            UPDATE account_lockouts
            SET unlocked_at = NOW()
            WHERE user_id = $1 AND unlocked_at IS NULL
            "#,
            user_id
        )
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    // Clean up expired lockouts
    pub async fn cleanup_expired_lockouts(&self) -> SqlxResult<u64> {
        let result = sqlx::query!(
            r#"
            UPDATE account_lockouts
            SET unlocked_at = NOW()
            WHERE unlocked_at IS NULL AND locked_until <= NOW()
            "#
        )
        .execute(&self.pool)
        .await?;

        Ok(result.rows_affected())
    }
}

#[derive(Clone)]
pub struct RefreshTokenRepository {
    pool: PgPool,
}

impl RefreshTokenRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    // Store a refresh token
    pub async fn store_token(&self, token: &RefreshTokenStorage) -> SqlxResult<()> {
        sqlx::query!(
            r#"
            INSERT INTO refresh_tokens (id, jti, user_id, token_hash, expires_at, revoked_at, created_at, last_used_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
            "#,
            token.id,
            token.jti,
            token.user_id,
            token.token_hash,
            token.expires_at,
            token.revoked_at,
            token.created_at,
            token.last_used_at
        )
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    // Find refresh token by JTI
    pub async fn find_by_jti(&self, jti: &str) -> SqlxResult<Option<RefreshTokenStorage>> {
        let row = sqlx::query!(
            r#"
            SELECT id, jti, user_id, token_hash, expires_at, revoked_at, created_at, last_used_at
            FROM refresh_tokens
            WHERE jti = $1
            "#,
            jti
        )
        .fetch_optional(&self.pool)
        .await?;

        if let Some(row) = row {
            Ok(Some(RefreshTokenStorage {
                id: row.id,
                jti: row.jti,
                user_id: row.user_id,
                token_hash: row.token_hash,
                expires_at: row.expires_at,
                revoked_at: row.revoked_at,
                created_at: row.created_at,
                last_used_at: row.last_used_at,
            }))
        } else {
            Ok(None)
        }
    }

    // Update last used time
    pub async fn update_last_used(&self, jti: &str) -> SqlxResult<()> {
        sqlx::query!(
            r#"
            UPDATE refresh_tokens
            SET last_used_at = NOW()
            WHERE jti = $1
            "#,
            jti
        )
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    // Revoke a refresh token
    pub async fn revoke_token(&self, jti: &str) -> SqlxResult<()> {
        sqlx::query!(
            r#"
            UPDATE refresh_tokens
            SET revoked_at = NOW()
            WHERE jti = $1
            "#,
            jti
        )
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    // Revoke all refresh tokens for a user
    pub async fn revoke_all_user_tokens(&self, user_id: Uuid) -> SqlxResult<()> {
        sqlx::query!(
            r#"
            UPDATE refresh_tokens
            SET revoked_at = NOW()
            WHERE user_id = $1 AND revoked_at IS NULL
            "#,
            user_id
        )
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    // Clean up expired tokens
    pub async fn cleanup_expired_tokens(&self) -> SqlxResult<u64> {
        let result = sqlx::query!("DELETE FROM refresh_tokens WHERE expires_at < NOW()")
            .execute(&self.pool)
            .await?;

        Ok(result.rows_affected())
    }
}
