use actix_web::{web, HttpResponse, Responder};
use sqlx::PgPool;
use uuid::Uuid;
use crate::models::share_post::{SharePostPayload, SharePostResponse};


pub async fn share_post(
    db: web::Data<PgPool>,
    json: web::Json<SharePostPayload>,
) -> impl Responder {
    for recipient_id in &json.shared_with {
        if let Err(e) = sqlx::query!(
            r#"
            INSERT INTO shared_posts (id, post_id, shared_by, shared_with, message)
            VALUES ($1, $2, $3, $4, $5)
            "#,
            Uuid::new_v4(),
            json.post_id,
            json.shared_by,
            recipient_id,
            json.message
        )
        .execute(db.get_ref())
        .await {
            eprintln!("Failed to insert share record: {:?}", e);
            return HttpResponse::InternalServerError().json(SharePostResponse {
                status: "error".to_string(),
                message: "Failed to share post".to_string(),
            });
        }
    }

    HttpResponse::Ok().json(SharePostResponse {
        status: "success".to_string(),
        message: "Post shared successfully".to_string(),
    })
}