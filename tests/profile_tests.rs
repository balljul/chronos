use chronos::app::models::auth::{ChangePasswordRequest, ProfileUpdateRequest};
use chronos::app::models::user::User;
use time::OffsetDateTime;
use uuid::Uuid;
use validator::Validate;

#[cfg(test)]
mod profile_tests {
    use super::*;

    // Helper function to create a test user
    fn create_test_user() -> User {
        User {
            id: Uuid::new_v4(),
            name: Some("Test User".to_string()),
            email: "test@example.com".to_string(),
            password_hash: "$argon2id$v=19$m=19456,t=2,p=1$test$hash".to_string(),
            created_at: Some(OffsetDateTime::now_utc()),
            updated_at: Some(OffsetDateTime::now_utc()),
        }
    }

    #[test]
    fn test_profile_update_request_validation_valid() {
        let valid_requests = vec![
            ProfileUpdateRequest {
                name: Some("John Doe".to_string()),
                email: Some("john.doe@example.com".to_string()),
                current_password: Some("OldPassword1!".to_string()),
            },
            ProfileUpdateRequest {
                name: Some("Jane Smith".to_string()),
                email: None,
                current_password: None,
            },
            ProfileUpdateRequest {
                name: None,
                email: Some("jane.smith@example.com".to_string()),
                current_password: Some("CurrentPass1!".to_string()),
            },
            ProfileUpdateRequest {
                name: None,
                email: None,
                current_password: None,
            },
        ];

        for request in valid_requests {
            assert!(
                request.validate().is_ok(),
                "Valid profile update request should pass validation"
            );
        }
    }

    #[test]
    fn test_profile_update_request_validation_invalid_email() {
        let invalid_requests = vec![
            ProfileUpdateRequest {
                name: Some("John Doe".to_string()),
                email: Some("invalid-email".to_string()),
                current_password: Some("Password1!".to_string()),
            },
            ProfileUpdateRequest {
                name: Some("Jane Smith".to_string()),
                email: Some("@example.com".to_string()),
                current_password: Some("Password1!".to_string()),
            },
            ProfileUpdateRequest {
                name: Some("Bob Johnson".to_string()),
                email: Some("user@".to_string()),
                current_password: Some("Password1!".to_string()),
            },
        ];

        for request in invalid_requests {
            assert!(
                request.validate().is_err(),
                "Invalid email should fail validation"
            );
        }
    }

    #[test]
    fn test_change_password_request_validation_valid() {
        let valid_requests = vec![
            ChangePasswordRequest {
                current_password: "OldPassword1!".to_string(),
                new_password: "NewPassword1!".to_string(),
            },
            ChangePasswordRequest {
                current_password: "Current123!".to_string(),
                new_password: "SuperSecure456@".to_string(),
            },
        ];

        for request in valid_requests {
            assert!(
                request.validate().is_ok(),
                "Valid change password request should pass validation"
            );
        }
    }

    #[test]
    fn test_change_password_request_validation_invalid() {
        let invalid_requests = vec![
            // New password too short
            ChangePasswordRequest {
                current_password: "OldPassword1!".to_string(),
                new_password: "short1!".to_string(),
            },
            // New password missing uppercase
            ChangePasswordRequest {
                current_password: "OldPassword1!".to_string(),
                new_password: "newpassword1!".to_string(),
            },
            // New password missing lowercase
            ChangePasswordRequest {
                current_password: "OldPassword1!".to_string(),
                new_password: "NEWPASSWORD1!".to_string(),
            },
            // New password missing numbers
            ChangePasswordRequest {
                current_password: "OldPassword1!".to_string(),
                new_password: "NewPassword!".to_string(),
            },
            // New password missing special characters
            ChangePasswordRequest {
                current_password: "OldPassword1!".to_string(),
                new_password: "NewPassword1".to_string(),
            },
        ];

        for request in invalid_requests {
            assert!(
                request.validate().is_err(),
                "Invalid new password should fail validation"
            );
        }
    }

    #[test]
    fn test_user_password_verification() {
        let password = "TestPassword123!";
        let user = User::new(
            Some("Test User".to_string()),
            "test@example.com".to_string(),
            password,
        )
        .unwrap();

        // Test correct password
        assert!(
            user.verify_password(password).unwrap(),
            "Correct password should verify"
        );

        // Test incorrect passwords
        let incorrect_passwords = vec![
            "WrongPassword123!",
            "testpassword123!",
            "TestPassword123",
            "TestPassword124!",
            "",
        ];

        for wrong_password in incorrect_passwords {
            assert!(
                !user.verify_password(wrong_password).unwrap(),
                "Incorrect password '{}' should not verify",
                wrong_password
            );
        }
    }

    #[test]
    fn test_user_to_response() {
        let user = create_test_user();
        let response = user.to_response();

        assert_eq!(response.id, user.id);
        assert_eq!(response.name, user.name);
        assert_eq!(response.email, user.email);
        assert_eq!(response.created_at, user.created_at);

        // Ensure sensitive data is not included
        // The to_response method should not include password_hash or other sensitive fields
        // This is implicitly tested by the UserResponse struct not having a password_hash field
    }

    #[test]
    fn test_user_creation_with_password_hashing() {
        let name = Some("Test User".to_string());
        let email = "test@example.com".to_string();
        let password = "TestPassword123!";

        let user = User::new(name.clone(), email.clone(), password).unwrap();

        assert_eq!(user.name, name);
        assert_eq!(user.email, email);
        assert_ne!(user.password_hash, password, "Password should be hashed");
        assert!(
            user.password_hash.starts_with("$argon2"),
            "Password hash should use Argon2"
        );
        assert!(
            user.verify_password(password).unwrap(),
            "Original password should verify against hash"
        );
    }

    #[test]
    fn test_password_hash_function() {
        let password = "TestPassword123!";

        // Test that hashing works
        let hash = User::hash_password(password).unwrap();
        assert_ne!(
            hash, password,
            "Hash should be different from original password"
        );
        assert!(hash.starts_with("$argon2"), "Hash should use Argon2 format");

        // Test that same password produces different hashes (due to random salt)
        let hash2 = User::hash_password(password).unwrap();
        assert_ne!(
            hash, hash2,
            "Same password should produce different hashes due to random salt"
        );
    }

    #[test]
    fn test_profile_update_email_change_logic() {
        // This test verifies the logic for determining when email changes require password verification
        let current_email = "current@example.com".to_string();

        // Test cases where email change is detected
        let email_changing_requests = vec![
            ProfileUpdateRequest {
                name: None,
                email: Some("new@example.com".to_string()),
                current_password: Some("Password1!".to_string()),
            },
            ProfileUpdateRequest {
                name: Some("Test User".to_string()),
                email: Some("different@example.com".to_string()),
                current_password: Some("Password1!".to_string()),
            },
        ];

        for request in email_changing_requests {
            let changing_email =
                request.email.is_some() && request.email != Some(current_email.clone());
            assert!(
                changing_email,
                "Should detect email change when new email is different"
            );
        }

        // Test cases where email is not changing
        let non_changing_requests = vec![
            ProfileUpdateRequest {
                name: Some("Test User".to_string()),
                email: None,
                current_password: None,
            },
            ProfileUpdateRequest {
                name: Some("Test User".to_string()),
                email: Some(current_email.clone()),
                current_password: None,
            },
        ];

        for request in non_changing_requests {
            let changing_email =
                request.email.is_some() && request.email != Some(current_email.clone());
            assert!(
                !changing_email,
                "Should not detect email change when email is same or None"
            );
        }
    }
}
