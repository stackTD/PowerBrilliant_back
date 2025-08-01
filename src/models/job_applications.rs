use serde::{Serialize, Deserialize};
use sqlx::FromRow;
use uuid::Uuid;
use chrono::NaiveDateTime;

#[derive(Serialize, Deserialize, FromRow)]
pub struct JobApplication {
    pub id: Uuid,
    pub job_id: Uuid,
    pub applicant_id: Uuid,
    pub cover_letter: Option<String>,
    pub resume_url: String,
    pub application_date: NaiveDateTime,
    pub status: String,
    pub updated_at: NaiveDateTime,
    pub is_active: bool,
}

// Updated to only require job_id and cover letter
#[derive(Deserialize)]
pub struct CreateJobApplicationRequest {
    pub job_id: Uuid,
    pub applicant_id: Uuid,
    pub cover_letter: Option<String>,
}