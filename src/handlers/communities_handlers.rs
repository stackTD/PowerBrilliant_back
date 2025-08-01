use actix_web::{web, HttpResponse, HttpRequest};
use uuid::Uuid;
use sqlx::PgPool;
use crate::models::communities::{Community, Create_Community, Update_Community, Pagination};
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

pub async fn get_communities_by_creator_id(
    db: web::Data<PgPool>,
    path: web::Path<(String, Uuid)>, // (creator_type, creator_id)
) -> HttpResponse {
    let (creator_type, creator_id) = path.into_inner();
    let query = match creator_type.as_str() {
        "user" => "SELECT * FROM communities WHERE creator_id = $1 ORDER BY created_at DESC",
        "business" => "SELECT * FROM communities WHERE creator_business_id = $1 ORDER BY created_at DESC",
        _ => return HttpResponse::BadRequest().json(json!({
            "status": "error",
            "message": "Invalid creator type"
        }))
    };

    let communities = sqlx::query_as::<_, Community>(query)
        .bind(creator_id)
        .fetch_all(db.get_ref())
        .await;

    match communities {
        Ok(communities) => {
            if communities.is_empty() {
                HttpResponse::NotFound().json(json!({
                    "status": "error",
                    "message": "No communities found for this creator"
                }))
            } else {
                HttpResponse::Ok().json(communities)
            }
        },
        Err(e) => {
            eprintln!("Database error: {:?}", e);
            HttpResponse::InternalServerError().json(json!({
                "status": "error",
                "message": format!("Error retrieving communities: {}", e)
            }))
        }
    }
}

pub async fn create_community(
    req: HttpRequest,
    db: web::Data<PgPool>,
    json: web::Json<Create_Community>,
) -> HttpResponse {
    let lang = get_lang(&req);            
    println!("Language preference: {}", lang);

    // Validate creator type
    if json.creator_type != "user" && json.creator_type != "business" {
        return HttpResponse::BadRequest().json(json!({
            "status": "error",
            "message": "Invalid creator type. Must be 'user' or 'business'"
        }));
    }

    // Validate creator IDs
    if json.creator_type == "user" && json.creator_id.is_none() {
        return HttpResponse::BadRequest().json(json!({
            "status": "error",
            "message": "creator_id required for user type"
        }));
    }

    if json.creator_type == "business" && json.creator_business_id.is_none() {
        return HttpResponse::BadRequest().json(json!({
            "status": "error",
            "message": "creator_business_id required for business type"
        }));
    }

    let result = sqlx::query_as::<_, Community>(
        r#"
        INSERT INTO communities (
            id, name, description, 
            creator_id, creator_business_id, creator_type,
            banner_url, is_public, is_active
        )
        VALUES ($1, $2, $3, $4, $5, $6, $7, COALESCE($8, true), COALESCE($9, true))
        RETURNING *
        "#
    )
    .bind(Uuid::new_v4())
    .bind(&json.name)
    .bind(&json.description)
    .bind(&json.creator_id)
    .bind(&json.creator_business_id)
    .bind(&json.creator_type)
    .bind(&json.banner_url)
    .bind(&json.is_public)
    .bind(&json.is_active)
    .fetch_one(db.get_ref())
    .await;

    match result {
        Ok(c) => HttpResponse::Ok().json(c),
        Err(e) => {
            eprintln!("DB error: {:?}", e);
            
            let error_message = if e.to_string().contains("foreign key constraint") {
                if json.creator_type == "user" {
                    "User not found with the provided creator_id"
                } else {
                    "Business account not found with the provided creator_business_id"
                }
            } else {
                "Error creating community"
            };
            
            HttpResponse::InternalServerError().json(json!({
                "status": "error",
                "message": error_message,
                "details": e.to_string()
            }))
        }
    }
}

pub async fn get_community(
    db: web::Data<PgPool>,
    web::Query(pagination): web::Query<Pagination>,
) -> HttpResponse {
    let limit = pagination.limit.unwrap_or(5);
    let offset = pagination.offset.unwrap_or(0);
 
    let users = sqlx::query_as::<_, Community>(
        "SELECT * FROM communities ORDER BY created_at DESC LIMIT $1 OFFSET $2"
    )
    .bind(limit)
    .bind(offset)
    .fetch_all(db.get_ref())
    .await;
 
    match users {
        Ok(users) => HttpResponse::Ok().json(users),
        Err(e) => {
            eprintln!("Database error: {:?}", e);
            HttpResponse::InternalServerError().body("Error retrieving community")
        }
    }
}

pub async fn delete_community_by_id(
    db: web::Data<PgPool>,
    id: web::Path<Uuid>,  // Changed from email to UUID
) -> HttpResponse {
    let result = sqlx::query!(
        "DELETE FROM communities WHERE id = $1 RETURNING id",
        id.into_inner()
    )
    .fetch_optional(db.get_ref())
    .await;

    match result {
        Ok(Some(_)) => HttpResponse::Ok().json(json!({
            "status": "success",
            "message": "community deleted successfully"
        })),
        Ok(None) => HttpResponse::NotFound().json(json!({
            "status": "error",
            "message": "community not found"
        })),
        Err(e) => {
            eprintln!("Database error: {:?}", e);
            HttpResponse::InternalServerError().json(json!({
                "status": "error",
                "message": "Error deleting community"
            }))
        }
    }
}

pub async fn update_community_by_id(
    db: web::Data<PgPool>,
    id: web::Path<Uuid>,
    json: web::Json<Update_Community>,
) -> HttpResponse {
    let query_result = sqlx::query!(
        r#"
        UPDATE communities 
        SET 
            name = COALESCE($1, name),
            description = COALESCE($2, description),
            banner_url = COALESCE($3, banner_url),
            is_public = COALESCE($4, is_public),
            is_active = COALESCE($5, is_active)
        WHERE id = $6
        RETURNING id
        "#,
        json.name.as_deref(),
        json.description,
        json.banner_url.as_deref(),
        json.is_public.unwrap_or(true),
        json.is_active.unwrap_or(true),
        id.into_inner(),
    )
    .fetch_optional(db.get_ref())
    .await;

    match query_result {
        Ok(Some(_)) => HttpResponse::Ok().json(json!({
            "status": "success",
            "message": "Community updated successfully"
        })),
        Ok(None) => HttpResponse::NotFound().json(json!({
            "status": "error",
            "message": "Community not found"
        })),
        Err(e) => {
            eprintln!("Database error: {:?}", e);
            HttpResponse::InternalServerError().json(json!({
                "status": "error",
                "message": "Error updating community"
            }))
        }
    }
}

pub async fn get_community_by_id(
    db: web::Data<PgPool>,
    id: web::Path<Uuid>,
) -> HttpResponse {
    let community = sqlx::query_as::<_, Community>(
        "SELECT * FROM communities WHERE id = $1"
    )
    .bind(id.into_inner())
    .fetch_optional(db.get_ref())
    .await;

    match community {
        Ok(Some(community)) => HttpResponse::Ok().json(community),
        Ok(None) => HttpResponse::NotFound().json(json!({
            "status": "error",
            "message": "Community not found"
        })),
        Err(e) => {
            eprintln!("Database error: {:?}", e);
            HttpResponse::InternalServerError().json(json!({
                "status": "error",
                "message": "Error retrieving community"
            }))
        }
    }
}
