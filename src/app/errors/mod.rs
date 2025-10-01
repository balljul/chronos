use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde::{Deserialize, Serialize};
use validator::ValidationErrors;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorResponse {
    pub error: ErrorDetail,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorDetail {
    pub code: String,
    pub message: String,
    pub details: Option<Vec<FieldError>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FieldError {
    pub field: String,
    pub message: String,
}

// Custom error types for different authentication scenarios
#[derive(Debug, Clone)]
pub enum AuthenticationError {
    // Validation errors
    ValidationFailed {
        errors: ValidationErrors,
    },
    InvalidInput {
        field: String,
        message: String,
    },

    // Authentication/Authorization errors
    InvalidCredentials,
    AccountLocked {
        lockout_duration: Option<u64>,
    },
    TokenExpired,
    TokenInvalid {
        reason: String,
    },
    TokenRevoked,
    InsufficientPermissions {
        required_role: String,
    },

    // Rate limiting
    RateLimitExceeded {
        retry_after: u64,
    },
    TooManyAttempts {
        lockout_duration: u64,
    },

    // User management
    UserNotFound,
    UserAlreadyExists {
        field: String,
    },
    AccountDisabled,

    // Password related
    WeakPassword {
        requirements: Vec<String>,
    },
    PasswordReused,
    PasswordResetTokenInvalid,
    PasswordResetTokenExpired,

    // System errors
    DatabaseError {
        context: String,
    },
    ExternalServiceError {
        service: String,
        message: String,
    },
    InternalError {
        message: String,
    },
    ConfigurationError {
        message: String,
    },
}

impl AuthenticationError {
    pub fn error_code(&self) -> &'static str {
        match self {
            AuthenticationError::ValidationFailed { .. } => "VALIDATION_FAILED",
            AuthenticationError::InvalidInput { .. } => "INVALID_INPUT",
            AuthenticationError::InvalidCredentials => "INVALID_CREDENTIALS",
            AuthenticationError::AccountLocked { .. } => "ACCOUNT_LOCKED",
            AuthenticationError::TokenExpired => "TOKEN_EXPIRED",
            AuthenticationError::TokenInvalid { .. } => "TOKEN_INVALID",
            AuthenticationError::TokenRevoked => "TOKEN_REVOKED",
            AuthenticationError::InsufficientPermissions { .. } => "INSUFFICIENT_PERMISSIONS",
            AuthenticationError::RateLimitExceeded { .. } => "RATE_LIMIT_EXCEEDED",
            AuthenticationError::TooManyAttempts { .. } => "TOO_MANY_ATTEMPTS",
            AuthenticationError::UserNotFound => "USER_NOT_FOUND",
            AuthenticationError::UserAlreadyExists { .. } => "USER_ALREADY_EXISTS",
            AuthenticationError::AccountDisabled => "ACCOUNT_DISABLED",
            AuthenticationError::WeakPassword { .. } => "WEAK_PASSWORD",
            AuthenticationError::PasswordReused => "PASSWORD_REUSED",
            AuthenticationError::PasswordResetTokenInvalid => "PASSWORD_RESET_TOKEN_INVALID",
            AuthenticationError::PasswordResetTokenExpired => "PASSWORD_RESET_TOKEN_EXPIRED",
            AuthenticationError::DatabaseError { .. } => "DATABASE_ERROR",
            AuthenticationError::ExternalServiceError { .. } => "EXTERNAL_SERVICE_ERROR",
            AuthenticationError::InternalError { .. } => "INTERNAL_ERROR",
            AuthenticationError::ConfigurationError { .. } => "CONFIGURATION_ERROR",
        }
    }

    pub fn user_message(&self) -> String {
        match self {
            AuthenticationError::ValidationFailed { .. } => {
                "The provided data contains validation errors".to_string()
            }
            AuthenticationError::InvalidInput { message, .. } => message.clone(),
            AuthenticationError::InvalidCredentials => {
                "Invalid email or password".to_string()
            }
            AuthenticationError::AccountLocked { lockout_duration } => {
                if let Some(duration) = lockout_duration {
                    format!(
                        "Account is temporarily locked due to too many failed attempts. Please try again in {} seconds",
                        duration
                    )
                } else {
                    "Account is temporarily locked due to too many failed attempts".to_string()
                }
            }
            AuthenticationError::TokenExpired => {
                "Your session has expired. Please log in again".to_string()
            }
            AuthenticationError::TokenInvalid { .. } => {
                "Invalid authentication token".to_string()
            }
            AuthenticationError::TokenRevoked => {
                "Authentication token has been revoked".to_string()
            }
            AuthenticationError::InsufficientPermissions { .. } => {
                "You don't have permission to perform this action".to_string()
            }
            AuthenticationError::RateLimitExceeded { retry_after } => {
                format!("Too many requests. Please try again in {} seconds", retry_after)
            }
            AuthenticationError::TooManyAttempts { lockout_duration } => {
                format!(
                    "Too many failed attempts. Account locked for {} seconds",
                    lockout_duration
                )
            }
            AuthenticationError::UserNotFound => {
                "User not found".to_string()
            }
            AuthenticationError::UserAlreadyExists { field } => {
                format!("An account with this {} already exists", field)
            }
            AuthenticationError::AccountDisabled => {
                "Account has been disabled. Please contact support".to_string()
            }
            AuthenticationError::WeakPassword { requirements } => {
                format!("Password does not meet security requirements: {}", requirements.join(", "))
            }
            AuthenticationError::PasswordReused => {
                "New password must be different from your current password".to_string()
            }
            AuthenticationError::PasswordResetTokenInvalid => {
                "Password reset link is invalid or has already been used".to_string()
            }
            AuthenticationError::PasswordResetTokenExpired => {
                "Password reset link has expired. Please request a new one".to_string()
            }
            AuthenticationError::DatabaseError { .. } => {
                "A temporary service issue occurred. Please try again later".to_string()
            }
            AuthenticationError::ExternalServiceError { service, .. } => {
                format!("External service ({}) is temporarily unavailable", service)
            }
            AuthenticationError::InternalError { .. } => {
                "An internal error occurred. Please try again later".to_string()
            }
            AuthenticationError::ConfigurationError { .. } => {
                "Service is temporarily unavailable due to configuration issues".to_string()
            }
        }
    }

    pub fn http_status(&self) -> StatusCode {
        match self {
            AuthenticationError::ValidationFailed { .. } => StatusCode::BAD_REQUEST,
            AuthenticationError::InvalidInput { .. } => StatusCode::BAD_REQUEST,
            AuthenticationError::InvalidCredentials => StatusCode::UNAUTHORIZED,
            AuthenticationError::AccountLocked { .. } => StatusCode::LOCKED,
            AuthenticationError::TokenExpired => StatusCode::UNAUTHORIZED,
            AuthenticationError::TokenInvalid { .. } => StatusCode::UNAUTHORIZED,
            AuthenticationError::TokenRevoked => StatusCode::UNAUTHORIZED,
            AuthenticationError::InsufficientPermissions { .. } => StatusCode::FORBIDDEN,
            AuthenticationError::RateLimitExceeded { .. } => StatusCode::TOO_MANY_REQUESTS,
            AuthenticationError::TooManyAttempts { .. } => StatusCode::TOO_MANY_REQUESTS,
            AuthenticationError::UserNotFound => StatusCode::NOT_FOUND,
            AuthenticationError::UserAlreadyExists { .. } => StatusCode::CONFLICT,
            AuthenticationError::AccountDisabled => StatusCode::FORBIDDEN,
            AuthenticationError::WeakPassword { .. } => StatusCode::BAD_REQUEST,
            AuthenticationError::PasswordReused => StatusCode::BAD_REQUEST,
            AuthenticationError::PasswordResetTokenInvalid => StatusCode::BAD_REQUEST,
            AuthenticationError::PasswordResetTokenExpired => StatusCode::BAD_REQUEST,
            AuthenticationError::DatabaseError { .. } => StatusCode::INTERNAL_SERVER_ERROR,
            AuthenticationError::ExternalServiceError { .. } => StatusCode::SERVICE_UNAVAILABLE,
            AuthenticationError::InternalError { .. } => StatusCode::INTERNAL_SERVER_ERROR,
            AuthenticationError::ConfigurationError { .. } => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }

    pub fn field_errors(&self) -> Vec<FieldError> {
        match self {
            AuthenticationError::ValidationFailed { errors } => {
                validation_errors_to_field_errors(errors)
            }
            AuthenticationError::InvalidInput { field, message } => {
                vec![FieldError {
                    field: field.clone(),
                    message: message.clone(),
                }]
            }
            AuthenticationError::UserAlreadyExists { field } => {
                vec![FieldError {
                    field: field.clone(),
                    message: format!("This {} is already registered", field),
                }]
            }
            AuthenticationError::WeakPassword { requirements } => {
                vec![FieldError {
                    field: "password".to_string(),
                    message: format!("Password requirements: {}", requirements.join(", ")),
                }]
            }
            _ => vec![],
        }
    }

    pub fn should_log(&self) -> bool {
        match self {
            AuthenticationError::DatabaseError { .. } |
            AuthenticationError::ExternalServiceError { .. } |
            AuthenticationError::InternalError { .. } |
            AuthenticationError::ConfigurationError { .. } |
            AuthenticationError::TooManyAttempts { .. } |
            AuthenticationError::RateLimitExceeded { .. } |
            AuthenticationError::AccountLocked { .. } => true,
            _ => false,
        }
    }

    pub fn log_level(&self) -> &'static str {
        match self {
            AuthenticationError::DatabaseError { .. } |
            AuthenticationError::ExternalServiceError { .. } |
            AuthenticationError::InternalError { .. } |
            AuthenticationError::ConfigurationError { .. } => "error",
            AuthenticationError::TooManyAttempts { .. } |
            AuthenticationError::RateLimitExceeded { .. } |
            AuthenticationError::AccountLocked { .. } |
            AuthenticationError::InvalidCredentials => "warn",
            _ => "info",
        }
    }
}

impl IntoResponse for AuthenticationError {
    fn into_response(self) -> Response {
        let field_errors = self.field_errors();
        let details = if field_errors.is_empty() {
            None
        } else {
            Some(field_errors)
        };

        let error_response = ErrorResponse {
            error: ErrorDetail {
                code: self.error_code().to_string(),
                message: self.user_message(),
                details,
            },
        };

        let status = self.http_status();
        (status, Json(error_response)).into_response()
    }
}

impl std::fmt::Display for AuthenticationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}: {}", self.error_code(), self.user_message())
    }
}

