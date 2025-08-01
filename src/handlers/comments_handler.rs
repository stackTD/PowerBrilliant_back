use actix_web::{web, HttpResponse, HttpRequest};
use serde::Serialize;
use serde::Deserialize;
use uuid::Uuid;
use sqlx::PgPool;
use crate::models::comments::{Comments, Create_Comment, Update_Comment};
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

pub async fn create_comment(
    req: HttpRequest,
    db: web::Data<PgPool>,
    json: web::Json<Create_Comment>,
) -> HttpResponse {
    let lang = get_lang(&req);            
    println!("Language preference: {}", lang);

    let result = sqlx::query_as::<_, Comments>(
        r#"
        INSERT INTO comments (id, post_id, author_id, author_type, content, parent_id, is_active)
        VALUES ($1, $2, $3, $4, $5, $6, COALESCE($7, true))
        RETURNING *
        "#
    )
    .bind(Uuid::new_v4())
    .bind(&json.post_id)
    .bind(&json.author_id)
    .bind(&json.author_type)
    .bind(&json.content)
    .bind(&json.parent_id)  // This handles nested replies
    .bind(&json.is_active)
    .fetch_one(db.get_ref())
    .await;

    match result {
        Ok(comment) => HttpResponse::Ok().json(comment),
        Err(e) => {
            eprintln!("DB error: {:?}", e);
            HttpResponse::InternalServerError().json(json!({
                "status": "error",
                "message": "Error creating comment"
            }))
        }
    }
}


pub async fn get_comments(db: web::Data<PgPool>) -> HttpResponse {
    let users = sqlx::query_as::<_, Comments>(
        "SELECT * FROM comments ORDER BY created_at DESC"
    )
    .fetch_all(db.get_ref())
    .await;

    match users {
        Ok(users) => HttpResponse::Ok().json(users),
        Err(e) => {
            eprintln!("Database error: {:?}", e);
            HttpResponse::InternalServerError().body("Error retrieving comments")
        }
    }
}
pub async fn delete_comment_by_id(
    db: web::Data<PgPool>,
    id: web::Path<Uuid>,  // Changed from email to UUID
) -> HttpResponse {
    let result = sqlx::query!(
        "DELETE FROM comments WHERE id = $1 RETURNING id",
        id.into_inner()
    )
    .fetch_optional(db.get_ref())
    .await;

    match result {
        Ok(Some(_)) => HttpResponse::Ok().json(json!({
            "status": "success",
            "message": "comment deleted successfully"
        })),
        Ok(None) => HttpResponse::NotFound().json(json!({
            "status": "error",
            "message": "comment not found"
        })),
        Err(e) => {
            eprintln!("Database error: {:?}", e);
            HttpResponse::InternalServerError().json(json!({
                "status": "error",
                "message": "Error deleting comment"
            }))
        }
    }
}

pub async fn update_comment_by_id(
    db: web::Data<PgPool>,
    id: web::Path<Uuid>,
    json: web::Json<Update_Comment>,
) -> HttpResponse {
    let query_result = sqlx::query!(
        r#"
        UPDATE comments 
        SET 
            content = COALESCE($1, content),
            parent_id = COALESCE($2, parent_id),
            is_active = COALESCE($3, is_active)

        WHERE id = $4
        RETURNING id
        "#,
        json.content,
        json.parent_id,
        json.is_active,

        id.into_inner(),
    )
    .fetch_optional(db.get_ref())
    .await;

    match query_result {
        Ok(Some(_)) => HttpResponse::Ok().json(json!({
            "status": "success",
            "message": "comment updated successfully"
        })),
        Ok(None) => HttpResponse::NotFound().json(json!({
            "status": "error",
            "message": "comment not found"
        })),
        Err(e) => {
            eprintln!("Database error: {:?}", e);
            HttpResponse::InternalServerError().json(json!({
                "status": "error",
                "message": "Error updating comment"
            }))
        }
    }
}

// Updated CommentWithAuthor struct
#[derive(Serialize, sqlx::FromRow, Clone)]
pub struct CommentWithAuthor {
    pub id: Uuid,
    pub post_id: Uuid,
    pub author_id: Uuid,
    pub author_type: String,
    pub content: serde_json::Value,
    pub parent_id: Option<Uuid>,
    pub created_at: chrono::NaiveDateTime,
    pub is_active: bool,
    pub author_name: String,
    pub author_username: Option<String>,
    pub author_profile: Option<String>,
    pub business_company_name: Option<String>,
    pub business_logo_url: Option<String>,
    pub likes: i64,
}

// Updated NestedComment struct
#[derive(Serialize, Clone)]
pub struct NestedComment {
    pub id: Uuid,
    pub post_id: Uuid,
    pub author_id: Uuid,
    pub author_type: String,
    pub content: serde_json::Value,
    pub parent_id: Option<Uuid>,
    pub created_at: chrono::NaiveDateTime,
    pub is_active: bool,
    pub author_name: String,
    pub author_username: Option<String>,
    pub author_profile: Option<String>,
    pub likes: i64,
    pub replies: Vec<NestedComment>,
}

