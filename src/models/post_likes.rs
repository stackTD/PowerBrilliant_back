use serde::{Serialize, Deserialize};
use sqlx::FromRow;
use uuid::Uuid;
use chrono::NaiveDateTime;
use serde_json::Value;

#[derive(Serialize, Deserialize, FromRow)]
pub struct Post_Likes {
    pub id: Uuid,
    pub post_id: Uuid,
    pub user_id: Uuid,
    pub liked_at: NaiveDateTime,
    pub is_liked: bool,
    pub updated_at: NaiveDateTime,
}

#[derive(Serialize, Deserialize, FromRow)]
pub struct Create_Post_Likes {
    pub post_id: Uuid,
    pub user_id: Uuid,
    pub is_liked: bool,
}


// Add this to your models/post_likes.rs
#[derive(Deserialize)]
pub struct Update_Post_Likes {
    pub is_liked: Option<bool>,
}