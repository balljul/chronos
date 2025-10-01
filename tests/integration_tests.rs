use chronos::app::models::auth::{RegisterRequest, RegisterResponse, AuthError};
use serde_json;
use validator::Validate;

#[cfg(test)]
mod integration_tests {
    use super::*;

    // Note: These tests would require a test database setup
    // For now, we'll focus on testing the request/response structure and validation

    #[tokio::test]
    async fn test_register_endpoint_request_structure() {
        let valid_request = RegisterRequest {
            name: Some("John Doe".to_string()),
            email: "john.doe@example.com".to_string(),
            password: "SecurePass1!".to_string(),
        };

        // Test serialization
        let json = serde_json::to_string(&valid_request).unwrap();
        assert!(json.contains("john.doe@example.com"));
        assert!(json.contains("SecurePass1!"));

        // Test deserialization
        let request_json = r#"{
            "name": "John Doe",
            "email": "john.doe@example.com",
            "password": "SecurePass1!"
        }"#;

        let parsed_request: RegisterRequest = serde_json::from_str(request_json).unwrap();
        assert_eq!(parsed_request.email, "john.doe@example.com");
        assert_eq!(parsed_request.password, "SecurePass1!");
    }

    #[tokio::test]
    async fn test_register_response_structure() {
        let response = RegisterResponse {
            message: "User registered successfully".to_string(),
            user: chronos::app::models::user::UserResponse {
                id: uuid::Uuid::new_v4(),
                name: Some("John Doe".to_string()),
                email: "john.doe@example.com".to_string(),
                created_at: Some(time::OffsetDateTime::now_utc()),
            },
        };

        // Test serialization
        let json = serde_json::to_string(&response).unwrap();
        assert!(json.contains("User registered successfully"));
        assert!(json.contains("john.doe@example.com"));
        assert!(!json.contains("password")); // Should not contain password
    }

    #[tokio::test]
    async fn test_auth_error_structure() {
        let error = AuthError::new("Email already registered");
        let json = serde_json::to_string(&error).unwrap();
        assert!(json.contains("Email already registered"));

        let error_with_details = AuthError::with_details(
            "Validation failed",
            vec!["email: Invalid email format".to_string()],
        );
        let json = serde_json::to_string(&error_with_details).unwrap();
        assert!(json.contains("Validation failed"));
        assert!(json.contains("Invalid email format"));
    }

    #[test]
    fn test_validation_scenarios() {
        // Test case 1: Invalid email format
        let invalid_email = RegisterRequest {
            name: Some("John Doe".to_string()),
            email: "invalid-email".to_string(),
            password: "SecurePass1!".to_string(),
        };
        assert!(invalid_email.validate().is_err());

        // Test case 2: Weak password
        let weak_password = RegisterRequest {
            name: Some("John Doe".to_string()),
            email: "john.doe@example.com".to_string(),
            password: "weak".to_string(),
        };
        assert!(weak_password.validate().is_err());

        // Test case 3: Valid request
        let valid_request = RegisterRequest {
            name: Some("John Doe".to_string()),
            email: "john.doe@example.com".to_string(),
            password: "SecurePass1!".to_string(),
        };
        assert!(valid_request.validate().is_ok());

        // Test case 4: No name provided (should be valid)
        let no_name = RegisterRequest {
            name: None,
            email: "john.doe@example.com".to_string(),
            password: "SecurePass1!".to_string(),
        };
        assert!(no_name.validate().is_ok());
    }

    #[test]
    fn test_password_strength_requirements() {
        let test_cases = vec![
            ("Short1!", false, "too short"),
            ("nouppercase1!", false, "no uppercase"),
            ("NOLOWERCASE1!", false, "no lowercase"),
            ("NoNumbers!", false, "no numbers"),
            ("NoSpecialChars1", false, "no special characters"),
            ("SecurePass1!", true, "valid password"),
            ("Complex123!@#", true, "valid complex password"),
            ("MyP@ssw0rd", true, "valid with special chars"),
        ];

        for (password, should_be_valid, description) in test_cases {
            let request = RegisterRequest {
                name: Some("Test User".to_string()),
                email: "test@example.com".to_string(),
                password: password.to_string(),
            };

            let is_valid = request.validate().is_ok();
            assert_eq!(
                is_valid, should_be_valid,
                "Password '{}' validation failed: {}",
                password, description
            );
        }
    }

    #[test]
    fn test_error_response_scenarios() {
        // Test validation error conversion
        let request = RegisterRequest {
            name: Some("John Doe".to_string()),
            email: "invalid-email".to_string(),
            password: "weak".to_string(),
        };

        if let Err(validation_errors) = request.validate() {
            let auth_error = AuthError::validation_error(&validation_errors);
            assert_eq!(auth_error.error, "Validation failed");
            assert!(auth_error.details.is_some());

            let details = auth_error.details.unwrap();
            assert!(details.iter().any(|detail| detail.contains("email")));
            assert!(details.iter().any(|detail| detail.contains("password")));
        }
    }
}