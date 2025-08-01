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
pub struct Community {
    pub id: Uuid,
    pub name: String,
    pub description: Option<Value>,
    pub creator_id: Option<Uuid>,              // Changed to Option
    pub creator_business_id: Option<Uuid>,     // New field
    pub creator_type: String,                  // New field
    pub banner_url: Option<String>,
    pub is_public: bool,
    pub is_active: bool,
    pub created_at: NaiveDateTime,
}



#[derive(Deserialize)]
pub struct Create_Community {
    pub name: String,
    pub description: Option<serde_json::Value>,
    pub creator_id: Option<Uuid>,              // Changed to Option
    pub creator_business_id: Option<Uuid>,     // New field
    pub creator_type: String,                  // New field
    pub banner_url: Option<String>,
    pub is_public: Option<bool>,
    pub is_active: Option<bool>,
}

#[derive(Deserialize)]
pub struct Update_Community {
    pub name: Option<String>,
    pub description: Option<serde_json::Value>,
    pub banner_url: Option<String>,
    pub is_public: Option<bool>,
    pub is_active: Option<bool>,
}
