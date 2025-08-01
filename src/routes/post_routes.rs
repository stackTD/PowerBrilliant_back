use actix_web::web;
use crate::handlers::post_handlers::{get_posts, create_post, get_posts_by_user_interest,  get_tags_by_post_id,delete_post_by_id, update_post_by_id, get_post_by_id, get_posts_by_author_id, upload_media};

pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/post")
            .route("/create", web::post().to(create_post))
            .route("/tags", web::get().to(get_tags_by_post_id))
            .route("/upload", web::post().to(upload_media)) 
            .route("", web::get().to(get_posts))
            .route("/{id}", web::get().to(get_post_by_id))  
            .route("/delete/{id}", web::delete().to(delete_post_by_id))
            .route("/update/{id}", web::put().to(update_post_by_id))
            .route("/by_interest/{user_id}", web::get().to(get_posts_by_user_interest))
            .route("/author/{author_id}", web::get().to(get_posts_by_author_id))
    );
}