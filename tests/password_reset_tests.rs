use chronos::app::models::auth::{ForgotPasswordRequest, ResetPasswordRequest, AuthError};
use chronos::app::models::password_reset::PasswordResetToken;
use chronos::app::models::user::User;
use chronos::app::services::email_service::MockEmailService;
use serde_json;
use validator::Validate;
use uuid::Uuid;
use time::OffsetDateTime;

#[cfg(test)]
mod password_reset_tests {
    use super::*;

    #[tokio::test]
    async fn test_password_reset_token_generation() {
        let user_id = Uuid::new_v4();
        let plain_token = PasswordResetToken::generate_secure_token();

        // Test token length and character set
        assert_eq!(plain_token.len(), 64);
        assert!(plain_token.chars().all(|c| c.is_alphanumeric()));

        // Test secure token creation
        let reset_token = PasswordResetToken::new(user_id, &plain_token).unwrap();
        assert_eq!(reset_token.user_id, user_id);
        assert!(!reset_token.used);
        assert!(!reset_token.is_expired());
        assert!(reset_token.is_valid());

        // Test token verification
        assert!(reset_token.verify_token(&plain_token).unwrap());
        assert!(!reset_token.verify_token("wrong_token").unwrap());
    }

    #[test]
    fn test_password_reset_token_expiration() {
        let user_id = Uuid::new_v4();
        let plain_token = PasswordResetToken::generate_secure_token();

        let mut reset_token = PasswordResetToken::new(user_id, &plain_token).unwrap();

        // Test valid token
        assert!(reset_token.is_valid());

        // Test expired token
        reset_token.expires_at = OffsetDateTime::now_utc() - time::Duration::hours(1);
        assert!(reset_token.is_expired());
        assert!(!reset_token.is_valid());

        // Test used token
        reset_token.expires_at = OffsetDateTime::now_utc() + time::Duration::hours(1);
        reset_token.used = true;
        assert!(!reset_token.is_valid());
    }

    #[test]
    fn test_forgot_password_request_validation() {
        // Valid request
        let valid_request = ForgotPasswordRequest {
            email: "test@example.com".to_string(),
        };
        assert!(valid_request.validate().is_ok());

        // Invalid email format
        let invalid_request = ForgotPasswordRequest {
            email: "invalid-email".to_string(),
        };
        assert!(invalid_request.validate().is_err());

        // Empty email
        let empty_request = ForgotPasswordRequest {
            email: "".to_string(),
        };
        assert!(empty_request.validate().is_err());
    }

    #[test]
    fn test_reset_password_request_validation() {
        // Valid request
        let valid_request = ResetPasswordRequest {
            token: "valid_token_123".to_string(),
            password: "NewSecurePass1!".to_string(),
        };
        assert!(valid_request.validate().is_ok());

        // Invalid password - too short
        let short_password = ResetPasswordRequest {
            token: "valid_token_123".to_string(),
            password: "weak".to_string(),
        };
        assert!(short_password.validate().is_err());

        // Invalid password - no uppercase
        let no_uppercase = ResetPasswordRequest {
            token: "valid_token_123".to_string(),
            password: "newsecurepass1!".to_string(),
        };
        assert!(no_uppercase.validate().is_err());

        // Invalid password - no lowercase
        let no_lowercase = ResetPasswordRequest {
            token: "valid_token_123".to_string(),
            password: "NEWSECUREPASS1!".to_string(),
        };
        assert!(no_lowercase.validate().is_err());

        // Invalid password - no number
        let no_number = ResetPasswordRequest {
            token: "valid_token_123".to_string(),
            password: "NewSecurePass!".to_string(),
        };
        assert!(no_number.validate().is_err());

        // Invalid password - no special character
        let no_special = ResetPasswordRequest {
            token: "valid_token_123".to_string(),
            password: "NewSecurePass1".to_string(),
        };
        assert!(no_special.validate().is_err());
    }

