use actix_web::{web, HttpResponse, HttpRequest};
use uuid::Uuid;
use sqlx::PgPool;
use crate::models::job_list::{Job_listings, CreateJob, UpdateJob,Pagination};
use serde_json::json;

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


pub async fn create_job(
    req: HttpRequest,
    db: web::Data<PgPool>,
    json: web::Json<CreateJob>,
) -> HttpResponse {
    let lang = get_lang(&req);            
    println!("Language preference: {}", lang);
    let user = sqlx::query_as::<_, Job_listings>(
    r#"
    INSERT INTO job_listings (
        id, posted_by, applied_by, organisation, title, description,
        location, job_type, skills, notice_period, min_experience,
        salary_range, is_active, job_posted_by
    )
    VALUES (
        $1, $2, $3, $4, $5, $6,
        $7, $8, $9, $10, $11,
        $12, $13, $14
    )
    RETURNING *
    "#
)
.bind(Uuid::new_v4())
.bind(json.posted_by)
.bind(json.applied_by)
.bind(&json.organisation)
.bind(&json.title)
.bind(&json.description)
.bind(&json.location)
.bind(&json.job_type.to_string())
.bind(&json.skills)
.bind(&json.notice_period)
.bind(&json.min_experience)
.bind(&json.salary_range)
.bind(json.is_active.unwrap_or(true))
.bind(&json.job_posted_by)
.fetch_one(db.get_ref())
.await;


    match user {
        Ok(u) => HttpResponse::Ok().json(u),
        Err(e) => {
            eprintln!("DB error: {:?}", e);
            HttpResponse::InternalServerError().body("Error creating job")
        }
    }
}

pub async fn get_joblist(
    db: web::Data<PgPool>,
    web::Query(pagination): web::Query<Pagination>,
) -> HttpResponse {
    let limit = pagination.limit.unwrap_or(5);
    let offset = pagination.offset.unwrap_or(0);
 
    let jobs = sqlx::query_as::<_, Job_listings>(
        "SELECT * FROM job_listings ORDER BY created_at DESC LIMIT $1 OFFSET $2"
    )
    .bind(limit)
    .bind(offset)
    .fetch_all(db.get_ref())
    .await;
 
    match jobs {
        Ok(jobs) => HttpResponse::Ok().json(jobs),
        Err(e) => {
            eprintln!("Database error: {:?}", e);
            HttpResponse::InternalServerError().body("Error retrieving job adv")
        }
    }
}
pub async fn delete_job_by_id(
    db: web::Data<PgPool>,
    id: web::Path<Uuid>,  // Changed from email to UUID
) -> HttpResponse {
    let result = sqlx::query!(
        "DELETE FROM job_listings WHERE id = $1 RETURNING id",
        id.into_inner()
    )
    .fetch_optional(db.get_ref())
    .await;

    match result {
        Ok(Some(_)) => HttpResponse::Ok().json(json!({
            "status": "success",
            "message": "Job deleted successfully"
        })),
        Ok(None) => HttpResponse::NotFound().json(json!({
            "status": "error",
            "message": "Job not found"
        })),
        Err(e) => {
            eprintln!("Database error: {:?}", e);
            HttpResponse::InternalServerError().json(json!({
                "status": "error",
                "message": "Error deleting job"
            }))
        }
    }
}


// ...existing code...

pub async fn update_job_by_id(
    db: web::Data<PgPool>,
    id: web::Path<Uuid>,
    json: web::Json<UpdateJob>,
) -> HttpResponse {
    let query_result = sqlx::query!(
        r#"
        UPDATE job_listings 
SET 
    posted_by = COALESCE($1, posted_by),
    applied_by = COALESCE($2, applied_by),
    organisation = COALESCE($3, organisation),
    title = COALESCE($4, title),
    description = COALESCE($5, description),
    location = COALESCE($6, location),
    job_type = COALESCE($7, job_type),
    skills = COALESCE($8, skills),
    notice_period = COALESCE($9, notice_period),
    min_experience = COALESCE($10, min_experience),
    salary_range = COALESCE($11, salary_range),
    job_posted_by = COALESCE($12, job_posted_by),
    is_active = COALESCE($13, is_active),
    updated_at = CURRENT_TIMESTAMP
WHERE id = $14
RETURNING id

        "#,
        json.posted_by,
json.applied_by,
json.organisation.as_deref(),
json.title,
json.description,
json.location.as_deref(),
json.job_type.as_deref(),
&json.skills as _,
json.notice_period.as_deref(),
json.min_experience,
json.salary_range.as_deref(),
json.job_posted_by.as_deref(),
json.is_active,
id.into_inner(),

    )
    .fetch_optional(db.get_ref())
    .await;

    match query_result {
        Ok(Some(_)) => HttpResponse::Ok().json(json!({
            "status": "success",
            "message": "Job updated successfully"
        })),
        Ok(None) => HttpResponse::NotFound().json(json!({
            "status": "error",
            "message": "Job not found"
        })),
        Err(e) => {
            eprintln!("Database error: {:?}", e);
            HttpResponse::InternalServerError().json(json!({
                "status": "error",
                "message": "Error updating job"
            }))
        }
    }
}
pub async fn get_job_by_id(
    db: web::Data<PgPool>,
    id: web::Path<Uuid>,
) -> HttpResponse {
    let result = sqlx::query_as::<_, Job_listings>(
        "SELECT * FROM job_listings WHERE id = $1"
    )
    .bind(id.into_inner())
    .fetch_optional(db.get_ref())
    .await;

    match result {
        Ok(Some(job)) => HttpResponse::Ok().json(job),
        Ok(None) => HttpResponse::NotFound().json(json!({
            "status": "error",
            "message": "Job not found"
        })),
        Err(e) => {
            eprintln!("Database error: {:?}", e);
            HttpResponse::InternalServerError().json(json!({
                "status": "error",
                "message": "Error retrieving job"
            }))
        }
    }
}

pub async fn get_jobs_by_posted_id(
    db: web::Data<PgPool>,
    posted_id: web::Path<Uuid>,
) -> HttpResponse {
    let result = sqlx::query_as::<_, Job_listings>(
        "SELECT * FROM job_listings WHERE posted_by = $1 ORDER BY created_at DESC"
    )
    .bind(posted_id.into_inner())
    .fetch_all(db.get_ref())
    .await;

    match result {
        Ok(jobs) => HttpResponse::Ok().json(jobs),
        Err(e) => {
            eprintln!("Database error: {:?}", e);
            HttpResponse::InternalServerError().json(json!({
                "status": "error",
                "message": "Error retrieving jobs"
            }))
        }
    }
}
