use actix_web::{web, HttpResponse, HttpRequest, Responder};
use actix_web::{web::BytesMut, Error};
use futures::stream::StreamExt;
use std::io::Write;
use std::path::Path;
use chrono::Utc;
use uuid::Uuid;
use sqlx::PgPool;
use crate::models::posts::{Post, CreatePost, UpdatePost, PostWithRank, Pagination, MediaType};
use serde_json::json;
use serde::{Serialize, Deserialize};
use chrono::NaiveDateTime;
use infer;

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

pub async fn create_post(
    req: HttpRequest,
    db: web::Data<PgPool>,
    json: web::Json<CreatePost>,
) -> HttpResponse {
    let lang = get_lang(&req);            
    println!("Language preference: {}", lang);
    let post_result = sqlx::query_as::<_, Post>(
         r#"
        INSERT INTO posts (id, community_id, author_id, title, content, post_type, is_active, tags, likes)
        VALUES ($1, $2, $3, $4, $5, $6, COALESCE($7, true), $8, $9)
        RETURNING *
        "#
    )
    .bind(Uuid::new_v4())
    .bind(&json.community_id)
    .bind(&json.author_id)
    .bind(&json.title)
    .bind(&json.content)
    .bind(&json.post_type)
    .bind(&json.is_active)
    .bind(&json.tags)
     .bind(json.likes.unwrap_or(0))
    .fetch_one(db.get_ref())
    .await;

    match post_result {
        Ok(post) => {
            // Save media if exists
            if let Some(media) = &json.media {
                for media_item in media {
                    sqlx::query!(
                        r#"
                        INSERT INTO post_media (id, post_id, media_type, url)
                        VALUES ($1, $2, $3, $4)
                        "#,
                        Uuid::new_v4(),
                        post.id,
                        media_item.media_type as MediaType,
                        &media_item.url
                    )
                    .execute(db.get_ref())
                    .await
                    .map_err(|e| {
                        eprintln!("Failed to save media: {:?}", e);
                    }).ok();
                }
            }
            
            HttpResponse::Ok().json(post)
        },
        Err(e) => {
            eprintln!("DB error: {:?}", e);
            HttpResponse::InternalServerError().json(json!({
                "status": "error",
                "message": "Error creating post"
            }))
        }
    }
}

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct PostWithLikeCount {
    pub id: Uuid,
    pub community_id: Uuid,
    pub author_id: Uuid,
    pub title: serde_json::Value,
    pub content: serde_json::Value,
    pub post_type: String,
    pub created_at: NaiveDateTime,
    pub is_active: bool,
    pub tags: Vec<String>,
    pub likes: i32,
    pub like_count: i64,
    pub media_urls: Vec<String>,  // Changed to i64
}

pub async fn get_posts(
    db: web::Data<PgPool>,
    web::Query(pagination): web::Query<Pagination>,
) -> HttpResponse {
    let limit = pagination.limit.unwrap_or(5);
    let offset = pagination.offset.unwrap_or(0);
    let sort = pagination.sort.as_deref().unwrap_or("latest");

    let base_query = r#"
        SELECT 
              p.id,
            p.community_id,
            p.author_id,
            p.title,
            p.content,
            p.post_type,
            p.created_at,
            p.is_active,
            p.tags,
            p.likes,
            (SELECT COUNT(*) FROM post_likes pl 
             WHERE pl.post_id = p.id AND pl.is_liked = true) as like_count,
            ARRAY(
                SELECT pm.url 
                FROM post_media pm 
                WHERE pm.post_id = p.id 
                ORDER BY pm.created_at
            ) as media_urls
        FROM posts p
    "#;

    let order_clause = match sort {
        "top" => "ORDER BY like_count DESC, p.created_at DESC",
        "latest" => "ORDER BY p.created_at DESC",
        _ => "ORDER BY p.created_at DESC",
    };

    let query_str = format!("{} {}", base_query, order_clause);
    let full_query = format!("{} LIMIT $1 OFFSET $2", query_str);

    let posts = sqlx::query_as::<_, PostWithLikeCount>(&full_query)
        .bind(limit)
        .bind(offset)
        .fetch_all(db.get_ref())
        .await;

    match posts {
        Ok(posts) => HttpResponse::Ok().json(posts),
        Err(e) => {
            eprintln!("Database error: {:?}", e);
            HttpResponse::InternalServerError().body("Error retrieving posts")
        }
    }
}

