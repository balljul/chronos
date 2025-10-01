use crate::app::models::auth::AuthError;
use crate::app::models::jwt::{LoginRequest, LoginResponse};
use crate::app::models::login_attempt::{AccountLockout, LoginAttempt};
use crate::app::repositories::login_attempt_repository::{
    AccountLockoutRepository, LoginAttemptRepository,
};
use crate::app::services::auth_service::AuthService;
use crate::app::services::jwt_service::JwtService;
use time::OffsetDateTime;
use uuid::Uuid;

pub struct SecureLoginService {
    auth_service: AuthService,
    jwt_service: JwtService,
    login_attempt_repository: LoginAttemptRepository,
    account_lockout_repository: AccountLockoutRepository,
}

impl SecureLoginService {
    pub fn new(
        auth_service: AuthService,
        jwt_service: JwtService,
        login_attempt_repository: LoginAttemptRepository,
        account_lockout_repository: AccountLockoutRepository,
    ) -> Self {
        Self {
            auth_service,
            jwt_service,
            login_attempt_repository,
            account_lockout_repository,
        }
    }

    pub async fn secure_login(
        &self,
        request: LoginRequest,
        ip_address: String,
        user_agent: Option<String>,
    ) -> Result<LoginResponse, AuthError> {
        // Step 1: Check IP rate limiting (5 attempts per 15 minutes)
        let rate_limit_window = OffsetDateTime::now_utc() - time::Duration::minutes(15);
        let ip_failures = self
            .login_attempt_repository
            .count_failed_attempts_by_ip(&ip_address, rate_limit_window)
            .await
            .map_err(|e| AuthError::new(&format!("Database error: {}", e)))?;

        if ip_failures >= 5 {
            let attempt = LoginAttempt::new_failure(
                ip_address,
                request.email.clone(),
                "IP rate limit exceeded".to_string(),
                user_agent,
            );

            if let Err(e) = self.login_attempt_repository.create_attempt(&attempt).await {
                eprintln!("Failed to log login attempt: {}", e);
            }

            return Err(AuthError::new(
                "Too many failed login attempts. Please try again later.",
            ));
        }

        // Step 2: Find user by email
        let user = match self.auth_service.find_user_by_email(&request.email).await {
            Ok(Some(user)) => user,
            Ok(None) => {
                // Log failed attempt for non-existent user
                let attempt = LoginAttempt::new_failure(
                    ip_address,
                    request.email,
                    "Invalid credentials".to_string(),
                    user_agent,
                );

                if let Err(e) = self.login_attempt_repository.create_attempt(&attempt).await {
                    eprintln!("Failed to log login attempt: {}", e);
                }

                return Err(AuthError::new("Invalid email or password"));
            }
            Err(e) => {
                return Err(AuthError::new(&format!("Database error: {}", e)));
            }
        };

        // Step 3: Check account lockout (10 failed attempts, 30 min lockout)
        if let Ok(Some(lockout)) = self
            .account_lockout_repository
            .get_active_lockout(user.id)
            .await
        {
            if lockout.is_locked() {
                let attempt = LoginAttempt::new_failure(
                    ip_address,
                    request.email.clone(),
                    "Account locked".to_string(),
                    user_agent,
                );

                if let Err(e) = self.login_attempt_repository.create_attempt(&attempt).await {
                    eprintln!("Failed to log login attempt: {}", e);
                }

                let locked_until = lockout
                    .locked_until
                    .format(&time::format_description::well_known::Rfc3339)
                    .unwrap_or("unknown time".to_string());
                return Err(AuthError::new(&format!(
                    "Account is temporarily locked until {}. Please try again later.",
                    locked_until
                )));
            }
        }

        // Step 4: Verify password using constant-time comparison
        let password_valid = match user.verify_password(&request.password) {
            Ok(valid) => valid,
            Err(_) => {
                let attempt = LoginAttempt::new_failure(
                    ip_address,
                    request.email.clone(),
                    "Password verification error".to_string(),
                    user_agent,
                );

                if let Err(e) = self.login_attempt_repository.create_attempt(&attempt).await {
                    eprintln!("Failed to log login attempt: {}", e);
                }

                return Err(AuthError::new("Authentication failed"));
            }
        };

        if !password_valid {
            // Step 5: Handle failed login attempt
            let attempt = LoginAttempt::new_failure(
                ip_address.clone(),
                request.email.clone(),
                "Invalid password".to_string(),
                user_agent,
            );

            if let Err(e) = self.login_attempt_repository.create_attempt(&attempt).await {
                eprintln!("Failed to log login attempt: {}", e);
            }

            // Check if we need to lock the account (10 failed attempts)
            let user_failure_window = OffsetDateTime::now_utc() - time::Duration::hours(1);
            if let Ok(user_failures) = self
                .login_attempt_repository
                .count_failed_attempts_by_email(&request.email, user_failure_window)
                .await
            {
                if user_failures >= 9 {
                    // This is the 10th failure, lock the account
                    let lockout = AccountLockout::new(user.id, (user_failures + 1) as i32, 30);
                    if let Err(e) = self
                        .account_lockout_repository
                        .create_lockout(&lockout)
                        .await
                    {
                        eprintln!("Failed to create account lockout: {}", e);
                    }
                    return Err(AuthError::new(
                        "Account has been temporarily locked due to too many failed login attempts. Please try again in 30 minutes.",
                    ));
                }
            }

            return Err(AuthError::new("Invalid email or password"));
        }

        // Step 6: Successful login - generate tokens
        let token_pair = match self.jwt_service.generate_token_pair(&user).await {
            Ok(tokens) => tokens,
            Err(_) => {
                let attempt = LoginAttempt::new_failure(
                    ip_address.clone(),
                    request.email.clone(),
                    "Token generation failed".to_string(),
                    user_agent,
                );

                if let Err(e) = self.login_attempt_repository.create_attempt(&attempt).await {
                    eprintln!("Failed to log login attempt: {}", e);
                }

                return Err(AuthError::new("Authentication failed"));
            }
        };

        // Step 7: Log successful login attempt
        let success_attempt =
            LoginAttempt::new_success(ip_address, request.email, user.id, user_agent);

        if let Err(e) = self
            .login_attempt_repository
            .create_attempt(&success_attempt)
            .await
        {
            eprintln!("Failed to log successful login attempt: {}", e);
        }

        // Step 8: Unlock account if it was locked (successful login resets lockout)
        if let Err(e) = self
            .account_lockout_repository
            .unlock_account(user.id)
            .await
        {
            eprintln!("Failed to unlock account: {}", e);
        }

        Ok(LoginResponse {
            message: "Login successful".to_string(),
            user: user.to_response(),
            tokens: token_pair,
        })
    }

    // Get login attempt statistics for security monitoring
    pub async fn get_login_statistics(&self, email: &str) -> Result<Vec<LoginAttempt>, AuthError> {
        self.login_attempt_repository
            .get_recent_attempts_by_email(email, 10)
            .await
            .map_err(|e| AuthError::new(&format!("Database error: {}", e)))
    }

    // Manual account unlock (admin function)
    pub async fn unlock_account(&self, user_id: Uuid) -> Result<(), AuthError> {
        self.account_lockout_repository
            .unlock_account(user_id)
            .await
            .map_err(|e| AuthError::new(&format!("Failed to unlock account: {}", e)))
    }

    // Maintenance: cleanup old login attempts
    pub async fn cleanup_old_login_attempts(&self, days: i64) -> Result<u64, AuthError> {
        let cutoff = OffsetDateTime::now_utc() - time::Duration::days(days);
        self.login_attempt_repository
            .cleanup_old_attempts(cutoff)
            .await
            .map_err(|e| AuthError::new(&format!("Cleanup failed: {}", e)))
    }
}
