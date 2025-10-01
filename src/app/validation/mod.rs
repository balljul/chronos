use ammonia::clean;
use regex::Regex;
use std::sync::OnceLock;
use validator::{Validate, ValidationError};

// Common password patterns to check against
static COMMON_PASSWORDS: &[&str] = &[
    "password", "123456", "123456789", "12345678", "12345", "1234567", "password123",
    "admin", "qwerty", "letmein", "welcome", "monkey", "dragon", "master", "654321",
    "111111", "123123", "1234567890", "iloveyou"
];

// Regex patterns for validation
static EMAIL_REGEX: OnceLock<Regex> = OnceLock::new();
static USERNAME_REGEX: OnceLock<Regex> = OnceLock::new();
static STRONG_PASSWORD_REGEX: OnceLock<Regex> = OnceLock::new();

fn get_email_regex() -> &'static Regex {
    EMAIL_REGEX.get_or_init(|| {
        Regex::new(r"^[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}$").unwrap()
    })
}

fn get_username_regex() -> &'static Regex {
    USERNAME_REGEX.get_or_init(|| {
        Regex::new(r"^[a-zA-Z0-9_-]{3,30}$").unwrap()
    })
}

fn get_strong_password_regex() -> &'static Regex {
    STRONG_PASSWORD_REGEX.get_or_init(|| {
        Regex::new(r"^(?=.*[a-z])(?=.*[A-Z])(?=.*\d)(?=.*[@$!%*?&])[A-Za-z\d@$!%*?&]").unwrap()
    })
}

// Custom validation functions
pub fn validate_strong_password(password: &str) -> Result<(), ValidationError> {
    // Length check
    if password.len() < 8 {
        let mut error = ValidationError::new("password_too_short");
        error.message = Some("Password must be at least 8 characters long".into());
        return Err(error);
    }

    if password.len() > 128 {
        let mut error = ValidationError::new("password_too_long");
        error.message = Some("Password must not exceed 128 characters".into());
        return Err(error);
    }

    // Check for common passwords
    let lower_password = password.to_lowercase();
    for common_pwd in COMMON_PASSWORDS {
        if lower_password.contains(common_pwd) {
            let mut error = ValidationError::new("password_too_common");
            error.message = Some("Password contains common patterns and is not secure".into());
            return Err(error);
        }
    }

    // Complexity check using regex
    if !get_strong_password_regex().is_match(password) {
        let mut error = ValidationError::new("password_complexity");
        error.message = Some("Password must contain at least one uppercase letter, one lowercase letter, one number, and one special character (@$!%*?&)".into());
        return Err(error);
    }

    // Check for sequential or repeated characters
    if has_sequential_chars(password) || has_repeated_chars(password) {
        let mut error = ValidationError::new("password_pattern");
        error.message = Some("Password must not contain sequential or repeated character patterns".into());
        return Err(error);
    }

    Ok(())
}

pub fn validate_enhanced_email(email: &str) -> Result<(), ValidationError> {
    if email.is_empty() {
        let mut error = ValidationError::new("email_required");
        error.message = Some("Email address is required".into());
        return Err(error);
    }

    if email.len() > 254 {
        let mut error = ValidationError::new("email_too_long");
        error.message = Some("Email address must not exceed 254 characters".into());
        return Err(error);
    }

    if !get_email_regex().is_match(email) {
        let mut error = ValidationError::new("email_invalid");
        error.message = Some("Please provide a valid email address".into());
        return Err(error);
    }

    // Check for potentially suspicious patterns
    let lower_email = email.to_lowercase();
    if lower_email.contains("..") || lower_email.starts_with('.') || lower_email.ends_with('.') {
        let mut error = ValidationError::new("email_invalid");
        error.message = Some("Email address contains invalid patterns".into());
        return Err(error);
    }

    Ok(())
}

pub fn validate_username(username: &str) -> Result<(), ValidationError> {
    if username.is_empty() {
        let mut error = ValidationError::new("username_required");
        error.message = Some("Username is required".into());
        return Err(error);
    }

    if !get_username_regex().is_match(username) {
        let mut error = ValidationError::new("username_invalid");
        error.message = Some("Username must be 3-30 characters long and contain only letters, numbers, hyphens, and underscores".into());
        return Err(error);
    }

    Ok(())
}

pub fn validate_name(name: &str) -> Result<(), ValidationError> {
    if name.is_empty() {
        let mut error = ValidationError::new("name_required");
        error.message = Some("Name is required".into());
        return Err(error);
    }

    if name.len() > 100 {
        let mut error = ValidationError::new("name_too_long");
        error.message = Some("Name must not exceed 100 characters".into());
        return Err(error);
    }

    // Check for potentially malicious content
    if name.contains('<') || name.contains('>') || name.contains('&') {
        let mut error = ValidationError::new("name_invalid_chars");
        error.message = Some("Name contains invalid characters".into());
        return Err(error);
    }

    Ok(())
}

