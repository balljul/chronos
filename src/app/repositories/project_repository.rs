use crate::app::models::project::{Project, CreateProjectRequest, UpdateProjectRequest};
use sqlx::{PgPool, Result as SqlxResult};
use time::OffsetDateTime;
use uuid::Uuid;

#[derive(Clone)]
pub struct ProjectRepository {
    pool: PgPool,
}

impl ProjectRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn create(&self, project_data: &CreateProjectRequest, user_id: Uuid) -> SqlxResult<Project> {
        let project = sqlx::query_as!(
            Project,
            r#"
            INSERT INTO projects (name, description, color, user_id, is_active, created_at, updated_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7)
            RETURNING id, name, description, color, user_id, is_active, created_at, updated_at
            "#,
            project_data.name,
            project_data.description,
            project_data.color,
            user_id,
            true,
            OffsetDateTime::now_utc(),
            OffsetDateTime::now_utc()
        )
        .fetch_one(&self.pool)
        .await?;

        Ok(project)
    }

    pub async fn find_by_id(&self, id: Uuid, user_id: Uuid) -> SqlxResult<Option<Project>> {
        let project = sqlx::query_as!(
            Project,
            r#"
            SELECT id, name, description, color, user_id, is_active, created_at, updated_at
            FROM projects
            WHERE id = $1 AND user_id = $2
            "#,
            id,
            user_id
        )
        .fetch_optional(&self.pool)
        .await?;

        Ok(project)
    }

    pub async fn find_by_user(&self, user_id: Uuid, include_inactive: Option<bool>) -> SqlxResult<Vec<Project>> {
        let include_inactive = include_inactive.unwrap_or(false);

        let projects = if include_inactive {
            sqlx::query_as!(
                Project,
                r#"
                SELECT id, name, description, color, user_id, is_active, created_at, updated_at
                FROM projects
                WHERE user_id = $1
                ORDER BY created_at DESC
                "#,
                user_id
            )
            .fetch_all(&self.pool)
            .await?
        } else {
            sqlx::query_as!(
                Project,
                r#"
                SELECT id, name, description, color, user_id, is_active, created_at, updated_at
                FROM projects
                WHERE user_id = $1 AND is_active = true
                ORDER BY created_at DESC
                "#,
                user_id
            )
            .fetch_all(&self.pool)
            .await?
        };

        Ok(projects)
    }

    pub async fn update(
        &self,
        id: Uuid,
        user_id: Uuid,
        project_data: &UpdateProjectRequest,
    ) -> SqlxResult<Option<Project>> {
        let project = sqlx::query_as!(
            Project,
            r#"
            UPDATE projects
            SET
                name = COALESCE($3, name),
                description = COALESCE($4, description),
                color = COALESCE($5, color),
                is_active = COALESCE($6, is_active),
                updated_at = $7
            WHERE id = $1 AND user_id = $2
            RETURNING id, name, description, color, user_id, is_active, created_at, updated_at
            "#,
            id,
            user_id,
            project_data.name.as_deref(),
            project_data.description.as_deref(),
            project_data.color.as_deref(),
            project_data.is_active,
            OffsetDateTime::now_utc()
        )
        .fetch_optional(&self.pool)
        .await?;

        Ok(project)
    }

    pub async fn delete(&self, id: Uuid, user_id: Uuid) -> SqlxResult<bool> {
        let result = sqlx::query!(
            "DELETE FROM projects WHERE id = $1 AND user_id = $2",
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
            UPDATE projects
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