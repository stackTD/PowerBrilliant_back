use serde::{Serialize, Deserialize};
use sqlx::FromRow;
use uuid::Uuid;
use chrono::NaiveDateTime;
use serde_json::Value;

#[derive(Serialize, Deserialize, FromRow)]
pub struct BusinessAccount {
    pub id: Uuid,
    pub company_name: String,
    pub contact_person_name: String,
    pub email: String,
    pub password_hash: String,
    pub address: String,
    pub gst_number: Option<String>,
    pub website: Option<String>,
    pub phone: Option<String>,
    pub logo_url: Option<String>,
    pub description: Option<String>,
    pub industry: Option<String>,
    pub size: Option<String>,
    pub is_verified: bool,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime
}

#[derive(Deserialize)]
pub struct CreateBusinessAccount {
    pub company_name: String,
    pub contact_person_name: String,
    pub email: String,
    pub password_hash: String,
    pub address: String,
    pub gst_number: Option<String>,
    pub website: Option<String>,
    pub phone: Option<String>,
    pub logo_url: Option<String>,
    pub description: Option<String>,
    pub industry: Option<String>,
    pub size: Option<String>,
    pub is_verified: Option<bool>
}

#[derive(Deserialize)]
pub struct UpdateBusinessAccount {
    pub company_name: Option<String>,
    pub contact_person_name: Option<String>,
    pub email: Option<String>,
    pub password_hash: Option<String>,
    pub address: Option<String>,
    pub gst_number: Option<String>,
    pub website: Option<String>,
    pub phone: Option<String>,
    pub logo_url: Option<String>,
    pub description: Option<String>,
    pub industry: Option<String>,
    pub size: Option<String>,
    pub is_verified: Option<bool>
}