impl std::error::Error for AuthenticationError {}

// Helper function to convert validator errors to field errors
fn validation_errors_to_field_errors(errors: &ValidationErrors) -> Vec<FieldError> {
    let mut field_errors = Vec::new();

    for (field_name, field_errors_list) in errors.field_errors() {
        for field_error in field_errors_list {
            let message = field_error
                .message
                .as_ref()
                .map(|cow| cow.to_string())
                .unwrap_or_else(|| "Invalid value".to_string());

            field_errors.push(FieldError {
                field: field_name.to_string(),
                message,
            });
        }
    }

    field_errors
}

// Convenience constructors for common errors
impl AuthenticationError {
    pub fn validation_failed(errors: ValidationErrors) -> Self {
        Self::ValidationFailed { errors }
    }

    pub fn invalid_credentials() -> Self {
        Self::InvalidCredentials
    }

    pub fn token_expired() -> Self {
        Self::TokenExpired
    }

    pub fn token_invalid(reason: impl Into<String>) -> Self {
        Self::TokenInvalid {
            reason: reason.into(),
        }
    }

    pub fn rate_limit_exceeded(retry_after: u64) -> Self {
        Self::RateLimitExceeded { retry_after }
    }

    pub fn user_not_found() -> Self {
        Self::UserNotFound
    }

