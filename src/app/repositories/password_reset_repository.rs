use sqlx::{PgPool, Result as SqlxResult};
use uuid::Uuid;
use time::OffsetDateTime;
use crate::app::models::password_reset::PasswordResetToken;

pub struct PasswordResetRepository {
    pool: PgPool,
}

impl PasswordResetRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn create(&self, token: &PasswordResetToken) -> SqlxResult<PasswordResetToken> {
        let row = sqlx::query!(
            r#"
            INSERT INTO password_reset_tokens (id, user_id, token_hash, expires_at, used, created_at)
            VALUES ($1, $2, $3, $4, $5, $6)
            RETURNING id, user_id, token_hash, expires_at, used, created_at
            "#,
            token.id,
            token.user_id,
            token.token_hash,
            token.expires_at,
            token.used,
            token.created_at
        )
        .fetch_one(&self.pool)
        .await?;

        Ok(PasswordResetToken {
            id: row.id,
            user_id: row.user_id,
            token_hash: row.token_hash,
            expires_at: row.expires_at,
            used: row.used.unwrap_or(false),
            created_at: row.created_at.unwrap_or_else(|| OffsetDateTime::now_utc()),
        })
    }

    pub async fn find_by_user_id(&self, user_id: Uuid) -> SqlxResult<Option<PasswordResetToken>> {
        let row = sqlx::query!(
            r#"
            SELECT id, user_id, token_hash, expires_at, used, created_at
            FROM password_reset_tokens
            WHERE user_id = $1 AND used = false AND expires_at > $2
            ORDER BY created_at DESC
            LIMIT 1
            "#,
            user_id,
            OffsetDateTime::now_utc()
        )
        .fetch_optional(&self.pool)
        .await?;

        Ok(row.map(|r| PasswordResetToken {
            id: r.id,
            user_id: r.user_id,
            token_hash: r.token_hash,
            expires_at: r.expires_at,
            used: r.used.unwrap_or(false),
            created_at: r.created_at.unwrap_or_else(|| OffsetDateTime::now_utc()),
        }))
    }

    pub async fn find_valid_tokens_by_user_id(&self, user_id: Uuid) -> SqlxResult<Vec<PasswordResetToken>> {
        let rows = sqlx::query!(
            r#"
            SELECT id, user_id, token_hash, expires_at, used, created_at
            FROM password_reset_tokens
            WHERE user_id = $1 AND used = false AND expires_at > $2
            ORDER BY created_at DESC
            "#,
            user_id,
            OffsetDateTime::now_utc()
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(rows.into_iter().map(|r| PasswordResetToken {
            id: r.id,
            user_id: r.user_id,
            token_hash: r.token_hash,
            expires_at: r.expires_at,
            used: r.used.unwrap_or(false),
            created_at: r.created_at.unwrap_or_else(|| OffsetDateTime::now_utc()),
        }).collect())
    }

    pub async fn mark_as_used(&self, id: Uuid) -> SqlxResult<bool> {
        let result = sqlx::query!(
            "UPDATE password_reset_tokens SET used = true WHERE id = $1",
            id
        )
        .execute(&self.pool)
        .await?;

        Ok(result.rows_affected() > 0)
    }

    pub async fn count_recent_requests(&self, user_id: Uuid, since: OffsetDateTime) -> SqlxResult<i64> {
        let count = sqlx::query_scalar!(
            "SELECT COUNT(*) FROM password_reset_tokens WHERE user_id = $1 AND created_at > $2",
            user_id,
            since
        )
        .fetch_one(&self.pool)
        .await?;

        Ok(count.unwrap_or(0))
    }

    pub async fn cleanup_expired_tokens(&self) -> SqlxResult<u64> {
        let result = sqlx::query!(
            "DELETE FROM password_reset_tokens WHERE expires_at < $1",
            OffsetDateTime::now_utc()
        )
        .execute(&self.pool)
        .await?;

        Ok(result.rows_affected())
    }
}