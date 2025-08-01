// models/user.rs
use serde::{Serialize, Deserialize};
use sqlx::FromRow;
use uuid::Uuid;
use chrono::NaiveDateTime;
use serde_json::Value;

#[derive(Serialize, Deserialize, FromRow, Debug)]
pub struct User {
    pub id: Uuid,
    pub first_name: String,
    pub last_name: String,
    pub username: String,
    pub password: String,
    pub mobile: Option<String>,
    pub email: String,
    pub organisation: Option<String>,
    pub bio: Option<serde_json::Value>,
    pub profile_pic: Option<String>,
    pub created_at: Option<NaiveDateTime>,
    pub updated_at: Option<NaiveDateTime>,
    pub is_active: Option<bool>,
    pub resume_url: Option<String>,
    pub interests: Option<Vec<String>>,
    pub provider: Option<String>,
    pub provider_user_id: Option<String>,
    pub access_token: Option<String>,
    pub refresh_token: Option<String>,
    pub oauth_email: Option<String>,
    pub college: Option<String>,
    pub batch: Option<String>,
    pub stream: Option<String>,
    pub linkedin: Option<String>,
    pub github: Option<String>,
    pub skills: Option<Vec<String>>,
    pub work_experiences: Option<serde_json::Value>,
    pub projects: Option<serde_json::Value>,
    pub pronouns: Option<String>,
    pub location: Option<String>,
    pub hackingon: Option<String>,
    pub learning: Option<String>,
    pub availablefor: Option<String>,
}

#[derive(Deserialize, Debug)] // Added Debug
pub struct CreateUser {
    pub first_name: String,
    pub last_name: String,
    pub username: String,
    pub password: String,
    pub email: String,
    pub resume_url: Option<String>,
    pub bio: Option<serde_json::Value>,
    pub interests: Option<Vec<String>>,
    pub provider: Option<String>,
    pub provider_user_id: Option<String>,
    pub access_token: Option<String>,
    pub profile_pic: Option<String>,
    // Removed refresh_token and oauth_email as they are not part of creation payload from client
    // They will be set by the backend (e.g., from Google OAuth response)
}

#[derive(Deserialize, Debug)]
pub struct UpdateUser {
    // Existing optional fields
    pub first_name: Option<String>,
    pub last_name: Option<String>,
    pub username: Option<String>,
    pub password: Option<String>, // Consider security implications of updating passwords this way
    pub mobile: Option<String>,
    pub organisation: Option<String>,
    pub bio: Option<String>,
    pub profile_pic: Option<String>,
    pub resume_url: Option<String>,
    pub is_active: Option<bool>,

    // New fields from the frontend
    pub college: Option<String>,
    pub batch: Option<String>,
    pub stream: Option<String>,
    pub linkedin: Option<String>,
    pub github: Option<String>,
    pub skills: Option<Vec<String>>,
    pub interest: Option<String>, // Note: singular 'interest' from frontend
    pub work_experiences: Option<Value>, // For JSONB
    pub projects: Option<Value>, 

    // Add these fields to match your DB and fix the errors
    pub pronouns: Option<String>,
    pub location: Option<String>,
    pub hacking_on: Option<String>,
    pub learning: Option<String>,
    pub available_for: Option<String>,        // For JSONB
}