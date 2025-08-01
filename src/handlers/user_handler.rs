// src/handlers/user_handler.rs
use actix_web::{web, HttpResponse, HttpRequest, get, Responder};
use uuid::Uuid;
use sqlx::PgPool;
use sqlx::{Pool, Postgres};
use sqlx::QueryBuilder;
use crate::models::user::{CreateUser, User, UpdateUser};
use serde_json::json;
use std::collections::HashMap;


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

#[derive(serde::Serialize, sqlx::FromRow)]
struct PublicUser {
    id: Uuid,
    username: String,
    first_name: String,
    last_name: String,
    profile_pic: Option<String>,
    bio: Option<serde_json::Value>,
    email: String,
    organisation: Option<String>,
    created_at: Option<chrono::NaiveDateTime>,
}

pub async fn create_user(
    req: HttpRequest,
    db: web::Data<PgPool>,
    json: web::Json<CreateUser>,
) -> HttpResponse {
    let lang = get_lang(&req);            
    println!("Language preference: {}", lang);

    // Ensure all fields from CreateUser and default/generated fields are handled.
    // The INSERT statement should list all columns you are providing values for,
    // and the number of placeholders ($1, $2, ...) must match the number of .bind() calls.
    // Also, ensure the types match.
    let user = sqlx::query_as::<_, User>(
        r#"
        INSERT INTO users (
            id,
            first_name,
            last_name,
            username,
            password,
            email,
            resume_url,
            bio,
            interests,
            provider,
            provider_user_id,
            access_token
        )
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12)
        RETURNING *
        "#
    )
    .bind(Uuid::new_v4())
    .bind(&json.first_name)
    .bind(&json.last_name)
    .bind(&json.username)
    .bind(&json.password)
    .bind(&json.email)
    .bind(&json.resume_url)
    .bind(&json.bio) // Bind bio from CreateUser
    .bind(&json.interests) // Bind interests from CreateUser
    .bind(&json.provider)
    .bind(&json.provider_user_id)
    .bind(&json.access_token)
    .fetch_one(db.get_ref())
    .await;

    match user {
        Ok(u) => HttpResponse::Ok().json(u),
        Err(e) => {
            eprintln!("DB error: {:?}", e);
            // Provide more specific error message if possible (e.g., unique constraint violation)
            HttpResponse::InternalServerError().body(format!("Error creating user: {}", e))
        }
    }
}

pub async fn get_users(db: web::Data<PgPool>) -> HttpResponse {
    // With SELECT *, it's crucial that the User struct exactly matches the database table schema.
    // We already fixed `User` to include `refresh_token` as Option<String>,
    // and other fields as Options where appropriate.
    let users = sqlx::query_as::<_, User>(
        "SELECT * FROM users ORDER BY created_at DESC"
    )
    .fetch_all(db.get_ref())
    .await;

    match users {
        Ok(users) => HttpResponse::Ok().json(users),
        Err(e) => {
            eprintln!("Database error: {:?}", e);
            HttpResponse::InternalServerError().body("Error retrieving users")
        }
    }
}

pub async fn delete_user_by_email(
    db: web::Data<PgPool>,
    email: web::Path<String>,
) -> HttpResponse {
    let result = sqlx::query!(
        "DELETE FROM users WHERE email = $1 RETURNING email",
        email.into_inner()
    )
    .fetch_optional(db.get_ref())
    .await;

    match result {
        Ok(Some(_)) => HttpResponse::Ok().json("User deleted successfully"),
        Ok(None) => HttpResponse::NotFound().json("User not found"),
        Err(e) => {
            eprintln!("Database error: {:?}", e);
            HttpResponse::InternalServerError().json("Error deleting user")
        }
    }
}