    #[test]
    fn test_password_strength_requirements() {
        let test_cases = vec![
            ("Short1!", false, "too short"),
            ("nouppercase1!", false, "no uppercase"),
            ("NOLOWERCASE1!", false, "no lowercase"),
            ("NoNumbers!", false, "no numbers"),
            ("NoSpecialChars1", false, "no special characters"),
            ("NewSecurePass1!", true, "valid password"),
            ("MyP@ssw0rd123", true, "valid with special chars"),
            ("ComplexP@ss1", true, "valid complex password"),
        ];

        for (password, should_be_valid, description) in test_cases {
            let request = ResetPasswordRequest {
                token: "test_token".to_string(),
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

    #[tokio::test]
    async fn test_mock_email_service() {
        let email_service = MockEmailService::new();
        let test_email = "test@example.com";
        let test_token = "secure_token_123";

        // Initially no emails sent
        assert_eq!(email_service.count_sent_emails(), 0);

        // Send password reset email
        let result = email_service.send_password_reset_email(test_email, test_token).await;
        assert!(result.is_ok());

        // Check email was recorded
        assert_eq!(email_service.count_sent_emails(), 1);

        let last_email = email_service.get_last_sent_email().unwrap();
        assert_eq!(last_email.to, test_email);
        assert!(last_email.subject.contains("Password Reset"));
        assert!(last_email.body.contains(test_token));
        assert!(last_email.body.contains("Chronos"));

        // Send another email
        email_service.send_password_reset_email("another@example.com", "another_token").await.unwrap();
        assert_eq!(email_service.count_sent_emails(), 2);

        // Clear emails
        email_service.clear_sent_emails();
        assert_eq!(email_service.count_sent_emails(), 0);
    }

    #[test]
    fn test_auth_error_creation() {
        // Test simple error
        let simple_error = AuthError::new("Test error message");
        assert_eq!(simple_error.error, "Test error message");
        assert!(simple_error.details.is_none());

        // Test error with details
        let details = vec!["Detail 1".to_string(), "Detail 2".to_string()];
        let detailed_error = AuthError::with_details("Validation failed", details.clone());
        assert_eq!(detailed_error.error, "Validation failed");
        assert_eq!(detailed_error.details, Some(details));

        // Test serialization
        let json = serde_json::to_string(&simple_error).unwrap();
        assert!(json.contains("Test error message"));

        let json = serde_json::to_string(&detailed_error).unwrap();
        assert!(json.contains("Validation failed"));
        assert!(json.contains("Detail 1"));
        assert!(json.contains("Detail 2"));
    }

    #[test]
    fn test_token_uniqueness() {
        // Generate multiple tokens and ensure they're unique
        let mut tokens = std::collections::HashSet::new();

        for _ in 0..1000 {
            let token = PasswordResetToken::generate_secure_token();
            assert!(!tokens.contains(&token), "Duplicate token generated: {}", token);
            tokens.insert(token);
        }

        assert_eq!(tokens.len(), 1000);
    }

    #[test]
    fn test_token_character_distribution() {
        let token = PasswordResetToken::generate_secure_token();

        let has_uppercase = token.chars().any(|c| c.is_ascii_uppercase());
        let has_lowercase = token.chars().any(|c| c.is_ascii_lowercase());
        let has_digit = token.chars().any(|c| c.is_ascii_digit());

        // In a 64-character token, we should have good distribution
        // This is probabilistic, but very likely to pass
        assert!(has_uppercase, "Token should contain uppercase letters");
        assert!(has_lowercase, "Token should contain lowercase letters");
        assert!(has_digit, "Token should contain digits");
    }

    #[test]
    fn test_forgot_password_response_structure() {
        use chronos::app::models::auth::ForgotPasswordResponse;

        let response = ForgotPasswordResponse {
            message: "If your email is registered, you will receive a password reset link shortly.".to_string(),
        };

        // Test serialization
        let json = serde_json::to_string(&response).unwrap();
        assert!(json.contains("reset link"));
        assert!(!json.contains("\"password\":")); // Should not contain password field
    }

    #[test]
    fn test_reset_password_response_structure() {
        use chronos::app::models::auth::ResetPasswordResponse;

        let response = ResetPasswordResponse {
            message: "Password reset successfully".to_string(),
        };

        // Test serialization
        let json = serde_json::to_string(&response).unwrap();
        assert!(json.contains("successfully"));
        assert!(!json.contains("token")); // Should not contain token info
    }

    #[test]
    fn test_user_password_hashing_compatibility() {
        let password = "TestPassword123!";
        let user = User::new(
            Some("Test User".to_string()),
            "test@example.com".to_string(),
            password,
        ).unwrap();

        // Test password verification
        assert!(user.verify_password(password).unwrap());
        assert!(!user.verify_password("wrong_password").unwrap());

        // Test that hashed password is different from plain password
        assert_ne!(user.password_hash, password);
        assert!(user.password_hash.len() > 50); // Argon2 hashes are long
    }
}