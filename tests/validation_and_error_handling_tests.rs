use chronos::app::{
    errors::AuthenticationError,
    validation::{
        validate_strong_password, validate_enhanced_email, validate_username, validate_name,
        validate_token_format, sanitize_html_input, sanitize_name_input, sanitize_text_input,
    },
};
use validator::{Validate, ValidationErrors};

#[cfg(test)]
mod validation_tests {
    use super::*;

    #[test]
    fn test_password_validation_success() {
        // Strong passwords should pass
        assert!(validate_strong_password("MyStr0ng!Pass").is_ok());
        assert!(validate_strong_password("C0mplex&Secure!2024").is_ok());
        assert!(validate_strong_password("ValidP@ssw0rd123").is_ok());
    }

    #[test]
    fn test_password_validation_failures() {
        // Too short
        assert!(validate_strong_password("short").is_err());
        assert!(validate_strong_password("Sh0rt!").is_err());

        // Common passwords
        assert!(validate_strong_password("password123").is_err());
        assert!(validate_strong_password("Password123").is_err());
        assert!(validate_strong_password("123456789").is_err());
        assert!(validate_strong_password("qwertyQWERTY123!").is_err());

        // Missing character types
        assert!(validate_strong_password("nocapitals123!").is_err()); // No uppercase
        assert!(validate_strong_password("NOLOWERCASE123!").is_err()); // No lowercase
        assert!(validate_strong_password("NoNumbers!@#").is_err()); // No numbers
        assert!(validate_strong_password("NoSpecialChars123").is_err()); // No special chars

        // Sequential/repeated patterns
        assert!(validate_strong_password("Abc1234!").is_err()); // Sequential
        assert!(validate_strong_password("Aaa1234!").is_err()); // Repeated

        // Too long
        let long_password = "A".repeat(130) + "1!";
        assert!(validate_strong_password(&long_password).is_err());
    }

    #[test]
    fn test_email_validation_success() {
        // Valid emails
        assert!(validate_enhanced_email("user@example.com").is_ok());
        assert!(validate_enhanced_email("test.email+tag@domain.co.uk").is_ok());
        assert!(validate_enhanced_email("user123@test-domain.com").is_ok());
        assert!(validate_enhanced_email("a@b.co").is_ok());
    }

    #[test]
    fn test_email_validation_failures() {
        // Empty email
        assert!(validate_enhanced_email("").is_err());

        // Invalid formats
        assert!(validate_enhanced_email("not-an-email").is_err());
        assert!(validate_enhanced_email("user@").is_err());
        assert!(validate_enhanced_email("@domain.com").is_err());
        assert!(validate_enhanced_email("user@domain").is_err());

        // Suspicious patterns
        assert!(validate_enhanced_email("user..name@domain.com").is_err());
        assert!(validate_enhanced_email(".user@domain.com").is_err());
        assert!(validate_enhanced_email("user.@domain.com").is_err());

        // Too long
        let long_email = "a".repeat(250) + "@example.com";
        assert!(validate_enhanced_email(&long_email).is_err());
    }

    #[test]
    fn test_username_validation_success() {
        // Valid usernames
        assert!(validate_username("validuser").is_ok());
        assert!(validate_username("user123").is_ok());
        assert!(validate_username("user-name").is_ok());
        assert!(validate_username("user_name").is_ok());
        assert!(validate_username("123user").is_ok());
    }

    #[test]
    fn test_username_validation_failures() {
        // Empty username
        assert!(validate_username("").is_err());

        // Too short
        assert!(validate_username("us").is_err());
        assert!(validate_username("u").is_err());

        // Too long
        let long_username = "a".repeat(31);
        assert!(validate_username(&long_username).is_err());

        // Invalid characters
        assert!(validate_username("user@domain").is_err());
        assert!(validate_username("user name").is_err());
        assert!(validate_username("user.name").is_err());
        assert!(validate_username("user!").is_err());
    }

