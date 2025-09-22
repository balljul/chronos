use uuid::Uuid;
use crate::app::models::user::User;
use crate::app::repositories::user_repository::UserRepository;

pub struct UserService {
    repository: UserRepository,
}

impl UserService {
    pub fn new(repository: UserRepository) -> Self {
        Self { repository }
    }

    pub async fn create_user(&self, name: String, email: String, password: String) -> Result<User, Box<dyn std::error::Error>> {
        let user = User::new(name, email, password);
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

    pub async fn update_user(&self, id: Uuid, name: Option<String>, email: Option<String>, password: Option<String>) -> Result<Option<User>, Box<dyn std::error::Error>> {
        let name_ref = name.as_deref();
        let email_ref = email.as_deref();
        let password_ref = password.as_deref();

        let user = self.repository.update(id, name_ref, email_ref, password_ref).await?;
        Ok(user)
    }

    pub async fn delete_user(&self, id: Uuid) -> Result<bool, Box<dyn std::error::Error>> {
        let deleted = self.repository.delete(id).await?;
        Ok(deleted)
    }
}
