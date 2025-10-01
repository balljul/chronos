use crate::app::models::auth::{
    AuthError, ForgotPasswordRequest, ForgotPasswordResponse, RegisterRequest, RegisterResponse,
    ResetPasswordRequest, ResetPasswordResponse,
};
use crate::app::models::password_reset::PasswordResetToken;
use crate::app::models::user::User;
use crate::app::repositories::password_reset_repository::PasswordResetRepository;
use crate::app::repositories::user_repository::UserRepository;
use crate::app::services::email_service::MockEmailService;
use time::OffsetDateTime;
use validator::Validate;

#[derive(Clone)]
pub struct AuthService {
    user_repository: UserRepository,
    password_reset_repository: PasswordResetRepository,
    email_service: MockEmailService,
}

impl AuthService {
    pub fn new(
        user_repository: UserRepository,
        password_reset_repository: PasswordResetRepository,
        email_service: MockEmailService,
    ) -> Self {
        Self {
            user_repository,
            password_reset_repository,
            email_service,
        }
    }

    pub async fn find_user_by_email(&self, email: &str) -> Result<Option<User>, sqlx::Error> {
        self.user_repository.find_by_email(email).await
    }

    pub async fn register(&self, request: RegisterRequest) -> Result<RegisterResponse, AuthError> {
        // Validate request
        if let Err(validation_errors) = request.validate() {
            return Err(AuthError::validation_error(&validation_errors));
        }

        // Check if email already exists
        match self.user_repository.find_by_email(&request.email).await {
            Ok(Some(_)) => {
                return Err(AuthError::new("Email already registered"));
            }
            Ok(None) => {
                // Email is available, continue
            }
            Err(e) => {
                return Err(AuthError::new(&format!("Database error: {}", e)));
            }
        }

        // Create user
        let user = match User::new(request.name, request.email, &request.password) {
            Ok(user) => user,
            Err(e) => {
                return Err(AuthError::new(&format!("Password hashing error: {}", e)));
            }
        };

        // Save user to database
        match self.user_repository.create(&user).await {
            Ok(created_user) => Ok(RegisterResponse {
                message: "User registered successfully".to_string(),
                user: created_user.to_response(),
            }),
            Err(e) => {
                // Check if it's a unique constraint violation (email already exists)
                if e.to_string().contains("duplicate key")
                    || e.to_string().contains("UNIQUE constraint")
                {
                    Err(AuthError::new("Email already registered"))
                } else {
                    Err(AuthError::new(&format!("Failed to create user: {}", e)))
                }
            }
        }
    }

    pub async fn forgot_password(
        &self,
        request: ForgotPasswordRequest,
    ) -> Result<ForgotPasswordResponse, AuthError> {
        // Validate request
        if let Err(validation_errors) = request.validate() {
            return Err(AuthError::validation_error(&validation_errors));
        }

        // Check if user exists (but don't reveal if email doesn't exist for security)
        let user = match self.user_repository.find_by_email(&request.email).await {
            Ok(user_opt) => user_opt,
            Err(e) => {
                return Err(AuthError::new(&format!("Database error: {}", e)));
            }
        };

        if let Some(user) = user {
            // Check rate limiting - max 3 requests per hour
            let one_hour_ago = OffsetDateTime::now_utc() - time::Duration::hours(1);
            match self
                .password_reset_repository
                .count_recent_requests(user.id, one_hour_ago)
                .await
            {
                Ok(count) if count >= 3 => {
                    return Err(AuthError::new(
                        "Too many password reset requests. Please try again later.",
                    ));
                }
                Ok(_) => {
                    // Rate limit not exceeded, continue
                }
                Err(e) => {
                    return Err(AuthError::new(&format!("Database error: {}", e)));
                }
            }

            // Generate secure token
            let plain_token = PasswordResetToken::generate_secure_token();
            let reset_token = match PasswordResetToken::new(user.id, &plain_token) {
                Ok(token) => token,
                Err(e) => {
                    return Err(AuthError::new(&format!("Token generation error: {}", e)));
                }
            };

            // Save token to database
            match self.password_reset_repository.create(&reset_token).await {
                Ok(_) => {
                    // Send email with token
                    if let Err(e) = self
                        .email_service
                        .send_password_reset_email(&request.email, &plain_token)
                        .await
                    {
                        return Err(AuthError::new(&format!("Email sending failed: {}", e)));
                    }
                }
                Err(e) => {
                    return Err(AuthError::new(&format!(
                        "Failed to create reset token: {}",
                        e
                    )));
                }
            }
        }

        // Always return success message regardless of whether email exists (security best practice)
        Ok(ForgotPasswordResponse {
            message: "If your email is registered, you will receive a password reset link shortly."
                .to_string(),
        })
    }

