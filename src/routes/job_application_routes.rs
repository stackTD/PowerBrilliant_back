use actix_web::web;
use crate::handlers::job_application_handler::{
    create_job_application, 
    get_applications_for_job,
    get_applied_jobs_for_user,
    get_applied_jobs_count_for_user,
    get_application_for_job_and_user

};
// use crate::handlers::joblist_handlers::get_joblist;
// use crate::handlers::joblist_handlers::delete_job_by_id;
// use crate::handlers::joblist_handlers::update_job_by_id;



pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/jobApplications")
            .route("/create", web::post().to(create_job_application))
            .route("/job/{job_id}", web::get().to(get_applications_for_job))
           
             .route("/{user_id}", web::get().to(get_applied_jobs_for_user))
            .route("/count/{user_id}", web::get().to(get_applied_jobs_count_for_user))
            .route("job/{job_id}/user/{user_id}", web::get().to(get_application_for_job_and_user))
    );
    ;
}
   
