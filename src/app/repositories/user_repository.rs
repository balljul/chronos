use sqlx::{PgPool, Result as SqlxResult};
use uuid::Uuid;
use time::OffsetDateTime;
use crate::app::models::user::User;

pub struct UserRepository {
    pool: PgPool,
}

impl UserRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn create(&self, user: &User) -> SqlxResult<User> {
        let user = sqlx::query_as!(
            User,
            r#"
            INSERT INTO users (id, first_name, email, password_hash, created_at, updated_at)
            VALUES ($1, $2, $3, $4, $5, $6)
            RETURNING id, first_name as name, email, password_hash, created_at, updated_at
            "#,
            user.id,
            user.name,
            user.email,
            user.password_hash,
            user.created_at,
            user.updated_at
        )
        .fetch_one(&self.pool)
        .await?;

        Ok(user)
    }

    pub async fn find_by_id(&self, id: Uuid) -> SqlxResult<Option<User>> {
        let user = sqlx::query_as!(
            User,
            r#"
            SELECT id, first_name as name, email, password_hash, created_at, updated_at
            FROM users
            WHERE id = $1
            "#,
            id
        )
        .fetch_optional(&self.pool)
        .await?;

        Ok(user)
    }

    pub async fn find_by_email(&self, email: &str) -> SqlxResult<Option<User>> {
        let user = sqlx::query_as!(
            User,
            r#"
            SELECT id, first_name as name, email, password_hash, created_at, updated_at
            FROM users
            WHERE email = $1
            "#,
            email
        )
        .fetch_optional(&self.pool)
        .await?;

        Ok(user)
    }

    pub async fn get_all(&self) -> SqlxResult<Vec<User>> {
        let users = sqlx::query_as!(
            User,
            r#"
            SELECT id, first_name as name, email, password_hash, created_at, updated_at
            FROM users
            ORDER BY created_at DESC
            "#
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(users)
    }

    pub async fn update(&self, id: Uuid, name: Option<&str>, email: Option<&str>, password_hash: Option<&str>) -> SqlxResult<Option<User>> {
        let user = sqlx::query_as!(
            User,
            r#"
            UPDATE users
            SET
                first_name = COALESCE($2, first_name),
                email = COALESCE($3, email),
                password_hash = COALESCE($4, password_hash),
                updated_at = $5
            WHERE id = $1
            RETURNING id, first_name as name, email, password_hash, created_at, updated_at
            "#,
            id,
            name,
            email,
            password_hash,
            OffsetDateTime::now_utc()
        )
        .fetch_optional(&self.pool)
        .await?;

        Ok(user)
    }

    pub async fn delete(&self, id: Uuid) -> SqlxResult<bool> {
        let result = sqlx::query!(
            "DELETE FROM users WHERE id = $1",
            id
        )
        .execute(&self.pool)
        .await?;

        Ok(result.rows_affected() > 0)
    }
}
