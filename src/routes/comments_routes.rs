use actix_web::web;
use crate::handlers::comments_handler::create_comment;
use crate::handlers::comments_handler::get_comments;
use crate::handlers::comments_handler::delete_comment_by_id;
use crate::handlers::comments_handler::update_comment_by_id;
use crate::handlers::comments_handler::get_comments_by_post_id;
use crate::handlers::comments_handler::like_comment;




pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/comment")
            .route("/create", web::post().to(create_comment))
            .route("", web::get().to(get_comments))
            .route("/delete/{id}", web::delete().to(delete_comment_by_id))
            .route("/update/{id}", web::put().to(update_comment_by_id))
            .route("/post/{post_id}", web::get().to(get_comments_by_post_id))
            .route("/like", web::post().to(like_comment))
    );
}