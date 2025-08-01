use actix_web::web;
use crate::handlers::upload_handler::upload_profile_photo;

pub fn config(cfg: &mut web::ServiceConfig) {
    // Remove .route() and use .service() instead
    cfg.service(upload_profile_photo);
}