    #[test]
    fn test_name_validation_success() {
        // Valid names
        assert!(validate_name("John Doe").is_ok());
        assert!(validate_name("Mary Jane").is_ok());
        assert!(validate_name("José María").is_ok());
        assert!(validate_name("O'Connor").is_ok());
        assert!(validate_name("Smith-Johnson").is_ok());
    }

    #[test]
    fn test_name_validation_failures() {
        // Empty name
        assert!(validate_name("").is_err());

        // Too long
        let long_name = "a".repeat(101);
        assert!(validate_name(&long_name).is_err());

        // Malicious content
        assert!(validate_name("<script>alert('xss')</script>").is_err());
        assert!(validate_name("John & Jane").is_err());
        assert!(validate_name("Name<tag>").is_err());
    }

    #[test]
    fn test_token_validation_success() {
        // Valid JWT-like tokens
        let valid_token = "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJzdWIiOiIxMjM0NTY3ODkwIiwibmFtZSI6IkpvaG4gRG9lIiwiaWF0IjoxNTE2MjM5MDIyfQ.SflKxwRJSMeKKF2QT4fwpMeJf36POk6yJV_adQssw5c";
        assert!(validate_token_format(valid_token).is_ok());
    }

    #[test]
    fn test_token_validation_failures() {
        // Empty token
        assert!(validate_token_format("").is_err());

        // Invalid format
        assert!(validate_token_format("invalid.token").is_err());
        assert!(validate_token_format("not-a-token").is_err());
        assert!(validate_token_format("a.b").is_err());
        assert!(validate_token_format("a.b.c.d").is_err());

        // Too short
        assert!(validate_token_format("a.b.c").is_err());

        // Too long
        let long_token = format!("{}.{}.{}", "a".repeat(2000), "b".repeat(2000), "c".repeat(2000));
        assert!(validate_token_format(&long_token).is_err());
    }
}

#[cfg(test)]
mod sanitization_tests {
    use super::*;

    #[test]
    fn test_html_sanitization() {
        // XSS attempts should be cleaned
        assert_eq!(sanitize_html_input("<script>alert('xss')</script>Hello"), "Hello");
        assert_eq!(sanitize_html_input("<img src=x onerror=alert(1)>"), "");
        assert_eq!(sanitize_html_input("Hello <b>World</b>"), "Hello <b>World</b>");

        // SQL injection attempts
        assert_eq!(sanitize_html_input("'; DROP TABLE users; --"), "'; DROP TABLE users; --");

        // Normal text should pass through
        assert_eq!(sanitize_html_input("Hello World"), "Hello World");
        assert_eq!(sanitize_html_input("user@example.com"), "user@example.com");
    }

    #[test]
    fn test_name_sanitization() {
        // Valid names should be preserved
        assert_eq!(sanitize_name_input("John Doe"), "John Doe");
        assert_eq!(sanitize_name_input("Mary-Jane O'Connor"), "Mary-Jane O'Connor");

        // Numbers and symbols should be filtered
        assert_eq!(sanitize_name_input("John123 Doe!"), "John Doe");
        assert_eq!(sanitize_name_input("Mary@Jane.com"), "Mary Jane");

        // Whitespace should be trimmed
        assert_eq!(sanitize_name_input("  John Doe  "), "John Doe");
    }

    #[test]
    fn test_text_sanitization() {
        // Allowed characters should pass
        assert_eq!(sanitize_text_input("Hello World 123!"), "Hello World 123!");
        assert_eq!(sanitize_text_input("user@example.com"), "user@example.com");
        assert_eq!(sanitize_text_input("Test_data-with.allowed?chars"), "Test_data-with.allowed?chars");

        // Disallowed characters should be filtered
        assert_eq!(sanitize_text_input("Hello<script>"), "Helloscript");
        assert_eq!(sanitize_text_input("Test#with%symbols"), "Testwithsymbols");
    }
}

#[cfg(test)]
mod error_handling_tests {
    use super::*;
    use axum::http::StatusCode;

