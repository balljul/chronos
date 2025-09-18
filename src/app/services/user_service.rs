use serde_json;
use sqlx;
use crate::app::models::user::User;

pub async fn test() -> Result<serde_json::Value, Box<dyn std::error::Error>> {
    let users = sqlx::query!("
        SELECT *
        FROM users
     ").fetch_all().await?;

    let user = User::new(
        "Julius Ball".to_string(),
        "contact@juliusball.com".to_string(),
    );

    let json = serde_json::to_value(&users)?;
    Ok(json)
}
