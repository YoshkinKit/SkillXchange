use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct UserSkills {
    pub user_id: i32,
    pub skill_id: i32,
}