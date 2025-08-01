use actix_web::{web, HttpResponse, HttpRequest};
use uuid::Uuid;
use sqlx::PgPool;
use crate::models::community_members::{Community_Members, Create_Community_Member, Update_Community_Member};
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

pub async fn create_community_member(
    req: HttpRequest,
    db: web::Data<PgPool>,
    json: web::Json<Create_Community_Member>,
) -> HttpResponse {
    let lang = get_lang(&req);            
    println!("Language preference: {}", lang);
    let user = sqlx::query_as::<_, Community_Members>(
        r#"
        INSERT INTO community_members (id, community_id, user_id, role)
        VALUES ($1, $2, $3, $4)
        RETURNING *
        "#
    )
    .bind(Uuid::new_v4())
    .bind(&json.community_id)
    .bind(&json.user_id)
    // .bind(Uuid::new_v4()) 
    .bind(&json.role)
    .fetch_one(db.get_ref())
    .await;

    match user {
        Ok(c) => HttpResponse::Ok().json(c),
        Err(e) => {
            eprintln!("DB error: {:?}", e);
            HttpResponse::InternalServerError().json(json!({
                "status": "error",
                "message": "Error adding member in community"
            }))
        }
    }
}

pub async fn get_community_members(db: web::Data<PgPool>) -> HttpResponse {
    let users = sqlx::query_as::<_, Community_Members>(
        "SELECT * FROM community_members"
    )
    .fetch_all(db.get_ref())
    .await;

    match users {
        Ok(users) => HttpResponse::Ok().json(users),
        Err(e) => {
            eprintln!("Database error: {:?}", e);
            HttpResponse::InternalServerError().body("Error retrieving community members")
        }
    }
}

pub async fn delete_community_member_by_id(
    db: web::Data<PgPool>,
    id: web::Path<Uuid>,  // Changed from email to UUID
) -> HttpResponse {
    let result = sqlx::query!(
        "DELETE FROM community_members WHERE id = $1 RETURNING id",
        id.into_inner()
    )
    .fetch_optional(db.get_ref())
    .await;

    match result {
        Ok(Some(_)) => HttpResponse::Ok().json(json!({
            "status": "success",
            "message": "community member deleted successfully"
        })),
        Ok(None) => HttpResponse::NotFound().json(json!({
            "status": "error",
            "message": "community member not found"
        })),
        Err(e) => {
            eprintln!("Database error: {:?}", e);
            HttpResponse::InternalServerError().json(json!({
                "status": "error",
                "message": "Error deleting community member"
            }))
        }
    }
}

pub async fn update_community_member_by_id(
    db: web::Data<PgPool>,
    id: web::Path<Uuid>,
    json: web::Json<Update_Community_Member>,
) -> HttpResponse {
    let query_result = sqlx::query!(
        r#"
        UPDATE community_members
        SET 
            role = COALESCE($1, role)
        WHERE id = $2
        RETURNING id
        "#,
        json.role.as_deref(),
        id.into_inner(),
    )
    .fetch_optional(db.get_ref())
    .await;

    match query_result {
        Ok(Some(_)) => HttpResponse::Ok().json(json!({
            "status": "success",
            "message": "Community member updated successfully"
        })),
        Ok(None) => HttpResponse::NotFound().json(json!({
            "status": "error",
            "message": "Community not found"
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
