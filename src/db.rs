use sqlx::postgres::PgPoolOptions;
use sqlx::PgPool;
use std::env;
use dotenv::dotenv;

pub async fn init_db() -> PgPool {
    dotenv().ok();
    let db_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    PgPoolOptions::new()
        .max_connections(5)
        .connect(&db_url)
        .await
        .expect("Failed to connect to database")
}
