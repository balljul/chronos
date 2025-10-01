use validator::Validate;
use crate::app::models::user::User;
use crate::app::models::auth::{RegisterRequest, RegisterResponse, AuthError};
use crate::app::repositories::user_repository::UserRepository;

pub struct AuthService {
    user_repository: UserRepository,
}

impl AuthService {
    pub fn new(user_repository: UserRepository) -> Self {
        Self { user_repository }
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
            Ok(created_user) => {
                Ok(RegisterResponse {
                    message: "User registered successfully".to_string(),
                    user: created_user.to_response(),
                })
            }
            Err(e) => {
                // Check if it's a unique constraint violation (email already exists)
                if e.to_string().contains("duplicate key") || e.to_string().contains("UNIQUE constraint") {
                    Err(AuthError::new("Email already registered"))
                } else {
                    Err(AuthError::new(&format!("Failed to create user: {}", e)))
                }
            }
        }
    }
}