use actix_web::web;
use crate::handlers::follower_handlers::create_follower;
use crate::handlers::follower_handlers::view_followers;
use crate::handlers::follower_handlers::delete_follower_by_id;
use crate::handlers::follower_handlers::update_follower_by_id;
use crate::handlers::follower_handlers::toggle_follow;
use crate::handlers::follower_handlers:: follow_status;
use crate::handlers::follower_handlers::get_followers_count;
use crate::handlers::follower_handlers::get_following_count;
use crate::handlers::follower_handlers::get_followers_list;
use crate::handlers::follower_handlers::get_following_list;

pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/follower")
            .route("/create", web::post().to(create_follower))
            .route("/", web::get().to(view_followers))
            .route("/delete/{id}", web::delete().to(delete_follower_by_id))
            .route("/update/{id}", web::put().to(update_follower_by_id))
            .route("/toggle", web::post().to(toggle_follow))
            .route("/status/{follower_id}/{followed_id}", web::get().to(follow_status))
            .route("/count/{user_id}", web::get().to(get_followers_count))
            .route("/following/count/{user_id}", web::get().to(get_following_count))
            .route("/followers/list/{user_id}", web::get().to(get_followers_list))
            .route("/following/list/{user_id}", web::get().to(get_following_list)),
    );
}