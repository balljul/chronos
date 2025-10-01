use crate::app::validation::{
    validate_strong_password, validate_enhanced_email, validate_name, validate_token_format,
    sanitize_name_input, sanitize_html_input
};
use serde::{Deserialize, Serialize};
use time;
use uuid;
use validator::{Validate, ValidationError};

#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct RegisterRequest {
    #[validate(custom(function = "validate_name", message = "Name is required and must be valid"))]
    pub name: Option<String>,
    #[validate(custom(function = "validate_enhanced_email", message = "Please provide a valid email address"))]
    pub email: String,
    #[validate(custom(
        function = "validate_strong_password",
        message = "Password must meet security requirements"
    ))]
    pub password: String,
}

impl RegisterRequest {
    pub fn sanitize(&mut self) {
        if let Some(name) = &self.name {
            let sanitized = sanitize_name_input(name);
            self.name = if sanitized.is_empty() { None } else { Some(sanitized) };
        }
        self.email = sanitize_html_input(&self.email).trim().to_lowercase();
    }
}

#[derive(Debug, Serialize)]
pub struct RegisterResponse {
    pub message: String,
    pub user: crate::app::models::user::UserResponse,
}

// Re-export the enhanced AuthenticationError from errors module
pub use crate::app::errors::AuthenticationError as AuthError;

// Legacy support - keep the old AuthError struct for backward compatibility
#[derive(Debug, Serialize)]
pub struct LegacyAuthError {
    pub error: String,
    pub details: Option<Vec<String>>,
}

impl LegacyAuthError {
    pub fn new(error: &str) -> Self {
        Self {
            error: error.to_string(),
            details: None,
        }
    }

    pub fn with_details(error: &str, details: Vec<String>) -> Self {
        Self {
            error: error.to_string(),
            details: Some(details),
        }
    }

    pub fn validation_error(errors: &validator::ValidationErrors) -> Self {
        let details: Vec<String> = errors
            .field_errors()
            .iter()
            .flat_map(|(field, field_errors)| {
                field_errors.iter().map(move |error| {
                    format!(
                        "{}: {}",
                        field,
                        error.message.as_ref().unwrap_or(&"Invalid value".into())
                    )
                })
            })
            .collect();

        Self::with_details("Validation failed", details)
    }
}

// Conversion from new AuthenticationError to legacy format
impl From<crate::app::errors::AuthenticationError> for LegacyAuthError {
    fn from(auth_error: crate::app::errors::AuthenticationError) -> Self {
        let field_errors = auth_error.field_errors();
        if field_errors.is_empty() {
            LegacyAuthError::new(&auth_error.user_message())
        } else {
            let details: Vec<String> = field_errors
                .iter()
                .map(|fe| format!("{}: {}", fe.field, fe.message))
                .collect();
            LegacyAuthError::with_details(&auth_error.user_message(), details)
        }
    }
}

// Password validation is now handled by the validation module
// Keep this function for backward compatibility but delegate to the new implementation
pub fn validate_password(password: &str) -> Result<(), ValidationError> {
    validate_strong_password(password)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_password_validation() {
        // Valid password
        assert!(validate_password("MyPassw0rd!").is_ok());

        // Too short
        assert!(validate_password("Short1!").is_err());

        // Missing uppercase
        assert!(validate_password("mypassw0rd!").is_err());

        // Missing lowercase
        assert!(validate_password("MYPASSW0RD!").is_err());

        // Missing number
        assert!(validate_password("MyPassword!").is_err());

        // Missing special character
        assert!(validate_password("MyPassw0rd1").is_err());
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct ForgotPasswordRequest {
    #[validate(custom(function = "validate_enhanced_email", message = "Please provide a valid email address"))]
    pub email: String,
}

impl ForgotPasswordRequest {
    pub fn sanitize(&mut self) {
        self.email = sanitize_html_input(&self.email).trim().to_lowercase();
    }
}

#[derive(Debug, Serialize)]
pub struct ForgotPasswordResponse {
    pub message: String,
}

#[derive(Debug, Serialize, Deserialize, Validate)]
pub struct ResetPasswordRequest {
    #[validate(custom(function = "validate_token_format", message = "Invalid token format"))]
    pub token: String,
    #[validate(custom(
        function = "validate_strong_password",
        message = "Password must meet security requirements"
    ))]
    pub password: String,
}

impl ResetPasswordRequest {
    pub fn sanitize(&mut self) {
        self.token = sanitize_html_input(&self.token).trim().to_string();
    }
}

#[derive(Debug, Serialize)]
pub struct ResetPasswordResponse {
    pub message: String,
}

#[derive(Debug, Serialize, Deserialize, Validate)]
pub struct ProfileUpdateRequest {
    #[validate(custom(function = "validate_name", message = "Name must be valid"))]
    pub name: Option<String>,
    #[validate(custom(function = "validate_enhanced_email", message = "Please provide a valid email address"))]
    pub email: Option<String>,
    pub current_password: Option<String>,
}

impl ProfileUpdateRequest {
    pub fn sanitize(&mut self) {
        if let Some(name) = &self.name {
            let sanitized = sanitize_name_input(name);
            self.name = if sanitized.is_empty() { None } else { Some(sanitized) };
        }
        if let Some(email) = &self.email {
            let sanitized = sanitize_html_input(email).trim().to_lowercase();
            self.email = if sanitized.is_empty() { None } else { Some(sanitized) };
        }
    }
}

#[derive(Debug, Serialize)]
pub struct ProfileResponse {
    pub id: uuid::Uuid,
    pub name: Option<String>,
    pub email: String,
    pub created_at: Option<time::OffsetDateTime>,
    pub updated_at: Option<time::OffsetDateTime>,
}

#[derive(Debug, Serialize, Deserialize, Validate)]
pub struct ChangePasswordRequest {
    pub current_password: String,
    #[validate(custom(
        function = "validate_strong_password",
        message = "Password must meet security requirements"
    ))]
    pub new_password: String,
}

#[derive(Debug, Serialize)]
pub struct ChangePasswordResponse {
    pub message: String,
}
