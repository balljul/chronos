use chronos::app::models::auth::{RegisterRequest, validate_password};
use chronos::app::models::user::User;
use validator::Validate;

#[cfg(test)]
mod auth_tests {
    use super::*;

    #[test]
    fn test_password_validation_valid() {
        let valid_passwords = vec![
            "MySecure1!",
            "Password123!@#",
            "Complex1$",
            "Str0ng!Pass",
        ];

        for password in valid_passwords {
            assert!(validate_password(password).is_ok(), "Password '{}' should be valid", password);
        }
    }

    #[test]
    fn test_password_validation_invalid() {
        let invalid_passwords = vec![
            ("short1!", "too short"),
            ("nouppercase1!", "no uppercase"),
            ("NOLOWERCASE1!", "no lowercase"),
            ("NoNumbers!", "no numbers"),
            ("NoSpecialChars1", "no special characters"),
            ("", "empty password"),
        ];

        for (password, reason) in invalid_passwords {
            assert!(validate_password(password).is_err(), "Password '{}' should be invalid ({})", password, reason);
        }
    }

    #[test]
    fn test_register_request_validation_valid() {
        let valid_request = RegisterRequest {
            name: Some("John Doe".to_string()),
            email: "john.doe@example.com".to_string(),
            password: "SecurePass1!".to_string(),
        };

        assert!(valid_request.validate().is_ok());
    }

    #[test]
    fn test_register_request_validation_invalid_email() {
        let invalid_email_request = RegisterRequest {
            name: Some("John Doe".to_string()),
            email: "invalid-email".to_string(),
            password: "SecurePass1!".to_string(),
        };

        assert!(invalid_email_request.validate().is_err());
    }

    #[test]
    fn test_register_request_validation_invalid_password() {
        let invalid_password_request = RegisterRequest {
            name: Some("John Doe".to_string()),
            email: "john.doe@example.com".to_string(),
            password: "weak".to_string(),
        };

        assert!(invalid_password_request.validate().is_err());
    }

    #[test]
    fn test_user_creation_with_valid_data() {
        let user = User::new(
            Some("John Doe".to_string()),
            "john.doe@example.com".to_string(),
            "SecurePass1!",
        );

        assert!(user.is_ok());

        let user = user.unwrap();
        assert_eq!(user.name, Some("John Doe".to_string()));
        assert_eq!(user.email, "john.doe@example.com");
        assert_ne!(user.password_hash, "SecurePass1!"); // Should be hashed
        assert!(user.created_at.is_some());
        assert!(user.updated_at.is_some());
    }

    #[test]
    fn test_password_hashing_and_verification() {
        let password = "SecurePass1!";
        let user = User::new(
            Some("John Doe".to_string()),
            "john.doe@example.com".to_string(),
            password,
        ).unwrap();

        // Password should be hashed
        assert_ne!(user.password_hash, password);

        // Should be able to verify correct password
        assert!(user.verify_password(password).unwrap());

        // Should not verify incorrect password
        assert!(!user.verify_password("WrongPassword1!").unwrap());
    }

    #[test]
    fn test_user_response_excludes_password() {
        let user = User::new(
            Some("John Doe".to_string()),
            "john.doe@example.com".to_string(),
            "SecurePass1!",
        ).unwrap();

        let response = user.to_response();

        assert_eq!(response.name, Some("John Doe".to_string()));
        assert_eq!(response.email, "john.doe@example.com");
        assert_eq!(response.id, user.id);
        assert_eq!(response.created_at, user.created_at);
        // Password should not be included in response
    }

    #[test]
    fn test_email_validation_with_validator() {
        let user = User {
            id: uuid::Uuid::new_v4(),
            name: Some("John Doe".to_string()),
            email: "invalid-email".to_string(),
            password_hash: "hash".to_string(),
            created_at: Some(time::OffsetDateTime::now_utc()),
            updated_at: Some(time::OffsetDateTime::now_utc()),
        };

        assert!(user.validate().is_err());

        let user_valid = User {
            id: uuid::Uuid::new_v4(),
            name: Some("John Doe".to_string()),
            email: "john.doe@example.com".to_string(),
            password_hash: "hash".to_string(),
            created_at: Some(time::OffsetDateTime::now_utc()),
            updated_at: Some(time::OffsetDateTime::now_utc()),
        };

        assert!(user_valid.validate().is_ok());
    }
}