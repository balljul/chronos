use uuid::Uuid;
use serde::{Serialize, Deserialize};
use time;
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    pub id: Uuid,
    pub name: String,
    pub email: String,
    pub password: String,
    pub created_at: Option<time::OffsetDateTime>,
    pub updated_at: Option<time::OffsetDateTime>
}

impl User {
    pub fn new(name: String, email: String, password: String) -> Self {

        let mut hasher = DefaultHasher::new();
        password.hash(&mut hasher);
        let hashed_password = hasher.finish().to_string();

        Self {
            id: Uuid::new_v4(),
            name,
            email,
            password: hashed_password,
            created_at: Some(time::OffsetDateTime::now_utc()),
            updated_at: Some(time::OffsetDateTime::now_utc()),
        }
    }
}
