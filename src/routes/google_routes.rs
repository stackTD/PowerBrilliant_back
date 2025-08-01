use actix_web::web;
use crate::handlers::google_handler::{google_login, google_callback};

pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.service(google_login)
       .service(google_callback);
}