pub async fn update_user_by_email(
    db: web::Data<PgPool>,
    email_path: web::Path<String>,
    json: web::Json<UpdateUser>,
) -> HttpResponse {
    let email = email_path.into_inner();
    let payload = json.into_inner();

    let mut builder = QueryBuilder::new("UPDATE users SET ");
    let mut first = true;

    macro_rules! push_field {
        ($opt:expr, $name:expr) => {
            if let Some(val) = $opt {
                if !first {
                    builder.push(", ");
                }
                builder.push(concat!($name, " = "));
                builder.push_bind(val);
                first = false;
            }
        };
    }

    push_field!(&payload.first_name, "first_name");
    push_field!(&payload.last_name, "last_name");
    push_field!(&payload.username, "username");
    push_field!(&payload.password, "password");
    push_field!(&payload.mobile, "mobile");
    push_field!(&payload.organisation, "organisation");

    
    if let Some(bio) = &payload.bio {
        if !first {
            builder.push(", ");
        }
        builder.push("bio = ");
        builder.push_bind(serde_json::json!(bio));
        first = false;
    }
    push_field!(&payload.profile_pic, "profile_pic");
    push_field!(&payload.resume_url, "resume_url");
    if let Some(is_active) = payload.is_active {
        if !first {
            builder.push(", ");
        }
        builder.push("is_active = ");
        builder.push_bind(is_active);
        first = false;
    }
    push_field!(&payload.college, "college");
    push_field!(&payload.batch, "batch");
    push_field!(&payload.stream, "stream");
    push_field!(&payload.linkedin, "linkedin");
    push_field!(&payload.github, "github");
    push_field!(&payload.skills, "skills");
    if let Some(interest) = &payload.interest {
        if !first {
            builder.push(", ");
        }
        builder.push("interests = ");
        builder.push_bind(vec![interest.clone()]);
        first = false;
    }
    push_field!(&payload.work_experiences, "work_experiences");
    push_field!(&payload.projects, "projects");
    push_field!(&payload.pronouns, "pronouns");
    push_field!(&payload.location, "location");
    push_field!(&payload.hacking_on, "hackingon");
    push_field!(&payload.learning, "learning");
    push_field!(&payload.available_for, "availablefor");

    if !first {
        builder.push(", ");
    }
    builder.push("updated_at = CURRENT_TIMESTAMP");

    builder.push(" WHERE email = ");
    builder.push_bind(&email);
    builder.push(" RETURNING *");

    let query = builder.build_query_as::<User>();

    match query.fetch_one(db.get_ref()).await {
        Ok(updated_user) => HttpResponse::Ok().json(updated_user),
        Err(sqlx::Error::RowNotFound) => HttpResponse::NotFound().json("User not found"),
        Err(e) => {
            eprintln!("Database error: {:?}", e);
            HttpResponse::InternalServerError().json(format!("Error updating user: {}", e))
        }
    }
}


pub async fn get_user_by_id(
    db: web::Data<PgPool>,
    user_id: web::Path<Uuid>,
) -> HttpResponse {
    // `SELECT *` here is fine if User struct matches DB.
    let user = sqlx::query_as::<_, User>(
        "SELECT * FROM users WHERE id = $1"
    )
    .bind(*user_id)
    .fetch_optional(db.get_ref())
    .await;

    match user {
        Ok(Some(user)) => HttpResponse::Ok().json(user),
        Ok(None) => HttpResponse::NotFound().json("User not found"),
        Err(e) => {
            eprintln!("Database error: {:?}", e);
            HttpResponse::InternalServerError().json("Error retrieving user")
        }
    }
}

#[derive(serde::Serialize, sqlx::FromRow)]
struct PublicProfileUser {
    id: Uuid,
    username: String,
    first_name: String,
    last_name: String,
    profile_pic: Option<String>,
    bio: Option<serde_json::Value>,
    email: String,
    organisation: Option<String>,
    created_at: Option<chrono::NaiveDateTime>,
    college: Option<String>,
    stream: Option<String>,
    linkedin: Option<String>,
    github: Option<String>,
    skills: Option<Vec<String>>,
    interests: Option<Vec<String>>,
    work_experiences: Option<serde_json::Value>,
    projects: Option<serde_json::Value>,
    batch: Option<String>,
    pronouns: Option<String>,
    location: Option<String>,
    hackingon: Option<String>,
    learning: Option<String>,
    availablefor: Option<String>,
}