pub async fn delete_post_by_id(
    db: web::Data<PgPool>,
    id: web::Path<Uuid>,
) -> HttpResponse {
    let post_id = id.into_inner();

    // First, delete associated media
    let media_delete_result = sqlx::query!(
        "DELETE FROM post_media WHERE post_id = $1",
        post_id
    )
    .execute(db.get_ref())
    .await;

    if let Err(e) = media_delete_result {
        eprintln!("Database error deleting media: {:?}", e);
        return HttpResponse::InternalServerError().json(json!({
            "status": "error",
            "message": "Error deleting post media"
        }));
    }

    // Next, delete from shared_posts
    let shared_delete_result = sqlx::query!(
        "DELETE FROM shared_posts WHERE post_id = $1",
        post_id
    )
    .execute(db.get_ref())
    .await;

    if let Err(e) = shared_delete_result {
        eprintln!("Database error deleting shared_posts: {:?}", e);
        return HttpResponse::InternalServerError().json(json!({
            "status": "error",
            "message": "Error deleting shared posts"
        }));
    }

    // Then, delete the post itself
    let result = sqlx::query!(
        "DELETE FROM posts WHERE id = $1 RETURNING id",
        post_id
    )
    .fetch_optional(db.get_ref())
    .await;

    match result {
        Ok(Some(_)) => HttpResponse::Ok().json(json!({
            "status": "success",
            "message": "post deleted successfully"
        })),
        Ok(None) => HttpResponse::NotFound().json(json!({
            "status": "error",
            "message": "post not found"
        })),
        Err(e) => {
            eprintln!("Database error: {:?}", e);
            HttpResponse::InternalServerError().json(json!({
                "status": "error",
                "message": "Error deleting post"
            }))
        }
    }
}

pub async fn update_post_by_id(
    db: web::Data<PgPool>,
    id: web::Path<Uuid>,
    json: web::Json<UpdatePost>,
) -> HttpResponse {
    let query_result = sqlx::query!(
          r#"
        UPDATE posts 
        SET 
            title = COALESCE($1, title),
            content = COALESCE($2, content),
            post_type = COALESCE($3, post_type),
            is_active = COALESCE($4, is_active),
            tags = COALESCE($5, tags),
            likes = COALESCE($6, likes)
        WHERE id = $7
        RETURNING id
        "#,
        json.title,
        json.content,
        json.post_type.as_deref(),
        json.is_active.unwrap_or(true),
        json.tags.as_deref(),
        json.likes,
        id.into_inner(),
    )
    .fetch_optional(db.get_ref())
    .await;

    match query_result {
        Ok(Some(_)) => HttpResponse::Ok().json(json!({
            "status": "success",
            "message": "POST updated successfully"
        })),
        Ok(None) => HttpResponse::NotFound().json(json!({
            "status": "error",
            "message": "post not found"
        })),
        Err(e) => {
            eprintln!("Database error: {:?}", e);
            HttpResponse::InternalServerError().json(json!({
                "status": "error",
                "message": "Error updating post"
            }))
        }
    }
}

pub async fn get_post_by_id(
    db: web::Data<PgPool>,
    id: web::Path<Uuid>,
) -> HttpResponse {
    let post = sqlx::query_as::<_, PostWithLikeCount>(
         r#"
        SELECT 
            p.id,
            p.community_id,
            p.author_id,
            p.title,
            p.content,
            p.post_type,
            p.created_at,
            p.is_active,
            p.tags,
            p.likes,
            (SELECT COUNT(*) FROM post_likes pl 
             WHERE pl.post_id = p.id AND pl.is_liked = true) as like_count,
            ARRAY(
                SELECT pm.url 
                FROM post_media pm 
                WHERE pm.post_id = p.id 
                ORDER BY pm.created_at
            ) as media_urls
        FROM posts p
        WHERE p.id = $1
        "#
    )
    .bind(id.into_inner())
    .fetch_optional(db.get_ref())
    .await;

    match post {
        Ok(Some(post)) => HttpResponse::Ok().json(post),
        Ok(None) => HttpResponse::NotFound().json(json!({
            "status": "error",
            "message": "Post not found"
        })),
        Err(e) => {
            eprintln!("Database error: {:?}", e);
            HttpResponse::InternalServerError().json(json!({
                "status": "error",
                "message": "Error fetching post"
            }))
        }
    }
}

pub async fn get_posts_by_author_id(
    db: web::Data<PgPool>,
    author_id: web::Path<Uuid>,
) -> HttpResponse {
    let posts = sqlx::query_as::<_, PostWithLikeCount>(
        r#"
        SELECT 
            p.id,
            p.community_id,
            p.author_id,
            p.title,
            p.content,
            p.post_type,
            p.created_at,
            p.is_active,
            p.tags,
            p.likes,
            (SELECT COUNT(*) FROM post_likes pl 
             WHERE pl.post_id = p.id AND pl.is_liked = true) as like_count,
            ARRAY(
                SELECT pm.url 
                FROM post_media pm 
                WHERE pm.post_id = p.id 
                ORDER BY pm.created_at
            ) as media_urls
        FROM posts p
        WHERE author_id = $1 
        ORDER BY created_at DESC
        "#
    )
    .bind(author_id.into_inner())
    .fetch_all(db.get_ref())
    .await;

    match posts {
        Ok(posts) => {
            if posts.is_empty() {
                HttpResponse::NotFound().json(json!({
                    "status": "error",
                    "message": "No posts found for this author"
                }))
            } else {
                HttpResponse::Ok().json(posts)
            }
        },
        Err(e) => {
            eprintln!("Database error: {:?}", e);
            HttpResponse::InternalServerError().json(json!({
                "status": "error",
                "message": "Error retrieving posts"
            }))
        }
    }
}

