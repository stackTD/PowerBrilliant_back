mod db;
mod models;
mod handlers;
mod routes;
mod oauth;
mod config;

use actix_web::{App, HttpServer, web};
use actix_files::Files;

use actix_cors::Cors;  // Add this line

use db::init_db;
use env_logger::Env;
use oauth::create_google_oauth_client;
use config::load_config;

use routes::{
    google_routes,
    follower_routes, 
    post_likes_routes, comments_routes,
     share_routes, user_badges_routes, user_routes, jobs_routes, job_application_routes, community_routes, community_members_routes, post_routes, business_accounts_routes
};

mod faker;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // Load configuration from environment variables
    let config = load_config();
    
    // Initialize logging with the configured level
    env_logger::init_from_env(Env::default().default_filter_or(&config.rust_log));

    println!("🌟 PowerBrilliant Server Starting...");
    println!("📊 Environment: {}", config.app_env);
    println!("🔌 Database: Connected");
    println!("🔑 OAuth: Configured");

    let db = init_db(&config).await;
    let google_oauth_client = create_google_oauth_client(&config);

    let server_address = format!("{}:{}", config.api_host, config.api_port);
    println!("🚀 Running on http://{}", server_address);

    // faker::seed_fake_users(&db, 100).await;
    // faker::seed_fake_communities(&db, 50).await;
    // faker::seed_fake_posts(&db, 50).await;
    // faker::seed_fake_jobs(&db, 100).await;


    HttpServer::new(move || {
        let cors = Cors::default()
            .allowed_origin("http://localhost:3000")
            .allowed_methods(vec!["GET", "POST", "PUT", "DELETE"])
            .allowed_headers(vec!["Content-Type", "Authorization"])
            .max_age(3600);

        App::new()
            .wrap(cors)  // Add the CORS middleware
            .app_data(web::Data::new(db.clone()))
            .app_data(web::Data::new(config.clone()))
           .service(
    Files::new("/uploads", "./uploads")
        .show_files_listing()
        .use_last_modified(true),
)

            .configure(share_routes::config)
            .app_data(web::Data::new(google_oauth_client.clone()))
            .configure(google_routes::config)
            .configure(user_routes::config)
            .configure(jobs_routes::config)
            .configure(community_routes::config)
            .configure(job_application_routes::config)
            .configure(community_members_routes::config)
            .configure(post_routes::config)
            .configure(user_badges_routes::config)
            .configure(comments_routes::config)
            .configure(post_likes_routes::config)
            .configure(follower_routes::config)
            .configure(business_accounts_routes::config) 
             
            // Register the share routes
            .configure(configure)
    })
    .bind(&server_address)?
    .run()
    .await
}

use crate::handlers::share_handler::share_post;

fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::resource("/share_post")
            .route(web::post().to(share_post))
    );
}

