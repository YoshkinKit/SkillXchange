use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Skill {
    pub skill_id: i32,
    pub title: String,
    pub description: String,
    pub category_id: i32,
    pub created_at: DateTime<Utc>,
    pub is_deleted: bool,
}