pub async fn get_tags_by_post_id(
    db: web::Data<PgPool>,
) -> HttpResponse {
    let result = sqlx::query!(
        "SELECT tags FROM posts"
    )
    .fetch_all(db.get_ref())
    .await;

    match result {
        Ok(records) => {
            let tags: Vec<serde_json::Value> = records
                .into_iter()
                .filter_map(|record| record.tags)
                .map(|tag_vec| json!(tag_vec))
                .collect();
            HttpResponse::Ok().json(json!({ "tags": tags }))
        },
        Err(e) => {
            eprintln!("Database error: {:?}", e);
            HttpResponse::InternalServerError().json(json!({
                "status": "error",
                "message": "Error retrieving tags"
            }))
        }
    }
}

pub async fn get_posts_by_user_interest(
    pool: web::Data<PgPool>,
    user_id: web::Path<Uuid>,
    web::Query(mut params): web::Query<HashMap<String, String>>,
) -> impl Responder {
    let sort_by = params.remove("sort").unwrap_or_else(|| "relevant".to_string());
    
    let order_clause = match sort_by.as_str() {
        "latest" => "ORDER BY p.created_at DESC",
        "top" => "ORDER BY like_count DESC, p.created_at DESC",
        _ => "ORDER BY match_count DESC, p.created_at DESC",
    };

    let interests_result = sqlx::query_scalar!(
        "SELECT interests FROM users WHERE id = $1",
        *user_id
    )
    .fetch_one(pool.get_ref())
    .await;

    let interests: Vec<String> = match interests_result {
        Ok(Some(interests)) => interests,
        Ok(None) | Err(_) => vec![],
    };

    let limit = params.get("limit")
        .and_then(|l| l.parse::<i64>().ok())
        .unwrap_or(5);
        
    let offset = params.get("offset")
        .and_then(|o| o.parse::<i64>().ok())
        .unwrap_or(0);

    let query_str = format!(
        
      r#"
    SELECT 
        p.id,
        p.community_id,
        p.author_id,
        p.title,
        p.content,
        p.post_type,
        p.created_at,
        p.is_active,
        p.tags,
        p.likes,
        cardinality(ARRAY(
            SELECT UNNEST(p.tags) INTERSECT SELECT UNNEST($1::text[])
        )) as match_count,
        (SELECT COUNT(*) FROM post_likes pl 
         WHERE pl.post_id = p.id AND pl.is_liked = true) as like_count,
        ARRAY(
            SELECT pm.url 
            FROM post_media pm 
            WHERE pm.post_id = p.id 
            ORDER BY pm.created_at
        ) as media_urls 
    FROM posts p
    {}
    LIMIT $2 OFFSET $3
    "#,
    order_clause
);

    let posts_result = sqlx::query_as::<_, PostWithRank>(&query_str)
        .bind(&interests)
        .bind(limit)
        .bind(offset)
        .fetch_all(pool.get_ref())
        .await;

    match posts_result {
        Ok(posts) => HttpResponse::Ok().json(posts),
        Err(e) => {
            eprintln!("❌ Error fetching posts: {}", e);
            HttpResponse::InternalServerError().body("Failed to fetch posts")
        }
    }
}

#[derive(Serialize)]
pub struct UploadResponse {
    pub url: String,
}

pub async fn upload_media(
    mut payload: actix_web::web::Payload,
) -> Result<HttpResponse, Error> {
    // Create uploads directory if it doesn't exist
    let upload_dir = Path::new("uploads");
    if !upload_dir.exists() {
        std::fs::create_dir_all(upload_dir)?;
    }

    let mut bytes = BytesMut::new();
    while let Some(chunk) = payload.next().await {
        let chunk = chunk?;
        bytes.extend_from_slice(&chunk);
    }

    // Get the file type using infer
    let kind = match infer::get(&bytes) {
        Some(k) => k,
        None => return Err(actix_web::error::ErrorBadRequest("Unrecognized file type")),
    };

    // Define allowed MIME types
    let allowed_mime_types = vec![
        "image/jpeg", "image/png", "image/gif", "image/webp", "image/bmp", "image/tiff", 
        "image/svg+xml", "image/vnd.microsoft.icon", "image/x-icon", "image/heic", "image/heif",
        "video/mp4", "video/webm", "video/quicktime", "video/x-msvideo", "video/mpeg",
        "video/ogg", "video/mp2t", "video/x-matroska", "video/3gpp", "video/3gpp2"
    ];

    if !allowed_mime_types.contains(&kind.mime_type()) {
        return Err(actix_web::error::ErrorBadRequest("Unsupported file type"));
    }

    let extension = kind.extension();

    // Generate unique filename with extension
    let timestamp = Utc::now().format("%Y%m%d%H%M%S");
    let filename = format!("{}_{}.{}", timestamp, Uuid::new_v4(), extension);
    let file_path = upload_dir.join(&filename);

    // Save file
    let mut file = std::fs::File::create(&file_path)?;
    file.write_all(&bytes)?;

    let media_url = format!("/uploads/{}", filename);

    Ok(HttpResponse::Ok().json(UploadResponse { url: media_url }))
}
