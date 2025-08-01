use actix_web::web;
use crate::handlers::business_accounts_handler::{
    create_business_account, 
    get_business_accounts, 
    get_business_account_by_id, 
    delete_business_account_by_email, 
    update_business_account_by_email
};

pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/business-accounts")
            .route("/create", web::post().to(create_business_account))
            .route("", web::get().to(get_business_accounts))
            .route("/{id}", web::get().to(get_business_account_by_id))
            .route("/delete/{email}", web::delete().to(delete_business_account_by_email))
            .route("/update/{email}", web::put().to(update_business_account_by_email))
    );
}
