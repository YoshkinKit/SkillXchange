use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Message {
    pub message_id: i32,
    pub sender_id: i32,
    pub receiver_id: i32,
    pub content: String,
    pub created_at: chrono::NaiveDateTime,
}