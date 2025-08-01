use serde::{Serialize, Deserialize};
use sqlx::FromRow;
use uuid::Uuid;
use chrono::NaiveDateTime;
use serde_json::Value;

#[derive(Serialize, Deserialize, FromRow)]
pub struct User_Badges {
    pub id: Uuid,
    pub user_id: Uuid,
    pub badge_name: String,
    pub badge_icon: String,
    pub awarded_at: NaiveDateTime,
}
#[derive(Deserialize)]
pub struct Create_User_Badge {
    pub user_id: Uuid,
    pub badge_name: String,
    pub badge_icon: String,
}