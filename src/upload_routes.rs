use actix_web::web;
use crate::handlers::upload_handler::upload_profile_photo;

pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::resource("/upload/profile-photo")
            .route(web::post().to(upload_profile_photo))
    );
}
