use actix_web::web;
use crate::handlers::user_badges_handlers::create_user_badge;
use crate::handlers::user_badges_handlers::get_user_badges;
use crate::handlers::user_badges_handlers::delete_user_badge_by_id;



pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/user_badges")
            .route("/create", web::post().to(create_user_badge))
            .route("", web::get().to(get_user_badges))
            .route("/delete/{id}", web::delete().to(delete_user_badge_by_id))
            // .route("/update/{id}", web::put().to(update_post_by_id))
            
    );
}