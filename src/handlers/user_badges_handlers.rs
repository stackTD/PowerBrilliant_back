use actix_web::{web, HttpResponse, HttpRequest};
use uuid::Uuid;
use sqlx::PgPool;
use crate::models::user_badges::{User_Badges, Create_User_Badge};
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

pub async fn create_user_badge(
    req: HttpRequest,
    db: web::Data<PgPool>,
    json: web::Json<Create_User_Badge>,
) -> HttpResponse {
    let lang = get_lang(&req);            
    println!("Language preference: {}", lang);
    let user = sqlx::query_as::<_, User_Badges>(
        r#"
        INSERT INTO user_badges (id, user_id, badge_name, badge_icon)
        VALUES ($1, $2, $3, $4)
        RETURNING *
        "#
    )
    .bind(Uuid::new_v4())
    .bind(&json.user_id)
    .bind(&json.badge_name)
    .bind(&json.badge_icon)


    .fetch_one(db.get_ref())
    .await;

    match user {
        Ok(c) => HttpResponse::Ok().json(c),
        Err(e) => {
            eprintln!("DB error: {:?}", e);
            HttpResponse::InternalServerError().json(json!({
                "status": "error",
                "message": "Error creating user badge"
            }))
        }
    }
}

pub async fn get_user_badges(db: web::Data<PgPool>) -> HttpResponse {
    let users = sqlx::query_as::<_, User_Badges>(
        "SELECT * FROM user_badges ORDER BY awarded_at DESC"
    )
    .fetch_all(db.get_ref())
    .await;

    match users {
        Ok(users) => HttpResponse::Ok().json(users),
        Err(e) => {
            eprintln!("Database error: {:?}", e);
            HttpResponse::InternalServerError().body("Error retrieving badges")
        }
    }
}

pub async fn delete_user_badge_by_id(
    db: web::Data<PgPool>,
    id: web::Path<Uuid>,  // Changed from email to UUID
) -> HttpResponse {
    let result = sqlx::query!(
        "DELETE FROM user_badges WHERE id = $1 RETURNING id",
        id.into_inner()
    )
    .fetch_optional(db.get_ref())
    .await;

    match result {
        Ok(Some(_)) => HttpResponse::Ok().json(json!({
            "status": "success",
            "message": "badge deleted successfully"
        })),
        Ok(None) => HttpResponse::NotFound().json(json!({
            "status": "error",
            "message": "badge not found"
        })),
        Err(e) => {
            eprintln!("Database error: {:?}", e);
            HttpResponse::InternalServerError().json(json!({
                "status": "error",
                "message": "Error deleting badge"
            }))
        }
    }
}