pub async fn get_public_profile(
    db: web::Data<PgPool>,
    identifier: web::Path<String>,
) -> impl Responder {
    // First try to find by username
    let by_username = sqlx::query_as::<_, PublicProfileUser>(
        r#"
        SELECT 
            id, username, first_name, last_name, profile_pic, bio, email, 
            organisation, created_at, college, stream, linkedin, github, 
            skills, interests, work_experiences, projects, batch,
            pronouns, location, hackingon, learning, availablefor
        FROM users
        WHERE username = $1
        "#
    )
    .bind(identifier.as_str())
    .fetch_optional(db.get_ref())
    .await;

    if let Ok(Some(user)) = by_username {
        return HttpResponse::Ok().json(user);
    }

    // Try by email
    let by_email = sqlx::query_as::<_, PublicProfileUser>(
        r#"
        SELECT 
            id, username, first_name, last_name, profile_pic, bio, email, 
            organisation, created_at, college, stream, linkedin, github, 
            skills, interests, work_experiences, projects, batch,
            pronouns, location, hackingon, learning, availablefor
        FROM users
        WHERE email = $1
        "#
    )
    .bind(identifier.as_str())
    .fetch_optional(db.get_ref())
    .await;

    match by_email {
        Ok(Some(user)) => HttpResponse::Ok().json(user),
        Ok(None) => HttpResponse::NotFound().body("User not found"),
        Err(e) => {
            eprintln!("Database error: {:?}", e);
            HttpResponse::InternalServerError().finish()
        }
    }
}

pub async fn get_user_interest_by_id(
    db: web::Data<PgPool>,
    user_id: web::Path<Uuid>,
) -> HttpResponse {
    let result = sqlx::query!(
        "SELECT interests FROM users WHERE id = $1",
        *user_id
    )
    .fetch_optional(db.get_ref())
    .await;

    match result {
        Ok(Some(record)) => {
            // interests column could be Option<String> or Option<serde_json::Value> depending on your schema
            // If interests is stored as JSONB array of strings in DB, you'd deserialize it here
            HttpResponse::Ok().json(record.interests)
        }
        Ok(None) => HttpResponse::NotFound().json("User not found"),
        Err(e) => {
            eprintln!("Database error: {:?}", e);
            HttpResponse::InternalServerError().json("Error retrieving user interests")
        }
    }
}

// Or /api/users/{id} if you pass ID in JWT
pub async fn get_user_by_email(
    pool: web::Data<Pool<Postgres>>,
    path_email: web::Path<String>,
) -> impl Responder {
    let email = path_email.into_inner();
     let email_lower = email.to_lowercase();
    match sqlx::query_as!(
        User,
        r#"
        SELECT
            id,
            first_name,
            last_name,
            username,
            password,
            mobile,
            email,
            organisation,
            bio,
            profile_pic,
            created_at,
            updated_at,
            is_active,
            resume_url,
            interests,
            provider,
            provider_user_id,
            access_token,
            refresh_token,
            oauth_email,
            college,
            batch,
            stream,
            linkedin,
            github,
            skills,
            work_experiences,
            projects,
            pronouns,
            location,
            hackingon,
            learning,
            availablefor
        FROM users
        WHERE email = $1
        "#,
        email_lower
    )
    .fetch_optional(pool.get_ref())
    .await
    {
        Ok(Some(user)) => HttpResponse::Ok().json(user),
        Ok(None) => HttpResponse::NotFound().body("User not found"),
        Err(e) => {
            eprintln!("Database error: {:?}", e); // This is where the error will be very clear
            HttpResponse::InternalServerError().body("Database error")
        }
    }
}