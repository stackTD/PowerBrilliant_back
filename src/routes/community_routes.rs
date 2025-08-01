use actix_web::web;
use crate::handlers::communities_handlers::create_community;
use crate::handlers::communities_handlers::get_community;
use crate::handlers::communities_handlers::delete_community_by_id;
use crate::handlers::communities_handlers::update_community_by_id;
use crate::handlers::communities_handlers::get_community_by_id;
use crate::handlers::communities_handlers::get_communities_by_creator_id;



// use crate::handlers::joblist_handlers::delete_job_by_id;
// use crate::handlers::joblist_handlers::update_job_by_id;



pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/communities")
            .route("/create", web::post().to(create_community))
            .route("", web::get().to(get_community))
            .route("/delete/{id}", web::delete().to(delete_community_by_id))
            .route("/update/{id}", web::put().to(update_community_by_id))
            // Updated route with creator_type parameter
            .route("/creator/{creator_type}/{creator_id}", web::get().to(get_communities_by_creator_id))
            .service(
                web::resource("/communities/{id}")
                    .route(web::get().to(get_community_by_id))
            )
    );
}