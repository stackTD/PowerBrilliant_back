use serde::{Serialize, Deserialize};
use sqlx::FromRow;
use uuid::Uuid;
use chrono::NaiveDateTime;
use serde_json::Value;

#[derive(Serialize, Deserialize, FromRow)]
pub struct Community_Members {
    pub id: Uuid,
    pub community_id: Uuid,
    pub user_id: Uuid,
    pub role: String,
    pub joined_at: NaiveDateTime,
}

#[derive(Serialize, Deserialize, FromRow)]
pub struct Create_Community_Member {
    pub community_id: Uuid,
    pub user_id: Uuid,
    pub role: String,
    // pub joined_at: NaiveDateTime,
}

#[derive(Deserialize)]
pub struct Update_Community_Member {
    pub role: Option<String>,
}