    #[test]
    fn test_authentication_error_creation() {
        // Test different error types
        let validation_err = AuthenticationError::validation_failed(ValidationErrors::new());
        assert_eq!(validation_err.error_code(), "VALIDATION_FAILED");
        assert_eq!(validation_err.http_status(), StatusCode::BAD_REQUEST);

        let invalid_creds = AuthenticationError::invalid_credentials();
        assert_eq!(invalid_creds.error_code(), "INVALID_CREDENTIALS");
        assert_eq!(invalid_creds.http_status(), StatusCode::UNAUTHORIZED);

        let token_expired = AuthenticationError::token_expired();
        assert_eq!(token_expired.error_code(), "TOKEN_EXPIRED");
        assert_eq!(token_expired.http_status(), StatusCode::UNAUTHORIZED);

        let user_not_found = AuthenticationError::user_not_found();
        assert_eq!(user_not_found.error_code(), "USER_NOT_FOUND");
        assert_eq!(user_not_found.http_status(), StatusCode::NOT_FOUND);
    }

    #[test]
    fn test_rate_limit_error() {
        let rate_limit_err = AuthenticationError::rate_limit_exceeded(300);
        assert_eq!(rate_limit_err.error_code(), "RATE_LIMIT_EXCEEDED");
        assert_eq!(rate_limit_err.http_status(), StatusCode::TOO_MANY_REQUESTS);
        assert!(rate_limit_err.user_message().contains("300 seconds"));
    }

    #[test]
    fn test_user_already_exists_error() {
        let user_exists_err = AuthenticationError::user_already_exists("email");
        assert_eq!(user_exists_err.error_code(), "USER_ALREADY_EXISTS");
        assert_eq!(user_exists_err.http_status(), StatusCode::CONFLICT);
        assert!(user_exists_err.user_message().contains("email"));
    }

    #[test]
    fn test_weak_password_error() {
        let requirements = vec![
            "At least 8 characters".to_string(),
            "One uppercase letter".to_string(),
            "One special character".to_string(),
        ];
        let weak_password_err = AuthenticationError::weak_password(requirements.clone());
        assert_eq!(weak_password_err.error_code(), "WEAK_PASSWORD");
        assert_eq!(weak_password_err.http_status(), StatusCode::BAD_REQUEST);

        let field_errors = weak_password_err.field_errors();
        assert_eq!(field_errors.len(), 1);
        assert_eq!(field_errors[0].field, "password");
        assert!(field_errors[0].message.contains("At least 8 characters"));
    }

    #[test]
    fn test_error_logging_flags() {
        // High-severity errors should be logged
        assert!(AuthenticationError::database_error("connection failed").should_log());
        assert!(AuthenticationError::internal_error("unexpected error").should_log());
        assert!(AuthenticationError::rate_limit_exceeded(60).should_log());

        // Low-severity errors should not be logged
        assert!(!AuthenticationError::invalid_credentials().should_log());
        assert!(!AuthenticationError::user_not_found().should_log());
        assert!(!AuthenticationError::token_expired().should_log());
    }

    #[test]
    fn test_error_response_format() {
        let error = AuthenticationError::invalid_input(
            "email".to_string(),
            "Invalid email format".to_string()
        );

        let field_errors = error.field_errors();
        assert_eq!(field_errors.len(), 1);
        assert_eq!(field_errors[0].field, "email");
        assert_eq!(field_errors[0].message, "Invalid email format");
    }

    #[test]
    fn test_legacy_compatibility() {
        // Test that legacy methods work
        let error1 = AuthenticationError::new("Test error message");
        assert_eq!(error1.error_code(), "INTERNAL_ERROR");

        let validation_errors = ValidationErrors::new();
        let error2 = AuthenticationError::validation_error(&validation_errors);
        assert_eq!(error2.error_code(), "VALIDATION_FAILED");
    }
}

#[cfg(test)]
mod integration_tests {
    use super::*;
    use validator::Validate;
    use chronos::app::models::auth::RegisterRequest;