pub fn validate_token_format(token: &str) -> Result<(), ValidationError> {
    if token.is_empty() {
        let mut error = ValidationError::new("token_required");
        error.message = Some("Token is required".into());
        return Err(error);
    }

    // Basic JWT format check (header.payload.signature)
    let parts: Vec<&str> = token.split('.').collect();
    if parts.len() != 3 {
        let mut error = ValidationError::new("token_invalid_format");
        error.message = Some("Token format is invalid".into());
        return Err(error);
    }

    // Check for reasonable length (JWT tokens are typically between 200-2000 chars)
    if token.len() < 50 || token.len() > 4000 {
        let mut error = ValidationError::new("token_invalid_length");
        error.message = Some("Token length is invalid".into());
        return Err(error);
    }

    Ok(())
}

// Input sanitization functions
pub fn sanitize_html_input(input: &str) -> String {
    clean(input)
}

pub fn sanitize_text_input(input: &str) -> String {
    input
        .chars()
        .filter(|&c| c.is_alphanumeric() || c.is_whitespace() || "-_@.!?".contains(c))
        .collect()
}

pub fn sanitize_name_input(input: &str) -> String {
    input
        .chars()
        .filter(|&c| c.is_alphabetic() || c.is_whitespace() || "'-".contains(c))
        .collect::<String>()
        .trim()
        .to_string()
}

// Helper functions
fn has_sequential_chars(password: &str) -> bool {
    let chars: Vec<char> = password.chars().collect();
    if chars.len() < 3 {
        return false;
    }

    for window in chars.windows(3) {
        if let [a, b, c] = window {
            // Check for ascending sequence
            if (*a as u32) + 1 == (*b as u32) && (*b as u32) + 1 == (*c as u32) {
                return true;
            }
            // Check for descending sequence
            if (*a as u32).saturating_sub(1) == (*b as u32) && (*b as u32).saturating_sub(1) == (*c as u32) {
                return true;
            }
        }
    }
    false
}

fn has_repeated_chars(password: &str) -> bool {
    let chars: Vec<char> = password.chars().collect();
    if chars.len() < 3 {
        return false;
    }

    for window in chars.windows(3) {
        if let [a, b, c] = window {
            if a == b && b == c {
                return true;
            }
        }
    }
    false
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_strong_password_validation() {
        // Valid passwords
        assert!(validate_strong_password("MyStr0ng!Pass").is_ok());
        assert!(validate_strong_password("C0mplex&Secure!").is_ok());

        // Invalid passwords
        assert!(validate_strong_password("short").is_err());
        assert!(validate_strong_password("password123").is_err());
        assert!(validate_strong_password("NoNumbers!").is_err());
        assert!(validate_strong_password("nonumbersorspecial").is_err());
        assert!(validate_strong_password("123456789").is_err());
        assert!(validate_strong_password("Abc1234!").is_err()); // sequential
        assert!(validate_strong_password("Aaa1234!").is_err()); // repeated
    }

    #[test]
    fn test_enhanced_email_validation() {
        // Valid emails
        assert!(validate_enhanced_email("user@example.com").is_ok());
        assert!(validate_enhanced_email("test.email+tag@domain.co.uk").is_ok());

        // Invalid emails
        assert!(validate_enhanced_email("").is_err());
        assert!(validate_enhanced_email("not-an-email").is_err());
        assert!(validate_enhanced_email("user@").is_err());
        assert!(validate_enhanced_email("@domain.com").is_err());
        assert!(validate_enhanced_email("user..name@domain.com").is_err());
        assert!(validate_enhanced_email(".user@domain.com").is_err());
    }

    #[test]
    fn test_username_validation() {
        // Valid usernames
        assert!(validate_username("validuser").is_ok());
        assert!(validate_username("user123").is_ok());
        assert!(validate_username("user-name").is_ok());
        assert!(validate_username("user_name").is_ok());

        // Invalid usernames
        assert!(validate_username("").is_err());
        assert!(validate_username("us").is_err()); // too short
        assert!(validate_username("this_is_a_very_long_username_that_exceeds_limit").is_err()); // too long
        assert!(validate_username("user@domain").is_err()); // invalid chars
        assert!(validate_username("user name").is_err()); // spaces
    }

    #[test]
    fn test_token_validation() {
        // Valid token format
        assert!(validate_token_format("eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJzdWIiOiIxMjM0NTY3ODkwIiwibmFtZSI6IkpvaG4gRG9lIiwiaWF0IjoxNTE2MjM5MDIyfQ.SflKxwRJSMeKKF2QT4fwpMeJf36POk6yJV_adQssw5c").is_ok());

        // Invalid tokens
        assert!(validate_token_format("").is_err());
        assert!(validate_token_format("invalid.token").is_err());
        assert!(validate_token_format("not-a-token").is_err());
        assert!(validate_token_format("a.b").is_err());
    }

    #[test]
    fn test_input_sanitization() {
        assert_eq!(sanitize_html_input("<script>alert('xss')</script>Hello"), "Hello");
        assert_eq!(sanitize_name_input("John O'Connor-Smith 123"), "John O'Connor-Smith");
        assert_eq!(sanitize_text_input("Hello <script>"), "Hello script");
    }
}