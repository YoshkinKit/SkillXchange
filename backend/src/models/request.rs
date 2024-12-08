use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Request {
    pub request_id: i32,
    pub sender_id: i32,
    pub receiver_id: i32,
    pub skill_id: i32,
    pub status: String,
    pub cover_letter: String,
    pub created_at: DateTime<Utc>,
}