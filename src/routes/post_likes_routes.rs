use actix_web::web;
use crate::handlers::post_likes_handles::create_post_like;
use crate::handlers::post_likes_handles::update_post_likes_by_id;
use crate::handlers::post_likes_handles::view_likes;
use crate::handlers::post_likes_handles::toggle_like_post;
use crate::handlers::post_likes_handles::get_post_likes;
use crate::handlers::post_likes_handles::get_liked_posts_by_user;
use crate::handlers::post_likes_handles::get_like_count_for_post;
// use crate::handlers::community_member_handlers::get_community_members;
// use crate::handlers::community_member_handlers::delete_community_member_by_id;
// use crate::handlers::community_member_handlers::update_community_member_by_id;



pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/post_like")
            .route("/create", web::post().to(create_post_like))
            .route("", web::get().to(view_likes))
            .route("/update/{id}", web::put().to(update_post_likes_by_id))
            .route("/like/toggle", web::post().to(toggle_like_post))
            .route("/list/{post_id}", web::get().to(get_post_likes))
            .route("/liked-posts", web::get().to(get_liked_posts_by_user)) // <-- updated
           .route("/count/{post_id}", web::get().to(get_like_count_for_post))   
   
    );
}