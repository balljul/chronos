use crate::app::models::time_entry::{TimeEntry, TimeEntryError, TimeEntryFilters};
use sqlx::PgPool;
use time::OffsetDateTime;
use uuid::Uuid;

#[derive(Clone)]
pub struct TimeEntryRepository {
    pool: PgPool,
}

impl TimeEntryRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn create(&self, entry: &TimeEntry) -> Result<TimeEntry, TimeEntryError> {
        if entry.end_time.is_none() {
            let running_count: i64 = sqlx::query_scalar(
                "SELECT COUNT(*) FROM time_entries WHERE user_id = $1 AND end_time IS NULL"
            )
            .bind(entry.user_id)
            .fetch_one(&self.pool)
            .await?;

            if running_count > 0 {
                return Err(TimeEntryError::RunningTimerExists);
            }
        }

        if let Some(end_time) = entry.end_time {
            if end_time <= entry.start_time {
                return Err(TimeEntryError::InvalidTimeRange);
            }
        }

        let result = sqlx::query_as!(
            TimeEntry,
            r#"
            INSERT INTO time_entries (id, user_id, description, project_id, task_id, start_time, end_time)
            VALUES ($1, $2, $3, $4, $5, $6, $7)
            RETURNING id, user_id, description, project_id, task_id, start_time, end_time, duration, created_at, updated_at
            "#,
            entry.id,
            entry.user_id,
            entry.description,
            entry.project_id,
            entry.task_id,
            entry.start_time,
            entry.end_time
        )
        .fetch_one(&self.pool)
        .await?;

