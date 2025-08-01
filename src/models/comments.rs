use serde::{Serialize, Deserialize};
use sqlx::FromRow;
use uuid::Uuid;
use chrono::NaiveDateTime;
use serde_json::Value;

#[derive(Deserialize)]
pub struct Pagination {
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

#[derive(Serialize, Deserialize, FromRow)]
pub struct Comments {
    pub id: Uuid,
    pub post_id: Uuid,
    pub author_id: Uuid,
    pub content: Value,
    pub parent_id: Option<Uuid>,
    pub created_at: NaiveDateTime,
    pub is_active: bool,
}

#[derive(Deserialize)]
pub struct Create_Comment {
    pub post_id: Uuid,
    pub author_id: Uuid,
    pub author_type: String, // 👈 Make sure this and all fields are `pub`
    pub content: serde_json::Value,
    pub parent_id: Option<Uuid>,
    pub is_active: Option<bool>,
}

#[derive(Deserialize)]
pub struct Update_Comment {
    pub content: Option<Value>,
    pub parent_id: Option<Uuid>,
    pub is_active: Option<bool>,
}

#[derive(Serialize, sqlx::FromRow)]
pub struct get_comments_by_post_id{
    pub id: Uuid,
    pub post_id: Uuid,
    pub author_id: Uuid,
    pub content: serde_json::Value,
    pub parent_id: Option<Uuid>,
    pub created_at: chrono::NaiveDateTime,
    pub is_active: bool,
    pub author_type: String,
    pub author_name: String,
    pub author_profile: Option<String>,
}

