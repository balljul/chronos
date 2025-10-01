use uuid::Uuid;
use crate::app::models::user::User;
use crate::app::repositories::user_repository::UserRepository;

#[derive(Debug)]
pub enum UserServiceError {
    DatabaseError(sqlx::Error),
    PasswordHashError(String),
}

impl std::fmt::Display for UserServiceError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            UserServiceError::DatabaseError(e) => write!(f, "Database error: {}", e),
            UserServiceError::PasswordHashError(e) => write!(f, "Password hash error: {}", e),
        }
    }
}

impl std::error::Error for UserServiceError {}

impl From<sqlx::Error> for UserServiceError {
    fn from(e: sqlx::Error) -> Self {
        UserServiceError::DatabaseError(e)
    }
}

impl From<argon2::password_hash::Error> for UserServiceError {
    fn from(e: argon2::password_hash::Error) -> Self {
        UserServiceError::PasswordHashError(e.to_string())
    }
}

pub struct UserService {
    repository: UserRepository,
}

impl UserService {
    pub fn new(repository: UserRepository) -> Self {
        Self { repository }
    }

    pub async fn create_user(&self, name: Option<String>, email: String, password: &str) -> Result<User, UserServiceError> {
        let user = User::new(name, email, password)?;
        let created_user = self.repository.create(&user).await?;
        Ok(created_user)
    }

    pub async fn get_user_by_id(&self, id: Uuid) -> Result<Option<User>, Box<dyn std::error::Error>> {
        let user = self.repository.find_by_id(id).await?;
        Ok(user)
    }

    pub async fn get_user_by_email(&self, email: &str) -> Result<Option<User>, Box<dyn std::error::Error>> {
        let user = self.repository.find_by_email(email).await?;
        Ok(user)
    }

    pub async fn get_all_users(&self) -> Result<Vec<User>, Box<dyn std::error::Error>> {
        let users = self.repository.get_all().await?;
        Ok(users)
    }

    pub async fn update_user(&self, id: Uuid, name: Option<String>, email: Option<String>, password: Option<String>) -> Result<Option<User>, UserServiceError> {
        let name_ref = name.as_deref();
        let email_ref = email.as_deref();
        let password_hash = if let Some(password) = password {
            Some(User::hash_password(&password)?)
        } else {
            None
        };
        let password_hash_ref = password_hash.as_deref();

        let user = self.repository.update(id, name_ref, email_ref, password_hash_ref).await?;
        Ok(user)
    }

    pub async fn delete_user(&self, id: Uuid) -> Result<bool, Box<dyn std::error::Error>> {
        let deleted = self.repository.delete(id).await?;
        Ok(deleted)
    }
}
