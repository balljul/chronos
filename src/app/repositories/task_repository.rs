use crate::app::models::task::{Task, CreateTaskRequest, UpdateTaskRequest};
use sqlx::{PgPool, Result as SqlxResult};
use time::OffsetDateTime;
use uuid::Uuid;

#[derive(Clone)]
pub struct TaskRepository {
    pool: PgPool,
}

impl TaskRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn create(&self, task_data: &CreateTaskRequest, user_id: Uuid) -> SqlxResult<Task> {
        let task = sqlx::query_as!(
            Task,
            r#"
            INSERT INTO tasks (name, description, project_id, user_id, is_active, created_at, updated_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7)
            RETURNING id, name, description, project_id, user_id, is_active, created_at, updated_at
            "#,
            task_data.name,
            task_data.description,
            task_data.project_id,
            user_id,
            true,
            OffsetDateTime::now_utc(),
            OffsetDateTime::now_utc()
        )
        .fetch_one(&self.pool)
        .await?;

        Ok(task)
    }

    pub async fn find_by_id(&self, id: Uuid, user_id: Uuid) -> SqlxResult<Option<Task>> {
        let task = sqlx::query_as!(
            Task,
            r#"
            SELECT id, name, description, project_id, user_id, is_active, created_at, updated_at
            FROM tasks
            WHERE id = $1 AND user_id = $2
            "#,
            id,
            user_id
        )
        .fetch_optional(&self.pool)
        .await?;

        Ok(task)
    }

    pub async fn find_by_user(&self, user_id: Uuid, include_inactive: Option<bool>) -> SqlxResult<Vec<Task>> {
        let include_inactive = include_inactive.unwrap_or(false);

        let tasks = if include_inactive {
            sqlx::query_as!(
                Task,
                r#"
                SELECT id, name, description, project_id, user_id, is_active, created_at, updated_at
                FROM tasks
                WHERE user_id = $1
                ORDER BY created_at DESC
                "#,
                user_id
            )
            .fetch_all(&self.pool)
            .await?
        } else {
            sqlx::query_as!(
                Task,
                r#"
                SELECT id, name, description, project_id, user_id, is_active, created_at, updated_at
                FROM tasks
                WHERE user_id = $1 AND is_active = true
                ORDER BY created_at DESC
                "#,
                user_id
            )
            .fetch_all(&self.pool)
            .await?
        };

        Ok(tasks)
    }

    pub async fn find_by_project(&self, project_id: Uuid, user_id: Uuid, include_inactive: Option<bool>) -> SqlxResult<Vec<Task>> {
        let include_inactive = include_inactive.unwrap_or(false);

        let tasks = if include_inactive {
            sqlx::query_as!(
                Task,
                r#"
                SELECT id, name, description, project_id, user_id, is_active, created_at, updated_at
                FROM tasks
                WHERE project_id = $1 AND user_id = $2
                ORDER BY created_at DESC
                "#,
                project_id,
                user_id
            )
            .fetch_all(&self.pool)
            .await?
        } else {
            sqlx::query_as!(
                Task,
                r#"
                SELECT id, name, description, project_id, user_id, is_active, created_at, updated_at
                FROM tasks
                WHERE project_id = $1 AND user_id = $2 AND is_active = true
                ORDER BY created_at DESC
                "#,
                project_id,
                user_id
            )
            .fetch_all(&self.pool)
            .await?
        };

        Ok(tasks)
    }

    pub async fn find_without_project(&self, user_id: Uuid, include_inactive: Option<bool>) -> SqlxResult<Vec<Task>> {
        let include_inactive = include_inactive.unwrap_or(false);

        let tasks = if include_inactive {
            sqlx::query_as!(
                Task,
                r#"
                SELECT id, name, description, project_id, user_id, is_active, created_at, updated_at
                FROM tasks
                WHERE project_id IS NULL AND user_id = $1
                ORDER BY created_at DESC
                "#,
                user_id
            )
            .fetch_all(&self.pool)
            .await?
        } else {
            sqlx::query_as!(
                Task,
                r#"
                SELECT id, name, description, project_id, user_id, is_active, created_at, updated_at
                FROM tasks
                WHERE project_id IS NULL AND user_id = $1 AND is_active = true
                ORDER BY created_at DESC
                "#,
                user_id
            )
            .fetch_all(&self.pool)
            .await?
        };

        Ok(tasks)
    }

    pub async fn update(
        &self,
        id: Uuid,
        user_id: Uuid,
        task_data: &UpdateTaskRequest,
    ) -> SqlxResult<Option<Task>> {
        let task = sqlx::query_as!(
            Task,
            r#"
            UPDATE tasks
            SET
                name = COALESCE($3, name),
                description = COALESCE($4, description),
                project_id = COALESCE($5, project_id),
                is_active = COALESCE($6, is_active),
                updated_at = $7
            WHERE id = $1 AND user_id = $2
            RETURNING id, name, description, project_id, user_id, is_active, created_at, updated_at
            "#,
            id,
            user_id,
            task_data.name.as_deref(),
            task_data.description.as_deref(),
            task_data.project_id,
            task_data.is_active,
            OffsetDateTime::now_utc()
        )
        .fetch_optional(&self.pool)
        .await?;

        Ok(task)
    }

    pub async fn delete(&self, id: Uuid, user_id: Uuid) -> SqlxResult<bool> {
        let result = sqlx::query!(
            "DELETE FROM tasks WHERE id = $1 AND user_id = $2",
            id,
            user_id
        )
        .execute(&self.pool)
        .await?;

        Ok(result.rows_affected() > 0)
    }

    pub async fn soft_delete(&self, id: Uuid, user_id: Uuid) -> SqlxResult<bool> {
        let result = sqlx::query!(
            r#"
            UPDATE tasks
            SET is_active = false, updated_at = $3
            WHERE id = $1 AND user_id = $2
            "#,
            id,
            user_id,
            OffsetDateTime::now_utc()
        )
        .execute(&self.pool)
        .await?;

        Ok(result.rows_affected() > 0)
    }
}