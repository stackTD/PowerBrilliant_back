use actix_web::{ web, HttpResponse, HttpRequest, Responder };
use serde::Deserialize;
use uuid::Uuid;
use sqlx::{PgPool, postgres::PgDatabaseError};
use crate::models::followers::{Followers, Create_Follower, Update_Follower, FollowerListUser, FollowingListUser};
use serde_json::json;
use actix_web::http::StatusCode;


#[derive(Deserialize)]
pub struct FollowPayload {
    pub follower_id: Uuid,
    
    pub followed_id: Uuid,
}

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

pub async fn create_follower(
    req: HttpRequest,
    db: web::Data<PgPool>,
    json: web::Json<Create_Follower>,
) -> HttpResponse {
    let lang = get_lang(&req);            
    println!("Language preference: {}", lang);
    let user = sqlx::query_as::<_, Followers>(
        r#"
        INSERT INTO followers (id, follower_id, followed_id, is_following)
        VALUES ($1, $2, $3, $4)
        RETURNING *
        "#
    )
    .bind(Uuid::new_v4())
    .bind(&json.follower_id)
    .bind(&json.followed_id)    
    .bind(&json.is_following)
    .fetch_one(db.get_ref())
    .await;

    match user {
        Ok(c) => HttpResponse::Ok().json(c),
        Err(e) => {
            eprintln!("DB error: {:?}", e);
            HttpResponse::InternalServerError().json(json!({
                "status": "error",
                "message": "Error in adding follower "
            }))
        }
    }
}

pub async fn view_followers(db: web::Data<PgPool>) -> HttpResponse {
    let users = sqlx::query_as::<_, Followers>(
        "SELECT * FROM followers"
    )
    .fetch_all(db.get_ref())
    .await;

    match users {
        Ok(users) => HttpResponse::Ok().json(users),
        Err(e) => {
            eprintln!("Database error: {:?}", e);
            HttpResponse::InternalServerError().body("Error retrieving followers")
        }
    }
}

pub async fn delete_follower_by_id(
    db: web::Data<PgPool>,
    id: web::Path<Uuid>,  // Changed from email to UUID
) -> HttpResponse {
    let result = sqlx::query!(
        "DELETE FROM followers WHERE id = $1 RETURNING id",
        id.into_inner()
    )
    .fetch_optional(db.get_ref())
    .await;

    match result {
        Ok(Some(_)) => HttpResponse::Ok().json(json!({
            "status": "success",
            "message": "follower removed successfully"
        })),
        Ok(None) => HttpResponse::NotFound().json(json!({
            "status": "error",
            "message": "follower not found"
        })),
        Err(e) => {
            eprintln!("Database error: {:?}", e);
            HttpResponse::InternalServerError().json(json!({
                "status": "error",
                "message": "Error removing follower"
            }))
        }
    }
}

pub async fn update_follower_by_id(
    db: web::Data<PgPool>,
    id: web::Path<Uuid>,
    json: web::Json<Update_Follower>,
) -> HttpResponse {
    let query_result = sqlx::query!(
        r#"
        UPDATE followers
        SET 
            is_following = COALESCE($1, is_following)
        WHERE id = $2
        RETURNING id
        "#,
        json.is_following,
        id.into_inner(),
    )
    .fetch_optional(db.get_ref())
    .await;

    match query_result {
        Ok(Some(_)) => HttpResponse::Ok().json(json!({
            "status": "success",
            "message": "following status updated successfully"
        })),
        Ok(None) => HttpResponse::NotFound().json(json!({
            "status": "error",
            "message": "follower not found"
        })),
        Err(e) => {
            eprintln!("Database error: {:?}", e);
            HttpResponse::InternalServerError().json(json!({
                "status": "error",
                "message": "Error updating follower"
            }))
        }
    }
}

pub async fn toggle_follow(
    payload: web::Json<FollowPayload>,
    db: web::Data<PgPool>,
) -> impl Responder {
    let FollowPayload { follower_id, followed_id } = payload.into_inner();

    // Check if follower is a valid user
    let is_valid_follower = sqlx::query_scalar!(
        "SELECT EXISTS(SELECT 1 FROM users WHERE id = $1)",
        follower_id
    )
    .fetch_one(db.get_ref())
    .await
    .unwrap_or(None)
    .unwrap_or(false);

    if !is_valid_follower {
        return HttpResponse::BadRequest().json(json!({
            "status": "error",
            "message": "Only normal users can follow others"
        }));
    }

    // Check if follow exists
    let exists = sqlx::query!(
        "SELECT id FROM followers WHERE follower_id = $1 AND followed_id = $2",
        follower_id,
        followed_id
    )
    .fetch_optional(db.get_ref())
    .await;

    match exists {
        Ok(Some(row)) => {
            // Unfollow (delete)
            if let Err(e) = sqlx::query!(
                "DELETE FROM followers WHERE id = $1",
                row.id
            )
            .execute(db.get_ref())
            .await {
                return handle_db_error(e);
            }

            HttpResponse::Ok().json("unfollowed")
        }
        Ok(None) => {
            // Follow (insert)
            let result = sqlx::query!(
                r#"INSERT INTO followers (id, follower_id, followed_id, is_following)
                VALUES ($1, $2, $3, $4)"#,
                Uuid::new_v4(),
                follower_id,
                followed_id,
                true
            )
            .execute(db.get_ref())
            .await;

            match result {
                Ok(_) => HttpResponse::Ok().json("followed"),
                Err(e) => handle_db_error(e)
            }
        }
        Err(e) => handle_db_error(e)
    }
}

