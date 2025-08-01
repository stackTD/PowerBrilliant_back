use actix_web::web;
use crate::handlers::share_handler::share_post;

pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::resource("/post/share")
            .route(web::post().to(share_post))
    );
}