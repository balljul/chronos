use regex::Regex;
use serde::{Deserialize, Serialize};
use time;
use uuid;
use validator::{Validate, ValidationError};

#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct RegisterRequest {
    pub name: Option<String>,
    #[validate(email(message = "Invalid email format"))]
    pub email: String,
    #[validate(custom(
        function = "validate_password",
        message = "Password must be at least 8 characters long, contain at least one uppercase letter, one lowercase letter, one number, and one special character"
    ))]
    pub password: String,
}

#[derive(Debug, Serialize)]
pub struct RegisterResponse {
    pub message: String,
    pub user: crate::app::models::user::UserResponse,
}

#[derive(Debug, Serialize)]
pub struct AuthError {
    pub error: String,
    pub details: Option<Vec<String>>,
}

impl AuthError {
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

pub fn validate_password(password: &str) -> Result<(), ValidationError> {
    if password.len() < 8 {
        return Err(ValidationError::new("password_too_short"));
    }

    let has_uppercase = Regex::new(r"[A-Z]").unwrap().is_match(password);
    let has_lowercase = Regex::new(r"[a-z]").unwrap().is_match(password);
    let has_number = Regex::new(r"\d").unwrap().is_match(password);
    let has_special = Regex::new(r"[!@#$%^&*(),.?\x22:{}|<>]")
        .unwrap()
        .is_match(password);

    if !has_uppercase || !has_lowercase || !has_number || !has_special {
        return Err(ValidationError::new("password_complexity"));
    }

    Ok(())
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
    #[validate(email(message = "Invalid email format"))]
    pub email: String,
}

#[derive(Debug, Serialize)]
pub struct ForgotPasswordResponse {
    pub message: String,
}

#[derive(Debug, Serialize, Deserialize, Validate)]
pub struct ResetPasswordRequest {
    pub token: String,
    #[validate(custom(
        function = "validate_password",
        message = "Password must be at least 8 characters long, contain at least one uppercase letter, one lowercase letter, one number, and one special character"
    ))]
    pub password: String,
}

#[derive(Debug, Serialize)]
pub struct ResetPasswordResponse {
    pub message: String,
}

#[derive(Debug, Serialize, Deserialize, Validate)]
pub struct ProfileUpdateRequest {
    pub name: Option<String>,
    #[validate(email(message = "Invalid email format"))]
    pub email: Option<String>,
    pub current_password: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct ProfileResponse {
    pub id: uuid::Uuid,
    pub name: Option<String>,
    pub email: String,
    #[serde(default, with = "time::serde::rfc3339::option")]
    pub created_at: Option<time::OffsetDateTime>,
    #[serde(default, with = "time::serde::rfc3339::option")]
    pub updated_at: Option<time::OffsetDateTime>,
}

#[derive(Debug, Serialize, Deserialize, Validate)]
pub struct ChangePasswordRequest {
    pub current_password: String,
    #[validate(custom(
        function = "validate_password",
        message = "Password must be at least 8 characters long, contain at least one uppercase letter, one lowercase letter, one number, and one special character"
    ))]
    pub new_password: String,
}

#[derive(Debug, Serialize)]
pub struct ChangePasswordResponse {
    pub message: String,
}
