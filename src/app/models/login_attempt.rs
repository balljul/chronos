use serde::{Deserialize, Serialize};
use time::OffsetDateTime;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoginAttempt {
    pub id: Uuid,
    pub ip_address: String,
    pub email: String,
    pub user_id: Option<Uuid>,
    pub success: bool,
    pub failure_reason: Option<String>,
    pub user_agent: Option<String>,
    pub created_at: OffsetDateTime,
}

impl LoginAttempt {
    pub fn new_success(
        ip_address: String,
        email: String,
        user_id: Uuid,
        user_agent: Option<String>,
    ) -> Self {
        Self {
            id: Uuid::new_v4(),
            ip_address,
            email,
            user_id: Some(user_id),
            success: true,
            failure_reason: None,
            user_agent,
            created_at: OffsetDateTime::now_utc(),
        }
    }

    pub fn new_failure(
        ip_address: String,
        email: String,
        failure_reason: String,
        user_agent: Option<String>,
    ) -> Self {
        Self {
            id: Uuid::new_v4(),
            ip_address,
            email,
            user_id: None,
            success: false,
            failure_reason: Some(failure_reason),
            user_agent,
            created_at: OffsetDateTime::now_utc(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccountLockout {
    pub id: Uuid,
    pub user_id: Uuid,
    pub locked_until: OffsetDateTime,
    pub failed_attempts: i32,
    pub locked_at: OffsetDateTime,
    pub unlocked_at: Option<OffsetDateTime>,
}

impl AccountLockout {
    pub fn new(user_id: Uuid, failed_attempts: i32, lockout_duration_minutes: i64) -> Self {
        let now = OffsetDateTime::now_utc();
        Self {
            id: Uuid::new_v4(),
            user_id,
            locked_until: now + time::Duration::minutes(lockout_duration_minutes),
            failed_attempts,
            locked_at: now,
            unlocked_at: None,
        }
    }

    pub fn is_locked(&self) -> bool {
        let now = OffsetDateTime::now_utc();
        self.unlocked_at.is_none() && now < self.locked_until
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RefreshTokenStorage {
    pub id: Uuid,
    pub jti: String,
    pub user_id: Uuid,
    pub token_hash: String, // Store hash of refresh token for security
    pub expires_at: OffsetDateTime,
    pub revoked_at: Option<OffsetDateTime>,
    pub created_at: Option<OffsetDateTime>, // Nullable in the database
    pub last_used_at: Option<OffsetDateTime>,
}

impl RefreshTokenStorage {
    pub fn new(jti: String, user_id: Uuid, token_hash: String, expires_at: OffsetDateTime) -> Self {
        Self {
            id: Uuid::new_v4(),
            jti,
            user_id,
            token_hash,
            expires_at,
            revoked_at: None,
            created_at: Some(OffsetDateTime::now_utc()),
            last_used_at: None,
        }
    }

    pub fn is_valid(&self) -> bool {
        let now = OffsetDateTime::now_utc();
        self.revoked_at.is_none() && now < self.expires_at
    }

    pub fn revoke(&mut self) {
        self.revoked_at = Some(OffsetDateTime::now_utc());
    }
}
