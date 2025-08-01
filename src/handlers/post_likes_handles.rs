use actix_web::{web, HttpResponse, HttpRequest, Responder};
use serde::Deserialize;
use uuid::Uuid;
use sqlx::PgPool;
use crate::models::post_likes::{Post_Likes, Create_Post_Likes, Update_Post_Likes};
use serde_json::json;
use chrono::NaiveDateTime;
use serde::Serialize;
use sqlx::Row; // Add this at the top if not present

// Removed duplicate LikePayload definition to resolve conflicting implementations

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

pub async fn create_post_like(
    req: HttpRequest,
    db: web::Data<PgPool>,
    json: web::Json<Create_Post_Likes>,
) -> HttpResponse {
    let lang = get_lang(&req);            
    println!("Language preference: {}", lang);
    let user = sqlx::query_as::<_, Post_Likes>(
        r#"
        INSERT INTO post_likes (id, post_id, user_id, is_liked)
        VALUES ($1, $2, $3, COALESCE($4, true))
        RETURNING *
        "#
    )
    .bind(Uuid::new_v4())
    .bind(&json.post_id)
    .bind(&json.user_id)
    .bind(&json.is_liked)

    .fetch_one(db.get_ref())
    .await;

    match user {
        Ok(c) => HttpResponse::Ok().json(c),
        Err(e) => {
            eprintln!("DB error: {:?}", e);
            HttpResponse::InternalServerError().json(json!({
                "status": "error",
                "message": "Error in liking"
            }))
        }
    }
}
pub async fn view_likes(db: web::Data<PgPool>) -> HttpResponse {
    let users = sqlx::query_as::<_, Post_Likes>(
        "SELECT * FROM post_likes"
    )
    .fetch_all(db.get_ref())
    .await;

    match users {
        Ok(users) => HttpResponse::Ok().json(users),
        Err(e) => {
            eprintln!("Database error: {:?}", e);
            HttpResponse::InternalServerError().body("Error retrieving like")
        }
    }
}
pub async fn update_post_likes_by_id(
    db: web::Data<PgPool>,
    id: web::Path<Uuid>,
    json: web::Json<Update_Post_Likes>,
) -> HttpResponse {
    let query_result = sqlx::query!(
        r#"
        UPDATE post_likes
        SET 
            is_liked = COALESCE($1, is_liked)
        WHERE id = $2
        RETURNING id
        "#,
        json.is_liked,
        id.into_inner(),
    )
    .fetch_optional(db.get_ref())
    .await;

    match query_result {
        Ok(Some(_)) => HttpResponse::Ok().json(json!({
            "status": "success",
            "message": "like updated successfully"
        })),
        Ok(None) => HttpResponse::NotFound().json(json!({
            "status": "error",
            "message": "like was not found"
        })),
        Err(e) => {
            eprintln!("Database error: {:?}", e);
            HttpResponse::InternalServerError().json(json!({
                "status": "error",
                "message": "Error updating post like"
            }))
        }
    }
}

#[derive(Serialize, sqlx::FromRow)]
pub struct PostLikeInfo {
    pub user_id: Uuid,
    pub name: String,
    pub user_type: String,
    pub liked_at: NaiveDateTime,
}


pub async fn get_post_likes(
    pool: web::Data<PgPool>,
    post_id: web::Path<Uuid>,
) -> impl Responder {
    let post_id = post_id.into_inner();

    let query = r#"
        SELECT 
            pl.user_id,
            CONCAT(u.first_name, ' ', u.last_name) AS name,
            'user' AS user_type,
            pl.liked_at
        FROM post_likes pl
        JOIN users u ON pl.user_id = u.id
        WHERE pl.post_id = $1 AND pl.user_type = 'user'

        UNION

        SELECT 
            pl.user_id,
            ba.company_name AS name,
            'business' AS user_type,
            pl.liked_at
        FROM post_likes pl
        JOIN business_accounts ba ON pl.user_id = ba.id
        WHERE pl.post_id = $1 AND pl.user_type = 'business'
    "#;

    match sqlx::query_as::<_, PostLikeInfo>(query)
        .bind(post_id)
        .fetch_all(pool.get_ref())
        .await
    {
        Ok(likes) => HttpResponse::Ok().json(likes),
        Err(err) => {
            eprintln!("DB error: {:?}", err);
            HttpResponse::InternalServerError().body("Failed to fetch likes")
        }
    }
}

#[derive(Deserialize)]
pub struct LikePayload {
    pub post_id: Uuid,
    pub user_id: Option<Uuid>,
    pub business_id: Option<Uuid>,
}