pub async fn get_comments_by_post_id(
    db: web::Data<PgPool>,
    post_id: web::Path<Uuid>,
) -> HttpResponse {
    // Fetch all comments for the post
    let comments_result = sqlx::query_as::<_, CommentWithAuthor>(
        r#"
        SELECT 
            c.id,
            c.post_id,
            c.author_id,
            c.author_type,
            c.content,
            c.parent_id,
            c.created_at,
            c.is_active,
               COALESCE(NULLIF(u.first_name || ' ' || u.last_name, ''), b.company_name, 'Unknown') as author_name,
            u.username AS author_username,
            u.profile_pic AS author_profile,
            b.company_name AS business_company_name,
            b.logo_url AS business_logo_url,
            (SELECT COUNT(*) FROM comment_likes cl WHERE cl.comment_id = c.id) as likes
        FROM comments c
        LEFT JOIN users u ON c.author_id = u.id AND c.author_type = 'user'
        LEFT JOIN business_accounts b ON c.author_id = b.id AND c.author_type = 'business'
        WHERE c.post_id = $1
        "#,
    )
    .bind(post_id.into_inner())
    .fetch_all(db.get_ref())
    .await;

    let flat_comments = match comments_result {
        Ok(comments) => comments,
        Err(e) => {
            eprintln!("DB error: {:?}", e);
            return HttpResponse::InternalServerError().json(json!({
                "status": "error",
                "message": "Error fetching comments"
            }));
        }
    };

    // Create a map for quick comment lookup
    let mut comments_map: HashMap<Uuid, NestedComment> = HashMap::new();
    let mut nested_comments: Vec<NestedComment> = Vec::new();

    // First pass: Create NestedComment objects and store in map
    for comment in &flat_comments {
        let (author_name, author_username, author_profile) = if comment.author_type == "user" {
            (
                comment.author_name.clone(),
                comment.author_username.clone(),
                comment.author_profile.clone(),
            )
        } else {
            // For businesses, use company name as author name
            (
                comment.business_company_name.clone().unwrap_or_default(),
                None, // Businesses don't have usernames
                comment.business_logo_url.clone(),
            )
        };

        let nested = NestedComment {
            id: comment.id,
            post_id: comment.post_id,
            author_id: comment.author_id,
            author_type: comment.author_type.clone(),
            content: comment.content.clone(),
            parent_id: comment.parent_id,
            created_at: comment.created_at,
            is_active: comment.is_active,
            author_name,
            author_username,
            author_profile,
            likes: comment.likes,
            replies: Vec::new(),
        };
        comments_map.insert(comment.id, nested);
    }

    // Second pass: Build nested structure by processing in reverse order
    for comment in flat_comments.iter().rev() {
        if let Some(parent_id) = comment.parent_id {
            // Try to remove the child comment from the map
            if let Some(child) = comments_map.remove(&comment.id) {
                // Attempt to find the parent and attach the child
                if let Some(parent) = comments_map.get_mut(&parent_id) {
                    parent.replies.push(child);
                } else {
                    // If parent not found, put child back in map
                    comments_map.insert(comment.id, child);
                }
            }
        }
    }

    // Collect top-level comments (those without parent)
    for (_, comment) in comments_map {
        if comment.parent_id.is_none() {
            nested_comments.push(comment);
        }
    }

    // Sort top-level comments by date (newest first)
    nested_comments.sort_by(|a, b| b.created_at.cmp(&a.created_at));

    // Sort replies within each comment (oldest first)
    for comment in &mut nested_comments {
        comment.replies.sort_by(|a, b| a.created_at.cmp(&b.created_at));
    }

    HttpResponse::Ok().json(nested_comments)
}

#[derive(Deserialize)]
pub struct LikeCommentPayload {
    pub comment_id: Uuid,
    pub user_id: Uuid,
    pub user_type: String, // "user" or "business"
}

pub async fn like_comment(
    db: web::Data<PgPool>,
    json: web::Json<LikeCommentPayload>,
) -> HttpResponse {
    let payload = json.into_inner();
    
    // Check if like exists
    let existing_like = sqlx::query!(
        r#"
        SELECT id FROM comment_likes 
        WHERE comment_id = $1 AND user_id = $2 AND user_type = $3
        "#,
        payload.comment_id,
        payload.user_id,
        payload.user_type
    )
    .fetch_optional(db.get_ref())
    .await;

    match existing_like {
        Ok(Some(_)) => {
            // Unlike
            let _ = sqlx::query!(
                r#"DELETE FROM comment_likes 
                WHERE comment_id = $1 AND user_id = $2 AND user_type = $3"#,
                payload.comment_id,
                payload.user_id,
                payload.user_type
            )
            .execute(db.get_ref())
            .await;
        }
        Ok(None) => {
            // Like
            let _ = sqlx::query!(
                r#"INSERT INTO comment_likes (id, comment_id, user_id, user_type)
                VALUES ($1, $2, $3, $4)"#,
                Uuid::new_v4(),
                payload.comment_id,
                payload.user_id,
                payload.user_type
            )
            .execute(db.get_ref())
            .await;
        }
        Err(e) => {
            return HttpResponse::InternalServerError().json(json!({
                "status": "error",
                "message": format!("Database error: {}", e)
            }));
        }
    }

    // Get updated like count
    let count = sqlx::query_scalar!(
        r#"SELECT COUNT(*) FROM comment_likes WHERE comment_id = $1"#,
        payload.comment_id
    )
    .fetch_one(db.get_ref())
    .await
    .unwrap_or(Some(0)); 

    HttpResponse::Ok().json(json!({ "likes": count }))
}