        Ok(result)
    }

    pub async fn find_by_id(&self, id: Uuid, user_id: Uuid) -> Result<TimeEntry, TimeEntryError> {
        let result = sqlx::query_as!(
            TimeEntry,
            "SELECT id, user_id, description, project_id, task_id, start_time, end_time, duration, created_at, updated_at FROM time_entries WHERE id = $1 AND user_id = $2",
            id,
            user_id
        )
        .fetch_optional(&self.pool)
        .await?;

        result.ok_or(TimeEntryError::NotFound)
    }

    pub async fn find_by_id_any_user(&self, id: Uuid) -> Result<TimeEntry, TimeEntryError> {
        let result = sqlx::query_as!(
            TimeEntry,
            "SELECT id, user_id, description, project_id, task_id, start_time, end_time, duration, created_at, updated_at FROM time_entries WHERE id = $1",
            id
        )
        .fetch_optional(&self.pool)
        .await?;

        result.ok_or(TimeEntryError::NotFound)
    }

    pub async fn update(&self, id: Uuid, user_id: Uuid, entry: &TimeEntry) -> Result<TimeEntry, TimeEntryError> {
        if let Some(end_time) = entry.end_time {
            if end_time <= entry.start_time {
                return Err(TimeEntryError::InvalidTimeRange);
            }
        }

        let result = sqlx::query_as!(
            TimeEntry,
            r#"
            UPDATE time_entries 
            SET description = $3, project_id = $4, task_id = $5, start_time = $6, end_time = $7, updated_at = CURRENT_TIMESTAMP
            WHERE id = $1 AND user_id = $2
            RETURNING id, user_id, description, project_id, task_id, start_time, end_time, duration, created_at, updated_at
            "#,
            id,
            user_id,
            entry.description,
            entry.project_id,
            entry.task_id,
            entry.start_time,
            entry.end_time
        )
        .fetch_optional(&self.pool)
        .await?;

        result.ok_or(TimeEntryError::NotFound)
    }

    pub async fn delete(&self, id: Uuid, user_id: Uuid) -> Result<(), TimeEntryError> {
        let result = sqlx::query!(
            "DELETE FROM time_entries WHERE id = $1 AND user_id = $2",
            id,
            user_id
        )
        .execute(&self.pool)
        .await?;

        if result.rows_affected() == 0 {
            return Err(TimeEntryError::NotFound);
        }

        Ok(())
    }

    pub async fn stop_timer(&self, id: Uuid, user_id: Uuid) -> Result<TimeEntry, TimeEntryError> {
        let result = sqlx::query_as!(
            TimeEntry,
            r#"
            UPDATE time_entries 
            SET end_time = $3, updated_at = CURRENT_TIMESTAMP
            WHERE id = $1 AND user_id = $2 AND end_time IS NULL
            RETURNING id, user_id, description, project_id, task_id, start_time, end_time, duration, created_at, updated_at
            "#,
            id,
            user_id,
            OffsetDateTime::now_utc()
        )
        .fetch_optional(&self.pool)
        .await?;

        result.ok_or(TimeEntryError::TimerNotRunning)
    }

    pub async fn find_current_timer(&self, user_id: Uuid) -> Result<TimeEntry, TimeEntryError> {
        let result = sqlx::query_as!(
            TimeEntry,
            "SELECT id, user_id, description, project_id, task_id, start_time, end_time, duration, created_at, updated_at FROM time_entries WHERE user_id = $1 AND end_time IS NULL",
            user_id
        )
        .fetch_optional(&self.pool)
        .await?;

        result.ok_or(TimeEntryError::NotFound)
    }

    pub async fn find_with_filters(&self, user_id: Uuid, filters: &TimeEntryFilters) -> Result<(Vec<TimeEntry>, i64, i32), TimeEntryError> {
        let page = filters.page.unwrap_or(1);
        let limit = filters.limit.unwrap_or(20);
        let offset = (page - 1) * limit;

        let mut where_conditions = vec!["user_id = $1".to_string()];
        let mut param_count = 1;
        
        if filters.start_date.is_some() {
            param_count += 1;
            where_conditions.push(format!("start_time >= ${}", param_count));
        }
        
        if filters.end_date.is_some() {
            param_count += 1;
            where_conditions.push(format!("start_time <= ${}", param_count));
        }
        
        if filters.project_id.is_some() {
            param_count += 1;
            where_conditions.push(format!("project_id = ${}", param_count));
        }
        
        if filters.task_id.is_some() {
            param_count += 1;
            where_conditions.push(format!("task_id = ${}", param_count));
        }
        
        if let Some(is_running) = filters.is_running {
            if is_running {
                where_conditions.push("end_time IS NULL".to_string());
            } else {
                where_conditions.push("end_time IS NOT NULL".to_string());
            }
        }

        let where_clause = where_conditions.join(" AND ");
        
        let sort_clause = match filters.sort_by.as_deref() {
            Some("duration") => "ORDER BY duration DESC NULLS LAST",
            Some("start_time") => "ORDER BY start_time DESC",
            _ => "ORDER BY start_time DESC",
        };

        let count_query = format!(
            "SELECT COUNT(*) FROM time_entries WHERE {}",
            where_clause
        );
        
        let duration_query = format!(
            "SELECT COALESCE(SUM(duration), 0) FROM time_entries WHERE {} AND duration IS NOT NULL",
            where_clause
        );

        let limit_param = param_count + 1;
        let offset_param = param_count + 2;
        let entries_query = format!(
            "SELECT id, user_id, description, project_id, task_id, start_time, end_time, duration, created_at, updated_at FROM time_entries WHERE {} {} LIMIT ${} OFFSET ${}",
            where_clause,
            sort_clause,
            limit_param,
            offset_param
        );

        let mut count_query_builder = sqlx::query_scalar::<_, i64>(&count_query).bind(user_id);
        let mut duration_query_builder = sqlx::query_scalar::<_, i32>(&duration_query).bind(user_id);
        let mut entries_query_builder = sqlx::query_as::<_, TimeEntry>(&entries_query).bind(user_id);

        if let Some(start_date_str) = &filters.start_date {
            let start_date = OffsetDateTime::parse(start_date_str, &time::format_description::well_known::Iso8601::DEFAULT)
                .map_err(|_| TimeEntryError::ValidationError("Invalid start_date format".to_string()))?;
            count_query_builder = count_query_builder.bind(start_date);
            duration_query_builder = duration_query_builder.bind(start_date);
            entries_query_builder = entries_query_builder.bind(start_date);
        }
        
        if let Some(end_date_str) = &filters.end_date {
            let end_date = OffsetDateTime::parse(end_date_str, &time::format_description::well_known::Iso8601::DEFAULT)
                .map_err(|_| TimeEntryError::ValidationError("Invalid end_date format".to_string()))?;
            count_query_builder = count_query_builder.bind(end_date);
            duration_query_builder = duration_query_builder.bind(end_date);
            entries_query_builder = entries_query_builder.bind(end_date);
        }
        
        if let Some(project_id) = filters.project_id {
            count_query_builder = count_query_builder.bind(project_id);
            duration_query_builder = duration_query_builder.bind(project_id);
            entries_query_builder = entries_query_builder.bind(project_id);
        }
        
        if let Some(task_id) = filters.task_id {
            count_query_builder = count_query_builder.bind(task_id);
            duration_query_builder = duration_query_builder.bind(task_id);
            entries_query_builder = entries_query_builder.bind(task_id);
        }

        entries_query_builder = entries_query_builder.bind(limit as i32).bind(offset as i32);

        let total_count = count_query_builder.fetch_one(&self.pool).await?;
        let total_duration = duration_query_builder.fetch_one(&self.pool).await?;
        let entries = entries_query_builder.fetch_all(&self.pool).await?;
        Ok((entries, total_count, total_duration))
    }
}