    #[test]
    fn test_register_request_validation_and_sanitization() {
        // Test valid request
        let mut valid_request = RegisterRequest {
            name: Some("John Doe".to_string()),
            email: "john@example.com".to_string(),
            password: "MyStr0ng!Pass".to_string(),
        };

        valid_request.sanitize();
        assert!(valid_request.validate().is_ok());
        assert_eq!(valid_request.email, "john@example.com");
        assert_eq!(valid_request.name, Some("John Doe".to_string()));

        // Test sanitization
        let mut malicious_request = RegisterRequest {
            name: Some("<script>alert('xss')</script>John123".to_string()),
            email: "  JOHN@EXAMPLE.COM  ".to_string(),
            password: "MyStr0ng!Pass".to_string(),
        };

        malicious_request.sanitize();
        assert_eq!(malicious_request.email, "john@example.com"); // Trimmed and lowercased
        assert_eq!(malicious_request.name, Some("John".to_string())); // XSS and numbers filtered
    }

    #[test]
    fn test_register_request_validation_failures() {
        // Test weak password
        let weak_password_request = RegisterRequest {
            name: Some("John Doe".to_string()),
            email: "john@example.com".to_string(),
            password: "weak".to_string(),
        };

        let validation_result = weak_password_request.validate();
        assert!(validation_result.is_err());

        if let Err(errors) = validation_result {
            assert!(errors.field_errors().contains_key("password"));
        }

        // Test invalid email
        let invalid_email_request = RegisterRequest {
            name: Some("John Doe".to_string()),
            email: "not-an-email".to_string(),
            password: "MyStr0ng!Pass".to_string(),
        };

        let validation_result = invalid_email_request.validate();
        assert!(validation_result.is_err());

        if let Err(errors) = validation_result {
            assert!(errors.field_errors().contains_key("email"));
        }
    }

    #[test]
    fn test_comprehensive_password_security() {
        // Test various attack vectors
        let weak_passwords = vec![
            "password",         // Common password
            "12345678",        // All numbers
            "abcdefgh",        // All lowercase
            "ABCDEFGH",        // All uppercase
            "Password",        // Missing numbers and special chars
            "password123",     // Common with numbers
            "Password123",     // Missing special chars
            "Passw0rd!",      // Contains "password"
            "123456789",      // Sequential numbers
            "abcdefg!1",      // Sequential letters
            "aaaaaa!1A",      // Repeated characters
        ];

        for password in weak_passwords {
            assert!(validate_strong_password(password).is_err(),
                "Password '{}' should be rejected", password);
        }

        // Test strong passwords
        let strong_passwords = vec![
            "MyStr0ng!Pass",
            "C0mplex&Secure!2024",
            "ValidP@ssw0rd123",
            "D1fferent$Password",
            "Un1que#Passw0rd!",
        ];

        for password in strong_passwords {
            assert!(validate_strong_password(password).is_ok(),
                "Password '{}' should be accepted", password);
        }
    }
}

// Performance tests (optional, for large-scale validation)
#[cfg(test)]
mod performance_tests {
    use super::*;
    use std::time::Instant;

    #[test]
    fn test_validation_performance() {
        let test_data = vec![
            ("user@example.com", "MyStr0ng!Pass"),
            ("test@domain.co.uk", "C0mplex&Secure!"),
            ("admin@company.org", "ValidP@ssw0rd123"),
        ];

        let start = Instant::now();

        for (email, password) in test_data.iter().cycle().take(1000) {
            assert!(validate_enhanced_email(email).is_ok());
            assert!(validate_strong_password(password).is_ok());
        }

        let duration = start.elapsed();
        println!("1000 validations took: {:?}", duration);

        // Validation should be fast (less than 1 second for 1000 validations)
        assert!(duration.as_secs() < 1, "Validation performance is too slow");
    }

    #[test]
    fn test_sanitization_performance() {
        let test_inputs = vec![
            "<script>alert('xss')</script>Hello World",
            "Normal text with @email.com",
            "Text with 123 numbers and symbols!@#",
        ];

        let start = Instant::now();

        for input in test_inputs.iter().cycle().take(1000) {
            let _sanitized = sanitize_html_input(input);
        }

        let duration = start.elapsed();
        println!("1000 sanitizations took: {:?}", duration);

        // Sanitization should be fast
        assert!(duration.as_secs() < 1, "Sanitization performance is too slow");
    }
}