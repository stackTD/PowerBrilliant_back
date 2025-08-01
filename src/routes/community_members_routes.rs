use actix_web::web;
use crate::handlers::community_member_handlers::create_community_member;
use crate::handlers::community_member_handlers::get_community_members;
use crate::handlers::community_member_handlers::delete_community_member_by_id;
use crate::handlers::community_member_handlers::update_community_member_by_id;



pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/community_member")
            .route("/create", web::post().to(create_community_member))
            .route("", web::get().to(get_community_members))
            .route("/delete/{id}", web::delete().to(delete_community_member_by_id))
            .route("/update/{id}", web::put().to(update_community_member_by_id))
            
    );
}