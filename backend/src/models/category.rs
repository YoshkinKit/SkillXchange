use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Category {
    pub category_id: i32,
    pub title: String,
    pub description: String,
    pub created_at: chrono::NaiveDateTime,
}
