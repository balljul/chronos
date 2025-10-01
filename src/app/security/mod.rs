use serde::{Deserialize, Serialize};
use tracing::{error, warn, info};
use time::OffsetDateTime;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityEvent {
    pub event_type: String,
    pub timestamp: OffsetDateTime,
    pub ip_address: String,
    pub user_agent: Option<String>,
    pub user_id: Option<Uuid>,
    pub email: Option<String>,
    pub success: bool,
    pub details: Option<String>,
    pub severity: SecuritySeverity,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SecuritySeverity {
    Low,
    Medium,
    High,
    Critical,
}

impl SecurityEvent {
    pub fn new(
        event_type: impl Into<String>,
        ip_address: impl Into<String>,
        user_agent: Option<&str>,
        user_id: Option<Uuid>,
        email: Option<&str>,
        success: bool,
        details: Option<&str>,
    ) -> Self {
        let event_type_str = event_type.into();
        let severity = Self::determine_severity(&event_type_str, success);

        Self {
            event_type: event_type_str,
            timestamp: OffsetDateTime::now_utc(),
            ip_address: ip_address.into(),
            user_agent: user_agent.map(|s| s.to_string()),
            user_id,
            email: email.map(|s| s.to_string()),
            success,
            details: details.map(|s| s.to_string()),
            severity,
        }
    }

    fn determine_severity(event_type: &str, success: bool) -> SecuritySeverity {
        match (event_type, success) {
            // Critical security events
            ("multiple_failed_logins", false) => SecuritySeverity::Critical,
            ("account_locked", _) => SecuritySeverity::High,
            ("password_brute_force", false) => SecuritySeverity::Critical,
            ("token_manipulation_attempt", false) => SecuritySeverity::High,

            // High severity events
            ("login_failed", false) => SecuritySeverity::High,
            ("password_reset_abuse", false) => SecuritySeverity::High,
            ("rate_limit_exceeded", false) => SecuritySeverity::High,
            ("suspicious_activity", false) => SecuritySeverity::High,

            // Medium severity events
            ("login_success", true) => SecuritySeverity::Medium,
            ("password_changed", true) => SecuritySeverity::Medium,
            ("profile_updated", true) => SecuritySeverity::Medium,
            ("token_refreshed", true) => SecuritySeverity::Medium,

            // Low severity events
            ("user_registration", true) => SecuritySeverity::Low,
            ("profile_accessed", true) => SecuritySeverity::Low,
            ("logout_successful", true) => SecuritySeverity::Low,

            // Default to medium for unknown events
            _ => SecuritySeverity::Medium,
        }
    }

    pub fn log(&self) {
        let log_data = serde_json::to_string(self)
            .unwrap_or_else(|_| "Failed to serialize security event".to_string());

        match self.severity {
            SecuritySeverity::Critical => {
                error!(
                    event_type = %self.event_type,
                    ip_address = %self.ip_address,
                    user_id = ?self.user_id,
                    success = %self.success,
                    "CRITICAL SECURITY EVENT: {}",
                    log_data
                );
            }
            SecuritySeverity::High => {
                error!(
                    event_type = %self.event_type,
                    ip_address = %self.ip_address,
                    user_id = ?self.user_id,
                    success = %self.success,
                    "High severity security event: {}",
                    log_data
                );
            }
            SecuritySeverity::Medium => {
                warn!(
                    event_type = %self.event_type,
                    ip_address = %self.ip_address,
                    user_id = ?self.user_id,
                    success = %self.success,
                    "Security event: {}",
                    log_data
                );
            }
            SecuritySeverity::Low => {
                info!(
                    event_type = %self.event_type,
                    ip_address = %self.ip_address,
                    user_id = ?self.user_id,
                    success = %self.success,
                    "Security event: {}",
                    log_data
                );
            }
        }
    }
}

// Enhanced security event logging function
pub fn log_security_event(
    event_type: impl Into<String>,
    ip_address: impl Into<String>,
    user_agent: Option<&str>,
    user_id: Option<&str>,
    email: Option<&str>,
    success: bool,
    details: Option<&str>,
) {
    let user_uuid = user_id.and_then(|id| Uuid::parse_str(id).ok());

    let event = SecurityEvent::new(
        event_type,
        ip_address,
        user_agent,
        user_uuid,
        email,
        success,
        details,
    );

    event.log();
}

// Additional security logging helpers
pub fn log_authentication_attempt(
    ip_address: &str,
    user_agent: Option<&str>,
    email: &str,
    success: bool,
    failure_reason: Option<&str>,
) {
    let event_type = if success {
        "login_success"
    } else {
        "login_failed"
    };

    log_security_event(
        event_type,
        ip_address,
        user_agent,
        None,
        Some(email),
        success,
        failure_reason,
    );
}

pub fn log_validation_failure(
    ip_address: &str,
    user_agent: Option<&str>,
    endpoint: &str,
    validation_errors: &str,
) {
    log_security_event(
        "validation_failed",
        ip_address,
        user_agent,
        None,
        None,
        false,
        Some(&format!("Endpoint: {}, Errors: {}", endpoint, validation_errors)),
    );
}

pub fn log_rate_limit_exceeded(
    ip_address: &str,
    user_agent: Option<&str>,
    endpoint: &str,
    user_id: Option<&str>,
) {
    log_security_event(
        "rate_limit_exceeded",
        ip_address,
        user_agent,
        user_id,
        None,
        false,
        Some(&format!("Endpoint: {}", endpoint)),
    );
}

pub fn log_suspicious_activity(
    ip_address: &str,
    user_agent: Option<&str>,
    activity_type: &str,
    details: &str,
) {
    log_security_event(
        "suspicious_activity",
        ip_address,
        user_agent,
        None,
        None,
        false,
        Some(&format!("Activity: {}, Details: {}", activity_type, details)),
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_security_event_creation() {
        let event = SecurityEvent::new(
            "login_failed",
            "192.168.1.1",
            Some("Mozilla/5.0..."),
            None,
            Some("test@example.com"),
            false,
            Some("Invalid credentials"),
        );

        assert_eq!(event.event_type, "login_failed");
        assert_eq!(event.ip_address, "192.168.1.1");
        assert!(!event.success);
        assert!(matches!(event.severity, SecuritySeverity::High));
    }

    #[test]
    fn test_severity_determination() {
        assert!(matches!(
            SecurityEvent::determine_severity("multiple_failed_logins", false),
            SecuritySeverity::Critical
        ));
        assert!(matches!(
            SecurityEvent::determine_severity("login_failed", false),
            SecuritySeverity::High
        ));
        assert!(matches!(
            SecurityEvent::determine_severity("login_success", true),
            SecuritySeverity::Medium
        ));
        assert!(matches!(
            SecurityEvent::determine_severity("user_registration", true),
            SecuritySeverity::Low
        ));
    }
}