    pub fn user_already_exists(field: impl Into<String>) -> Self {
        Self::UserAlreadyExists {
            field: field.into(),
        }
    }

    pub fn weak_password(requirements: Vec<String>) -> Self {
        Self::WeakPassword { requirements }
    }

    pub fn database_error(context: impl Into<String>) -> Self {
        Self::DatabaseError {
            context: context.into(),
        }
    }

    pub fn internal_error(message: impl Into<String>) -> Self {
        Self::InternalError {
            message: message.into(),
        }
    }

    pub fn invalid_input(field: impl Into<String>, message: impl Into<String>) -> Self {
        Self::InvalidInput {
            field: field.into(),
            message: message.into(),
        }
    }

    // Legacy compatibility methods
    pub fn new(message: impl Into<String>) -> Self {
        Self::InternalError {
            message: message.into(),
        }
    }

    pub fn validation_error(errors: &ValidationErrors) -> Self {
        Self::validation_failed(errors.clone())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_response_format() {
        let error = AuthenticationError::invalid_credentials();
        let response = error.into_response();
        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    }

    #[test]
    fn test_error_codes() {
        assert_eq!(AuthenticationError::invalid_credentials().error_code(), "INVALID_CREDENTIALS");
        assert_eq!(AuthenticationError::token_expired().error_code(), "TOKEN_EXPIRED");
        assert_eq!(AuthenticationError::user_not_found().error_code(), "USER_NOT_FOUND");
    }

    #[test]
    fn test_http_status_codes() {
        assert_eq!(AuthenticationError::invalid_credentials().http_status(), StatusCode::UNAUTHORIZED);
        assert_eq!(AuthenticationError::user_not_found().http_status(), StatusCode::NOT_FOUND);
        assert_eq!(AuthenticationError::rate_limit_exceeded(60).http_status(), StatusCode::TOO_MANY_REQUESTS);
    }

    #[test]
    fn test_should_log() {
        assert!(AuthenticationError::database_error("connection failed").should_log());
        assert!(AuthenticationError::internal_error("unexpected error").should_log());
        assert!(!AuthenticationError::invalid_credentials().should_log());
    }
}