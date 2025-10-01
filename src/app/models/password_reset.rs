use argon2::{
    Argon2, PasswordHash, PasswordHasher, PasswordVerifier,
    password_hash::{SaltString, rand_core::OsRng},
};
use serde::{Deserialize, Serialize};
use time::OffsetDateTime;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PasswordResetToken {
    pub id: Uuid,
    pub user_id: Uuid,
    pub token_hash: String,
    pub expires_at: OffsetDateTime,
    pub used: bool,
    pub created_at: OffsetDateTime,
}

impl PasswordResetToken {
    pub fn new(user_id: Uuid, plain_token: &str) -> Result<Self, argon2::password_hash::Error> {
        let token_hash = Self::hash_token(plain_token)?;
        let expires_at = OffsetDateTime::now_utc() + time::Duration::hours(1);

        Ok(Self {
            id: Uuid::new_v4(),
            user_id,
            token_hash,
            expires_at,
            used: false,
            created_at: OffsetDateTime::now_utc(),
        })
    }

    pub fn hash_token(token: &str) -> Result<String, argon2::password_hash::Error> {
        let salt = SaltString::generate(&mut OsRng);
        let argon2 = Argon2::default();
        let token_hash = argon2.hash_password(token.as_bytes(), &salt)?;
        Ok(token_hash.to_string())
    }

    pub fn verify_token(&self, token: &str) -> Result<bool, argon2::password_hash::Error> {
        let parsed_hash = PasswordHash::new(&self.token_hash)?;
        let argon2 = Argon2::default();
        Ok(argon2
            .verify_password(token.as_bytes(), &parsed_hash)
            .is_ok())
    }

    pub fn is_expired(&self) -> bool {
        OffsetDateTime::now_utc() > self.expires_at
    }

    pub fn is_valid(&self) -> bool {
        !self.used && !self.is_expired()
    }

    pub fn generate_secure_token() -> String {
        use rand::Rng;
        const CHARSET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZ\
                               abcdefghijklmnopqrstuvwxyz\
                               0123456789";
        const TOKEN_LEN: usize = 64;

        let mut rng = rand::thread_rng();

        (0..TOKEN_LEN)
            .map(|_| {
                let idx = rng.gen_range(0..CHARSET.len());
                CHARSET[idx] as char
            })
            .collect()
    }
}