    pub async fn reset_password(
        &self,
        request: ResetPasswordRequest,
    ) -> Result<ResetPasswordResponse, AuthError> {
        // Validate request
        if let Err(validation_errors) = request.validate() {
            return Err(AuthError::validation_error(&validation_errors));
        }

        // Find all valid tokens and check if any match
        let mut matching_token: Option<PasswordResetToken> = None;
        let mut user_id: Option<uuid::Uuid> = None;

        // We need to check all potential tokens since we can't search by plain token
        // This is a simplified approach - in production, you might want to optimize this
        // by using a more efficient token storage strategy

        // For now, we'll iterate through recent tokens to find a match
        // In production, consider adding a token index or using a different approach

        // Since we can't easily find tokens by plain text, we'll check user validation differently
        // Let's first try to extract any valid tokens from recent requests

        // Instead, let's use a different approach - check all recent non-expired, non-used tokens
        // and verify against them

        // For this implementation, we'll need to get users and check their tokens
        // This is not the most efficient, but works for the demonstration

        // Alternative: We could store a mapping or use a different token structure
        // For now, let's implement a working solution by checking all recent tokens

        // We'll need to modify our approach. Let's find tokens by trying to verify against recent ones.
        // Since this is complex, let's implement a simpler approach by finding all recent tokens
        // and checking them one by one.

        // Get all users (not efficient, but works for demo - in production, optimize this)
        let users = match self.user_repository.get_all().await {
            Ok(users) => users,
            Err(e) => {
                return Err(AuthError::new(&format!("Database error: {}", e)));
            }
        };

        for user in users {
            if let Ok(tokens) = self
                .password_reset_repository
                .find_valid_tokens_by_user_id(user.id)
                .await
            {
                for token in tokens {
                    if token.is_valid() {
                        if let Ok(true) = token.verify_token(&request.token) {
                            matching_token = Some(token);
                            user_id = Some(user.id);
                            break;
                        }
                    }
                }
            }
            if matching_token.is_some() {
                break;
            }
        }

        let (token, uid) = match (matching_token, user_id) {
            (Some(t), Some(u)) => (t, u),
            _ => {
                return Err(AuthError::new("Invalid or expired reset token"));
            }
        };

        // Mark token as used
        if let Err(e) = self.password_reset_repository.mark_as_used(token.id).await {
            return Err(AuthError::new(&format!(
                "Failed to invalidate token: {}",
                e
            )));
        }

        // Update user password
        let new_password_hash = match User::hash_password(&request.password) {
            Ok(hash) => hash,
            Err(e) => {
                return Err(AuthError::new(&format!("Password hashing error: {}", e)));
            }
        };

        match self
            .user_repository
            .update(uid, None, None, Some(&new_password_hash))
            .await
        {
            Ok(Some(_)) => Ok(ResetPasswordResponse {
                message: "Password reset successfully".to_string(),
            }),
            Ok(None) => Err(AuthError::new("User not found")),
            Err(e) => Err(AuthError::new(&format!("Failed to update password: {}", e))),
        }
    }
}
