use crate::app::models::time_entry::{
    TimeEntry, TimeEntryError, CreateTimeEntryRequest, UpdateTimeEntryRequest, TimeEntryFilters
};
use crate::app::repositories::time_entry_repository::TimeEntryRepository;
use time::OffsetDateTime;
use uuid::Uuid;
use validator::Validate;

#[derive(Clone)]
pub struct TimeEntryService {
    repository: TimeEntryRepository,
}

impl TimeEntryService {
    pub fn new(repository: TimeEntryRepository) -> Self {
        Self { repository }
    }

    pub async fn create_time_entry(
        &self,
        user_id: Uuid,
        request: CreateTimeEntryRequest,
    ) -> Result<TimeEntry, TimeEntryError> {
        if let Err(validation_errors) = request.validate() {
            return Err(TimeEntryError::ValidationError(validation_errors.to_string()));
        }

        if let Some(end_time) = request.end_time {
            if end_time <= request.start_time {
                return Err(TimeEntryError::InvalidTimeRange);
            }
        }

        let entry = TimeEntry {
            id: Uuid::new_v4(),
            user_id,
            description: request.description,
            project_id: request.project_id,
            task_id: request.task_id,
            start_time: request.start_time,
            end_time: request.end_time,
            duration: None,
            created_at: Some(OffsetDateTime::now_utc()),
            updated_at: Some(OffsetDateTime::now_utc()),
        };

        self.repository.create(&entry).await
    }

    pub async fn start_timer(
        &self,
        user_id: Uuid,
        description: Option<String>,
        project_id: Option<Uuid>,
        task_id: Option<Uuid>,
    ) -> Result<TimeEntry, TimeEntryError> {
        let request = CreateTimeEntryRequest {
            description,
            project_id,
            task_id,
            start_time: OffsetDateTime::now_utc(),
            end_time: None,
        };

        self.create_time_entry(user_id, request).await
    }

    pub async fn get_time_entry(&self, id: Uuid, user_id: Uuid) -> Result<TimeEntry, TimeEntryError> {
        self.repository.find_by_id(id, user_id).await
    }

    pub async fn update_time_entry(
        &self,
        id: Uuid,
        user_id: Uuid,
        request: UpdateTimeEntryRequest,
    ) -> Result<TimeEntry, TimeEntryError> {
        if let Err(validation_errors) = request.validate() {
            return Err(TimeEntryError::ValidationError(validation_errors.to_string()));
        }

        let existing_entry = self.repository.find_by_id_any_user(id).await?;
        
        if existing_entry.user_id != user_id {
            return Err(TimeEntryError::Forbidden);
        }

        let updated_entry = TimeEntry {
            id,
            user_id,
            description: request.description.or(existing_entry.description),
            project_id: request.project_id.or(existing_entry.project_id),
            task_id: request.task_id.or(existing_entry.task_id),
            start_time: request.start_time.unwrap_or(existing_entry.start_time),
            end_time: request.end_time.or(existing_entry.end_time),
            duration: None,
            created_at: existing_entry.created_at,
            updated_at: Some(OffsetDateTime::now_utc()),
        };

        self.repository.update(id, user_id, &updated_entry).await
    }

    pub async fn delete_time_entry(&self, id: Uuid, user_id: Uuid) -> Result<(), TimeEntryError> {
        let entry = self.repository.find_by_id_any_user(id).await?;
        
        if entry.user_id != user_id {
            return Err(TimeEntryError::Forbidden);
        }

        self.repository.delete(id, user_id).await
    }

    pub async fn stop_timer(&self, id: Uuid, user_id: Uuid) -> Result<TimeEntry, TimeEntryError> {
        let entry = self.repository.find_by_id_any_user(id).await?;
        
        if entry.user_id != user_id {
            return Err(TimeEntryError::Forbidden);
        }

        if entry.end_time.is_some() {
            return Err(TimeEntryError::TimerNotRunning);
        }

        self.repository.stop_timer(id, user_id).await
    }

    pub async fn get_current_timer(&self, user_id: Uuid) -> Result<TimeEntry, TimeEntryError> {
        self.repository.find_current_timer(user_id).await
    }

    pub async fn list_time_entries(
        &self,
        user_id: Uuid,
        filters: TimeEntryFilters,
    ) -> Result<(Vec<TimeEntry>, i64, i32), TimeEntryError> {
        if let Err(validation_errors) = filters.validate() {
            return Err(TimeEntryError::ValidationError(validation_errors.to_string()));
        }

        if let (Some(start_date_str), Some(end_date_str)) = (&filters.start_date, &filters.end_date) {
            match (OffsetDateTime::parse(start_date_str, &time::format_description::well_known::Iso8601::DEFAULT), 
                   OffsetDateTime::parse(end_date_str, &time::format_description::well_known::Iso8601::DEFAULT)) {
                (Ok(start_date), Ok(end_date)) => {
                    if end_date <= start_date {
                        return Err(TimeEntryError::ValidationError(
                            "End date must be after start date".to_string()
                        ));
                    }
                }
                _ => {
                    return Err(TimeEntryError::ValidationError(
                        "Invalid date format. Use ISO 8601 format".to_string()
                    ));
                }
            }
        }

        self.repository.find_with_filters(user_id, &filters).await
    }
}