#[derive(Serialize, sqlx::FromRow)]
pub struct LikedPostId {
    pub post_id: Uuid,
}
#[derive(Deserialize)]
pub struct LikeQuery {
    user_id: Option<Uuid>,
    business_id: Option<Uuid>,
}


pub async fn get_liked_posts_by_user(
    db: web::Data<PgPool>,
    query: web::Query<LikeQuery>,
) -> impl Responder {
    if let Some(uid) = query.user_id {
        let result = sqlx::query_as::<_, LikedPostId>(
            r#"
            SELECT post_id FROM post_likes
            WHERE user_id = $1 AND is_liked = true
            "#
        )
        .bind(uid)  // ✅ bind only if it's a concrete Uuid
        .fetch_all(db.get_ref())
        .await;

        return match result {
            Ok(posts) => HttpResponse::Ok().json(posts),
            Err(e) => {
                eprintln!("DB error (user): {:?}", e);
                HttpResponse::InternalServerError().body("Error fetching liked posts (user)")
            }
        };
    } else if let Some(bid) = query.business_id {
        let result = sqlx::query_as::<_, LikedPostId>(
            r#"
            SELECT post_id FROM post_likes
            WHERE business_id = $1 AND is_liked = true
            "#
        )
        .bind(bid)
        .fetch_all(db.get_ref())
        .await;

        return match result {
            Ok(posts) => HttpResponse::Ok().json(posts),
            Err(e) => {
                eprintln!("DB error (business): {:?}", e);
                HttpResponse::InternalServerError().body("Error fetching liked posts (business)")
            }
        };
    }

    HttpResponse::BadRequest().body("Missing user_id or business_id")
}

  


pub async fn toggle_like_post(
    payload: web::Json<LikePayload>,
    db: web::Data<PgPool>,
) -> impl Responder {
    let LikePayload {
        post_id,
        user_id,
        business_id,
    } = payload.into_inner();

    if user_id.is_none() && business_id.is_none() {
        return HttpResponse::BadRequest().body("Missing user or business ID");
    }

    // Build SQL and binding based on which ID is present
    let (check_sql, id_value) = if let Some(uid) = user_id {
        (
            "SELECT id, is_liked FROM post_likes WHERE post_id = $1 AND user_id = $2",
            uid,
        )
    } else if let Some(bid) = business_id {
        (
            "SELECT id, is_liked FROM post_likes WHERE post_id = $1 AND business_id = $2",
            bid,
        )
    } else {
        unreachable!()
    };

    // Check if like exists
    let existing = sqlx::query(check_sql)
        .bind(post_id)
        .bind(id_value)
        .fetch_optional(db.get_ref())
        .await;

    match existing {
        Ok(Some(row)) => {
            let is_liked: bool = row.try_get("is_liked").unwrap_or(false);
            let like_id: Uuid = row.try_get("id").unwrap();

            sqlx::query!(
                "UPDATE post_likes SET is_liked = $1, updated_at = now() WHERE id = $2",
                !is_liked,
                like_id
            )
            .execute(db.get_ref())
            .await
            .ok();
        }
        Ok(None) => {
            sqlx::query!(
                r#"
                INSERT INTO post_likes (id, post_id, user_id, business_id, is_liked)
                VALUES ($1, $2, $3, $4, true)
                "#,
                Uuid::new_v4(),
                post_id,
                user_id,
                business_id
            )
            .execute(db.get_ref())
            .await
            .ok();
        }
        Err(e) => {
            eprintln!("DB error: {:?}", e);
            return HttpResponse::InternalServerError().finish();
        }
    }

    // Count current likes for the post
    let count = sqlx::query_scalar!(
        "SELECT COUNT(*) FROM post_likes WHERE post_id = $1 AND is_liked = true",
        post_id
    )
    .fetch_one(db.get_ref())
    .await
    .unwrap_or(Some(0));

    HttpResponse::Ok().json(count.unwrap_or(0))
}


#[derive(Serialize)]
pub struct LikeCountResponse {
    pub post_id: Uuid,
    pub like_count: i64,
}

pub async fn get_like_count_for_post(
    db: web::Data<PgPool>,
    path: web::Path<Uuid>,
) -> impl Responder {
    let post_id = path.into_inner();

    let result = sqlx::query_scalar!(
        "SELECT COUNT(*) FROM post_likes WHERE post_id = $1 AND is_liked = true",
        post_id
    )
    .fetch_one(db.get_ref())
    .await;

    match result {
        Ok(count) => HttpResponse::Ok().json(LikeCountResponse {
            post_id,
            like_count: count.unwrap_or(0),
        }),
        Err(e) => {
            eprintln!("DB Error: {:?}", e);
            HttpResponse::InternalServerError().body("Error fetching like count")
        }
    }
}