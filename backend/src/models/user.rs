use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct User {
    pub user_id: i32,
    pub username: String,
    pub email: String,
    pub password_hash: String,
    pub role: String,
    pub profile_picture: Option<String>,
    pub bio: Option<String>,
    pub created_at: DateTime<Utc>,
}