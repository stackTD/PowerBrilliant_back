use actix_web::web;
use crate::handlers::user_handler::{create_user, get_users, get_user_by_email, delete_user_by_email, update_user_by_email, get_user_by_id,get_public_profile, get_user_interest_by_id};



// ... existing code ...

pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/users")
            .route("/create", web::post().to(create_user))
            .route("", web::get().to(get_users))
            .route("/public-profile/{identifier}", web::get().to(get_public_profile))
            .route("/{id}", web::get().to(get_user_by_id))
            .route("/delete/{email}", web::delete().to(delete_user_by_email))
            .route("/update/{email}", web::put().to(update_user_by_email))
           
            .route("/{id}/interest", web::get().to(get_user_interest_by_id))
            .route("/email/{email}", web::get().to(get_user_by_email))
    );
}