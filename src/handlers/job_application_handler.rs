use actix_web::{web, HttpResponse, HttpRequest};
use uuid::Uuid;
use sqlx::PgPool;
 use serde::Serialize;

use serde_json::json;
use crate::models::user::User;
use crate::models::job_applications::{JobApplication, CreateJobApplicationRequest};

fn get_lang(req: &HttpRequest) -> String {
    req.headers()
        .get("Accept-Language")
        .and_then(|val| val.to_str().ok())
        .map(|s| {
            if s.contains("hi") {
                "hi"
            } else {
                "en"
            }
        })
        .unwrap_or("en")
        .to_string()
}




pub async fn create_job_application(
    req: HttpRequest,
    db: web::Data<PgPool>,
    json: web::Json<CreateJobApplicationRequest>,
) -> HttpResponse {
    let lang = get_lang(&req);
    println!("Language preference: {}", lang);
    
    // Get user's resume
    let user_result = sqlx::query_as::<_, User>(
        "SELECT * FROM users WHERE id = $1"
    )
    .bind(&json.applicant_id)
    .fetch_one(db.get_ref())
    .await;

    let user = match user_result {
        Ok(user) => user,
        Err(e) => {
            eprintln!("DB error fetching user: {:?}", e);
            return HttpResponse::BadRequest().body("User not found");
        }
    };

    let resume_url = user.resume_url.unwrap_or_else(|| "".to_string());
    // Create application
    let application = sqlx::query_as::<_, JobApplication>(
        r#"
        INSERT INTO job_applications (
            id, job_id, applicant_id, cover_letter, 
            resume_url, application_date, status, 
            updated_at, is_active
        )
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
        RETURNING *
        "#
    )
    .bind(Uuid::new_v4())
    .bind(&json.job_id)
    .bind(&json.applicant_id)
    .bind(&json.cover_letter)
    .bind(resume_url) // Use user's resume URL (can be empty)
    .bind(chrono::Utc::now().naive_utc())
    .bind("applied") // Default status
    .bind(chrono::Utc::now().naive_utc())
    .bind(true)
    .fetch_one(db.get_ref())
    .await;

    match application {
        Ok(app) => HttpResponse::Ok().json(app),
        Err(e) => {
            eprintln!("DB error: {:?}", e);
            HttpResponse::InternalServerError().json(json!({
                "error": "Error creating job application",
                "details": e.to_string()
            }))
        }
    }
}

// New handler for business to view applications
pub async fn get_applications_for_job(
    db: web::Data<PgPool>,
    job_id: web::Path<Uuid>,
) -> HttpResponse {
    let applications = sqlx::query_as::<_, ApplicationWithUser>(
        r#"
        SELECT 
            ja.id, ja.job_id, ja.cover_letter, ja.resume_url, 
            ja.application_date, ja.status, ja.updated_at, ja.is_active,
            u.id as user_id, u.first_name, u.last_name, u.email, 
            u.profile_pic, u.organisation, u.bio, u.mobile
        FROM job_applications ja
        JOIN users u ON ja.applicant_id = u.id
        WHERE ja.job_id = $1
        "#
    )
    .bind(job_id.into_inner())
    .fetch_all(db.get_ref())
    .await;

    match applications {
        Ok(apps) => HttpResponse::Ok().json(apps),
        Err(e) => {
            eprintln!("DB error: {:?}", e);
            HttpResponse::InternalServerError().body("Error fetching applications")
        }
    }
}

#[derive(Serialize, sqlx::FromRow)]
struct ApplicationWithUser {
    // Job application fields (non-nullable in DB)
    id: Uuid,
    job_id: Uuid,
    cover_letter: Option<String>,
    resume_url: String, // Changed to non-optional
    application_date: chrono::NaiveDateTime, // Changed to non-optional
    status: String, // Changed to non-optional
    updated_at: chrono::NaiveDateTime, // Changed to non-optional
    is_active: bool, // Changed to non-optional
    
    // User fields
    user_id: Uuid,
    first_name: String, // Changed to non-optional
    last_name: String, // Changed to non-optional
    email: String, // Changed to non-optional
    profile_pic: Option<String>,
    organisation: Option<String>,
    bio: Option<serde_json::Value>,
    mobile: Option<String>,
}


pub async fn get_applied_jobs_count_for_user(
    db: web::Data<PgPool>,
    user_id: web::Path<Uuid>,
) -> HttpResponse {
    let count = sqlx::query_scalar!(
        "SELECT COUNT(*) FROM job_applications WHERE applicant_id = $1",
        user_id.into_inner()
    )
    .fetch_one(db.get_ref())
    .await;

    match count {
        Ok(count) => HttpResponse::Ok().json(json!({ "count": count })),
        Err(e) => {
            eprintln!("DB error: {:?}", e);
            HttpResponse::InternalServerError().json(json!({
                "error": "Error fetching count",
                "details": e.to_string()
            }))
        }
    }
}

#[derive(Serialize, sqlx::FromRow)]
pub struct AppliedJob {
    pub application_id: Uuid,
    pub application_status: String,
    pub application_date: chrono::NaiveDateTime,
    pub job_id: Uuid,
    pub title: serde_json::Value,
    pub organisation: String,
    pub location: String,
}

pub async fn get_applied_jobs_for_user(
    db: web::Data<PgPool>,
    user_id: web::Path<Uuid>,
) -> HttpResponse {
    let applications = sqlx::query_as::<_, AppliedJob>(
        r#"
        SELECT 
            ja.id AS application_id,
            ja.status AS application_status,
            ja.application_date,
            jl.id AS job_id,
            jl.title,
            jl.organisation,
            jl.location
        FROM job_applications ja
        JOIN job_listings jl ON ja.job_id = jl.id
        WHERE ja.applicant_id = $1
        ORDER BY ja.application_date DESC
        "#
    )
    .bind(user_id.into_inner())
    .fetch_all(db.get_ref())
    .await;

    match applications {
        Ok(apps) => HttpResponse::Ok().json(apps),
        Err(e) => {
            eprintln!("DB error: {:?}", e);
            HttpResponse::InternalServerError().json(json!({
                "error": "Error fetching applied jobs",
                "details": e.to_string()
            }))
        }
    }
}
#[derive(Serialize, sqlx::FromRow)]
pub struct UserApplication {
    id: Uuid,
    job_id: Uuid,
    applicant_id: Uuid,
    cover_letter: Option<String>,
    resume_url: String,
    application_date: chrono::NaiveDateTime,
    status: String,
    updated_at: chrono::NaiveDateTime,
}

pub async fn get_application_for_job_and_user(
    db: web::Data<PgPool>,
    path: web::Path<(Uuid, Uuid)>,
) -> HttpResponse {
    let (job_id, user_id) = path.into_inner();
    let application = sqlx::query_as::<_, UserApplication>(
        "SELECT * FROM job_applications 
        WHERE job_id = $1 AND applicant_id = $2"
    )
    .bind(job_id)
    .bind(user_id)
    .fetch_optional(db.get_ref())
    .await;

    match application {
        Ok(Some(app)) => HttpResponse::Ok().json(app),
        Ok(None) => HttpResponse::NotFound().json(json!({
            "error": "Application not found"
        })),
        Err(e) => {
            eprintln!("DB error: {:?}", e);
            HttpResponse::InternalServerError().json(json!({
                "error": "Error fetching application",
                "details": e.to_string()
            }))
        }
    }
}