fn handle_db_error(e: sqlx::Error) -> HttpResponse {
    eprintln!("Database error: {:?}", e);
    
    if let Some(db_err) = e.as_database_error() {
        if let Some(constraint) = db_err.constraint() {
            if constraint == "followers_follower_id_fkey" {
                return HttpResponse::BadRequest().json(json!({
                    "status": "error",
                    "message": "Invalid follower ID. Only user accounts can follow others."
                }));
            }
            if constraint == "followers_followed_id_fkey" {
                return HttpResponse::BadRequest().json(json!({
                    "status": "error",
                    "message": "Invalid user to follow."
                }));
            }
        }
    }

    HttpResponse::InternalServerError().json(json!({
        "status": "error",
        "message": "Database operation failed"
    }))
}

pub async fn follow_status(
    path: web::Path<(Uuid, Uuid)>,
    db: web::Data<PgPool>,
) -> impl Responder {
    let (follower_id, followed_id) = path.into_inner();

    let result = sqlx::query_scalar!(
        "SELECT EXISTS(SELECT 1 FROM followers WHERE follower_id = $1 AND followed_id = $2)",
        follower_id,
        followed_id
    )
    .fetch_one(db.get_ref())
    .await;

    match result {
        Ok(exists) => HttpResponse::Ok().json(json!({ "is_following": exists })),
        Err(_) => HttpResponse::InternalServerError().json(json!({ "error": "Failed to check follow status" })),
    }
}

pub async fn get_followers_count(
    db: web::Data<PgPool>,
    user_id: web::Path<Uuid>
) -> impl Responder {
    let count = sqlx::query_scalar!(
        "SELECT COUNT(*) FROM followers WHERE followed_id = $1",
        user_id.into_inner()
    )
    .fetch_one(db.get_ref())
    .await;

    match count {
        Ok(count) => HttpResponse::Ok().json(json!({ "followers": count })),
        Err(e) => {
            eprintln!("DB error: {:?}", e);
            HttpResponse::InternalServerError().finish()
        }
    }
}

pub async fn get_following_count(
    db: web::Data<PgPool>,
    user_id: web::Path<Uuid>
) -> impl Responder {
    let count = sqlx::query_scalar!(
        "SELECT COUNT(*) FROM followers WHERE follower_id = $1",
        user_id.into_inner()
    )
    .fetch_one(db.get_ref())
    .await;

    match count {
        Ok(count) => HttpResponse::Ok().json(json!({ "following": count })),
        Err(e) => {
            eprintln!("DB error: {:?}", e);
            HttpResponse::InternalServerError().finish()
        }
    }
}


pub async fn get_followers_list(db: web::Data<PgPool>, user_id: web::Path<Uuid>) -> impl Responder {
    let result = sqlx::query_as!(
        FollowerListUser,
        r#"
        SELECT u.id, u.first_name, u.last_name, u.username, u.profile_pic
        FROM followers f
        JOIN users u ON f.follower_id = u.id
        WHERE f.followed_id = $1
        "#,
        user_id.into_inner()
    )
    .fetch_all(db.get_ref())
    .await;

    match result {
        Ok(rows) => HttpResponse::Ok().json(rows),
        Err(e) => {
            eprintln!("DB error: {:?}", e);
            HttpResponse::InternalServerError().finish()
        }
    }
}


pub async fn get_following_list(db: web::Data<PgPool>, user_id: web::Path<Uuid>) -> impl Responder {
    let result = sqlx::query_as!(
        FollowingListUser,
        r#"
        SELECT u.id, u.first_name, u.last_name, u.username, u.profile_pic
        FROM followers f
        JOIN users u ON f.followed_id = u.id
        WHERE f.follower_id = $1
        "#,
        user_id.into_inner()
    )
    .fetch_all(db.get_ref())
    .await;

    match result {
        Ok(rows) => HttpResponse::Ok().json(rows),
        Err(e) => {
            eprintln!("DB error: {:?}", e);
            HttpResponse::InternalServerError().finish()
        }
    }
}
