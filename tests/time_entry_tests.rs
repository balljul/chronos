use chronos::app::models::time_entry::{
    CreateTimeEntryRequest, UpdateTimeEntryRequest, TimeEntryFilters, TimeEntryError
};
use serde_json;
use time::OffsetDateTime;
use uuid::Uuid;
use validator::Validate;

#[cfg(test)]
mod time_entry_tests {
    use super::*;

    #[tokio::test]
    async fn test_create_time_entry_request_validation() {
        // Valid request
        let valid_request = CreateTimeEntryRequest {
            description: Some("Working on new feature".to_string()),
            project_id: Some(Uuid::new_v4()),
            task_id: Some(Uuid::new_v4()),
            start_time: OffsetDateTime::now_utc(),
            end_time: None, // Running timer
        };

        assert!(valid_request.validate().is_ok());

        // Test serialization
        let json = serde_json::to_string(&valid_request).unwrap();
        assert!(json.contains("Working on new feature"));

        // Test deserialization
        let deserialized: CreateTimeEntryRequest = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.description, valid_request.description);
    }

    #[tokio::test]
    async fn test_create_time_entry_request_validation_errors() {
        // Description too long (> 1000 characters)
        let long_description = "a".repeat(1001);
        let invalid_request = CreateTimeEntryRequest {
            description: Some(long_description),
            project_id: None,
            task_id: None,
            start_time: OffsetDateTime::now_utc(),
            end_time: None,
        };

        let validation_result = invalid_request.validate();
        assert!(validation_result.is_err());
        let errors = validation_result.unwrap_err();
        assert!(errors.to_string().contains("Description cannot exceed 1000 characters"));
    }

    #[tokio::test]
    async fn test_update_time_entry_request_validation() {
        // Valid partial update
        let valid_request = UpdateTimeEntryRequest {
            description: Some("Updated description".to_string()),
            project_id: None, // Not updating project
            task_id: None,    // Not updating task
            start_time: None, // Not updating start time
            end_time: Some(OffsetDateTime::now_utc()),
        };

        assert!(valid_request.validate().is_ok());
    }

    #[tokio::test]
    async fn test_time_entry_filters_validation() {
        // Valid filters
        let valid_filters = TimeEntryFilters {
            start_date: Some(OffsetDateTime::now_utc() - time::Duration::days(7)),
            end_date: Some(OffsetDateTime::now_utc()),
            project_id: Some(Uuid::new_v4()),
            task_id: None,
            is_running: Some(false),
            page: Some(1),
            limit: Some(20),
            sort_by: Some("start_time".to_string()),
        };

        assert!(valid_filters.validate().is_ok());

        // Invalid page (< 1)
        let invalid_filters = TimeEntryFilters {
            start_date: None,
            end_date: None,
            project_id: None,
            task_id: None,
            is_running: None,
            page: Some(0), // Invalid
            limit: Some(20),
            sort_by: None,
        };

        let validation_result = invalid_filters.validate();
        assert!(validation_result.is_err());
        let errors = validation_result.unwrap_err();
        assert!(errors.to_string().contains("Page must be between 1 and 1000"));

        // Invalid limit (> 100)
        let invalid_filters = TimeEntryFilters {
            start_date: None,
            end_date: None,
            project_id: None,
            task_id: None,
            is_running: None,
            page: Some(1),
            limit: Some(101), // Invalid
            sort_by: None,
        };

        let validation_result = invalid_filters.validate();
        assert!(validation_result.is_err());
        let errors = validation_result.unwrap_err();
        assert!(errors.to_string().contains("Limit must be between 1 and 100"));
    }

    #[test]
    fn test_time_entry_error_display() {
        let error = TimeEntryError::NotFound;
        assert_eq!(error.to_string(), "Time entry not found");

        let error = TimeEntryError::Forbidden;
        assert_eq!(error.to_string(), "Time entry belongs to another user");

        let error = TimeEntryError::InvalidTimeRange;
        assert_eq!(error.to_string(), "End time must be after start time");

        let error = TimeEntryError::RunningTimerExists;
        assert_eq!(error.to_string(), "User already has a running timer");

        let error = TimeEntryError::TimerNotRunning;
        assert_eq!(error.to_string(), "Timer is not currently running");

        let error = TimeEntryError::ValidationError("Test validation error".to_string());
        assert_eq!(error.to_string(), "Validation error: Test validation error");
    }

    #[test]
    fn test_time_entry_model_calculations() {
        use chronos::app::models::time_entry::TimeEntry;

        let start_time = OffsetDateTime::now_utc();
        let end_time = start_time + time::Duration::hours(2);

        // Completed time entry
        let completed_entry = TimeEntry {
            id: Uuid::new_v4(),
            user_id: Uuid::new_v4(),
            description: Some("Test entry".to_string()),
            project_id: None,
            task_id: None,
            start_time,
            end_time: Some(end_time),
            duration: Some(7200), // 2 hours in seconds
            created_at: Some(OffsetDateTime::now_utc()),
            updated_at: Some(OffsetDateTime::now_utc()),
        };

        assert!(!completed_entry.is_running());
        assert_eq!(completed_entry.calculate_duration(), Some(7200));

        // Running time entry
        let running_entry = TimeEntry {
            id: Uuid::new_v4(),
            user_id: Uuid::new_v4(),
            description: Some("Running entry".to_string()),
            project_id: None,
            task_id: None,
            start_time,
            end_time: None,
            duration: None,
            created_at: Some(OffsetDateTime::now_utc()),
            updated_at: Some(OffsetDateTime::now_utc()),
        };

        assert!(running_entry.is_running());
        assert_eq!(running_entry.calculate_duration(), None);
        
        // Current duration should be > 0 for a running timer
        let current_duration = running_entry.current_duration();
        assert!(current_duration >= 0);
    }

    #[tokio::test]
    async fn test_time_entry_json_serialization() {
        use chronos::app::models::time_entry::{TimeEntry, TimeEntryResponse};

        let time_entry = TimeEntry {
            id: Uuid::new_v4(),
            user_id: Uuid::new_v4(),
            description: Some("JSON test entry".to_string()),
            project_id: Some(Uuid::new_v4()),
            task_id: None,
            start_time: OffsetDateTime::now_utc(),
            end_time: None,
            duration: None,
            created_at: Some(OffsetDateTime::now_utc()),
            updated_at: Some(OffsetDateTime::now_utc()),
        };

        // Test TimeEntry serialization
        let json = serde_json::to_string(&time_entry).unwrap();
        assert!(json.contains("JSON test entry"));

        // Test TimeEntryResponse conversion and serialization
        let response: TimeEntryResponse = time_entry.clone().into();
        assert_eq!(response.description, time_entry.description);
        assert_eq!(response.is_running, true);

        let response_json = serde_json::to_string(&response).unwrap();
        assert!(response_json.contains("JSON test entry"));
        assert!(response_json.contains("\"is_running\":true"));
    }
}