use serde::{Deserialize, Serialize};
use time::OffsetDateTime;
use uuid::Uuid;
use validator::Validate;

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct TimeEntry {
    pub id: Uuid,
    pub user_id: Uuid,
    pub description: Option<String>,
    pub project_id: Option<Uuid>,
    pub task_id: Option<Uuid>,
    pub start_time: OffsetDateTime,
    pub end_time: Option<OffsetDateTime>,
    pub duration: Option<i32>, // Duration in seconds
    pub created_at: Option<OffsetDateTime>,
    pub updated_at: Option<OffsetDateTime>,
}

#[derive(Debug, Serialize, Deserialize, Validate)]
pub struct CreateTimeEntryRequest {
    #[validate(length(max = 1000, message = "Description cannot exceed 1000 characters"))]
    pub description: Option<String>,
    pub project_id: Option<Uuid>,
    pub task_id: Option<Uuid>,
    pub start_time: OffsetDateTime,
    pub end_time: Option<OffsetDateTime>,
}

#[derive(Debug, Serialize, Deserialize, Validate)]
pub struct UpdateTimeEntryRequest {
    #[validate(length(max = 1000, message = "Description cannot exceed 1000 characters"))]
    pub description: Option<String>,
    pub project_id: Option<Uuid>,
    pub task_id: Option<Uuid>,
    pub start_time: Option<OffsetDateTime>,
    pub end_time: Option<OffsetDateTime>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TimeEntryResponse {
    pub id: Uuid,
    pub description: Option<String>,
    pub project_id: Option<Uuid>,
    pub task_id: Option<Uuid>,
    pub start_time: OffsetDateTime,
    pub end_time: Option<OffsetDateTime>,
    pub duration: Option<i32>,
    pub is_running: bool,
    pub created_at: Option<OffsetDateTime>,
    pub updated_at: Option<OffsetDateTime>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TimeEntriesListResponse {
    pub entries: Vec<TimeEntryResponse>,
    pub total_count: i64,
    pub total_duration: i32, // Total duration in seconds for filtered results
    pub page: i64,
    pub per_page: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct TimeEntryFilters {
    pub start_date: Option<OffsetDateTime>,
    pub end_date: Option<OffsetDateTime>,
    pub project_id: Option<Uuid>,
    pub task_id: Option<Uuid>,
    pub is_running: Option<bool>,
    #[validate(range(min = 1, max = 1000, message = "Page must be between 1 and 1000"))]
    pub page: Option<i64>,
    #[validate(range(min = 1, max = 100, message = "Limit must be between 1 and 100"))]
    pub limit: Option<i64>,
    pub sort_by: Option<String>,
}

impl From<TimeEntry> for TimeEntryResponse {
    fn from(entry: TimeEntry) -> Self {
        Self {
            id: entry.id,
            description: entry.description,
            project_id: entry.project_id,
            task_id: entry.task_id,
            start_time: entry.start_time,
            end_time: entry.end_time,
            duration: entry.duration,
            is_running: entry.end_time.is_none(),
            created_at: entry.created_at,
            updated_at: entry.updated_at,
        }
    }
}

impl TimeEntry {
    pub fn is_running(&self) -> bool {
        self.end_time.is_none()
    }

    pub fn calculate_duration(&self) -> Option<i32> {
        if let Some(end_time) = self.end_time {
            Some((end_time - self.start_time).whole_seconds() as i32)
        } else {
            None
        }
    }

    pub fn current_duration(&self) -> i32 {
        if let Some(end_time) = self.end_time {
            (end_time - self.start_time).whole_seconds() as i32
        } else {
            (OffsetDateTime::now_utc() - self.start_time).whole_seconds() as i32
        }
    }
}

#[derive(Debug, thiserror::Error)]
pub enum TimeEntryError {
    #[error("Time entry not found")]
    NotFound,
    #[error("Time entry belongs to another user")]
    Forbidden,
    #[error("End time must be after start time")]
    InvalidTimeRange,
    #[error("User already has a running timer")]
    RunningTimerExists,
    #[error("Timer is not currently running")]
    TimerNotRunning,
    #[error("Database error: {0}")]
    DatabaseError(#[from] sqlx::Error),
    #[error("Validation error: {0}")]
    ValidationError(String),
}