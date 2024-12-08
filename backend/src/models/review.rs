use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Review {
    pub review_id: i32,
    pub sender_id: i32,
    pub receiver_id: i32,
    pub skill_id: i32,
    pub rating: i32,
    pub comment: String,
    pub created_at: DateTime<Utc>,
}