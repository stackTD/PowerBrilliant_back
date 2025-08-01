use actix_web::web;
use crate::handlers::joblist_handlers::create_job;
use crate::handlers::joblist_handlers::get_joblist;
use crate::handlers::joblist_handlers::delete_job_by_id;
use crate::handlers::joblist_handlers::update_job_by_id;
use crate::handlers::joblist_handlers::get_job_by_id;
use crate::handlers::joblist_handlers::get_jobs_by_posted_id;



pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/jobs")
            .route("/create", web::post().to(create_job))
            .route("", web::get().to(get_joblist))
            .route("/delete/{id}", web::delete().to(delete_job_by_id))
            .route("/update/{id}", web::put().to(update_job_by_id))
            .route("/{id}", web::get().to(get_job_by_id))
            .route("/by-poster/{posted_id}", web::get().to(get_jobs_by_posted_id))

            
    );
}
