use serde::{Serialize, Deserialize};
use sqlx::FromRow;
use uuid::Uuid;
use chrono::NaiveDateTime;
use serde_json::Value;





#[derive(Serialize, Deserialize, FromRow)]
pub struct Post {
    pub id: Uuid,
    pub community_id: Uuid,
    pub author_id: Uuid,
    pub title: Value,
    pub content: Value,
    pub post_type: String,
    pub created_at: NaiveDateTime,
    pub is_active: bool,
    pub tags: Vec<String>,
    pub likes: i32,
}

#[derive(Deserialize)]
pub struct CreatePost {
    pub community_id: Uuid,
    pub author_id: Uuid,
    pub title: serde_json::Value,
    pub content: serde_json::Value,
    pub post_type: String,
    pub is_active: Option<bool>,
    pub tags: Option<Vec<String>>,
    pub likes: Option<i32>,
    pub media: Option<Vec<NewMedia>>, 
}

#[derive(Deserialize)]
pub struct UpdatePost {
    pub title: Option<serde_json::Value>,
    pub content: Option<serde_json::Value>,
    pub post_type: Option<String>,
    pub is_active: Option<bool>,
    pub tags: Option<Vec<String>>,
    pub likes: Option<i32>,
}

#[derive(Serialize, Deserialize, FromRow)]
pub struct PostWithRank {
    pub id: Uuid,
    pub community_id: Option<Uuid>,
    pub author_id: Option<Uuid>,
    pub title: Option<serde_json::Value>,
    pub content: Option<serde_json::Value>,
    pub post_type: Option<String>,
    pub created_at: Option<chrono::NaiveDateTime>,
    pub is_active: Option<bool>,
    pub tags: Option<Vec<String>>,
    pub likes: Option<i32>,
    pub match_count: i32,
    pub like_count: i64, 
    pub media_urls: Option<Vec<String>>, // Added
}

#[derive(Deserialize)]
pub struct Pagination {
    pub limit: Option<i64>,
    pub offset: Option<i64>,
    pub sort: Option<String>, 
}

#[derive(Serialize, Deserialize, sqlx::Type, Debug, Clone, Copy)]
#[sqlx(type_name = "media_type", rename_all = "lowercase")]
#[serde(rename_all = "lowercase")]
pub enum MediaType {
    Image,
    Video,
    Gif,
}

#[derive(Serialize, Deserialize, sqlx::FromRow)]
pub struct PostMedia {
    pub id: Uuid,
    pub post_id: Uuid,
    pub media_type: MediaType,
    
    pub url: String,
    pub created_at: NaiveDateTime,
    
}

#[derive(Deserialize)]
pub struct NewMedia {
    pub media_type: MediaType,
    pub url: String,
}

