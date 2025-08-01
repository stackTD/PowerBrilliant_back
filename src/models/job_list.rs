use serde::{Serialize, Deserialize};
use sqlx::FromRow;
use uuid::Uuid;
use chrono::NaiveDateTime;
use serde_json::Value;
use std::fmt;

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum JobType {
    Remote,
    Hybrid,
    Onsite
}
impl fmt::Display for JobType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            JobType::Remote => write!(f, "remote"),
            JobType::Hybrid => write!(f, "hybrid"),
            JobType::Onsite => write!(f, "onsite"),
        }
    }
}

#[derive(Deserialize)]
pub struct Pagination {
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

#[derive(Serialize, Deserialize, FromRow)]
pub struct Job_listings {
    pub id: Uuid,
    pub posted_by: Uuid,
    pub applied_by: Option<Uuid>,
    pub organisation: String,
    pub title: Value,
    pub description: Value,
    pub location: String,
    pub job_type: Option<String>,
    pub notice_period: Option<String>,
    pub min_experience: Option<i32>,
    pub skills: Option<Vec<String>>,
    pub salary_range: Option<String>,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
    pub is_active: Option<bool>,
    pub job_posted_by: Option<String>,
}

#[derive(Deserialize)]
pub struct CreateJob {
    pub posted_by: Uuid,
    pub applied_by: Option<Uuid>,
    pub organisation: String,
    pub title: Value,
    pub description: Value,
    pub location: String,
    pub job_type: JobType,
    pub skills: Vec<String>,
    pub notice_period: String,
    pub min_experience: Option<i32>,
    pub salary_range: String,
    pub job_posted_by: String,
    pub is_active: Option<bool>,
}

#[derive(Deserialize)]
pub struct UpdateJob {
    pub posted_by: Option<Uuid>,
    pub applied_by: Option<Uuid>,
    pub organisation: Option<String>,
    pub title: Option<Value>,
    pub description: Option<Value>,
    pub location: Option<String>,
    pub job_type: Option<String>,
    pub skills: Option<Vec<String>>,
    pub notice_period: Option<String>,
    pub min_experience: Option<i32>,
    pub salary_range: Option<String>,
    pub job_posted_by: Option<String>,
    pub is_active: Option<bool>,
}