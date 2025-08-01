use serde::{Serialize, Deserialize};
use sqlx::FromRow;
use uuid::Uuid;
use chrono::NaiveDateTime;
use serde_json::Value;

#[derive(Serialize, Deserialize, FromRow)]
pub struct Followers {
    pub id: Uuid,
    pub follower_id: Uuid,
    pub followed_id: Uuid,
    pub followed_at: NaiveDateTime,
    is_following: bool,
    updated_at: NaiveDateTime,
}

#[derive(Serialize, Deserialize, FromRow)]
pub struct Create_Follower {
    pub follower_id: Uuid,
    pub followed_id: Uuid,
    pub is_following: bool,
}

#[derive(Deserialize)]
pub struct Update_Follower {
    pub is_following: Option<bool>,
}

#[derive(Serialize, FromRow)]
pub struct FollowerListUser {
    pub id: Uuid,
    pub first_name: String,
    pub last_name: String,
    pub username: String,
    pub profile_pic: Option<String>,
}

#[derive(Serialize, FromRow)]
pub struct FollowingListUser {
    pub id: Uuid,
    pub first_name: String,
    pub last_name: String,
    pub username: String,
    pub profile_pic: Option<String>,
}