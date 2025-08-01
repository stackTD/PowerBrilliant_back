use actix_web::{get, http::header, web, HttpResponse, Responder};
use chrono::Utc;
use jsonwebtoken::{encode, EncodingKey, Header};
use log::{debug, error};
use oauth2::basic::BasicClient;
use oauth2::{AuthorizationCode, CsrfToken, Scope, TokenResponse};
use rand::Rng;
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use uuid::Uuid;

// Note: Ensure the `CreateUser` struct in `models/user.rs` also has `profile_pic: Option<String>`
use crate::models::user::{CreateUser, User}; 
use crate::oauth::create_google_oauth_client;
use crate::config::Config;

#[derive(Deserialize, Debug)]
struct GoogleUser {
    id: String,
    email: String,
    name: String,
    given_name: String,
    family_name: String,
    picture: Option<String>,
}

// FIX #1: Add `profile_pic` to the JWT Claims
// This allows the frontend to get the image URL immediately upon login.
#[derive(Serialize)]
struct Claims {
    email: String,
    name: Option<String>,
    profile_pic: Option<String>, 
    exp: usize,
}

#[get("/auth/google/login")]
pub async fn google_login(oauth_client: web::Data<BasicClient>) -> impl Responder {
    debug!("Initiating Google OAuth login flow");
    let (auth_url, _csrf_token) = oauth_client
        .authorize_url(CsrfToken::new_random)
        .add_scope(Scope::new("https://www.googleapis.com/auth/userinfo.profile".to_string()))
        .add_scope(Scope::new("https://www.googleapis.com/auth/userinfo.email".to_string()))
        .url();
    debug!("Generated authorization URL: {}", auth_url);
    HttpResponse::Found()
        .append_header((header::LOCATION, auth_url.to_string()))
        .finish()
}

#[get("/auth/google/callback")]
pub async fn google_callback(
    params: web::Query<std::collections::HashMap<String, String>>,
    db: web::Data<PgPool>,
    oauth_client: web::Data<BasicClient>,
    config: web::Data<Config>,
) -> impl Responder {
    let code = match params.get("code") {
        Some(c) => AuthorizationCode::new(c.clone()),
        None => {
            error!("Missing authorization code in callback");
            return HttpResponse::BadRequest().body("Missing authorization code");
        }
    };

    let client = oauth_client.get_ref();
    let token = match client
        .exchange_code(code)
        .request_async(oauth2::reqwest::async_http_client)
        .await
    {
        Ok(token) => token,
        Err(e) => {
            error!("OAuth token exchange error: {}", e);
            return HttpResponse::InternalServerError().body("Token exchange error");
        }
    };

    let google_user: GoogleUser = match reqwest::Client::new()
        .get("https://www.googleapis.com/oauth2/v2/userinfo")
        .bearer_auth(token.access_token().secret())
        .send()
        .await
    {
        Ok(resp) => {
            if !resp.status().is_success() {
                error!("Google userinfo API returned non-success status: {}", resp.status());
                return HttpResponse::InternalServerError().body("Failed to get user info from Google");
            }
            match resp.json().await {
                Ok(user) => user,
                Err(e) => {
                    error!("Failed to parse JSON from Google user info: {}", e);
                    return HttpResponse::InternalServerError().body("Failed to parse user info");
                }
            }
        },
        Err(e) => {
            error!("Request to Google user info endpoint failed: {}", e);
            return HttpResponse::InternalServerError().body("Failed to fetch user info");
        }
    };
    
    // --- FIX #2: Refactored Logic to Handle ALL Users Correctly ---

    // Find an existing user or create a new one. `mut` is needed to update the pic later.
    let mut user = match get_user_by_email_internal(&db, &google_user.email).await {
        Some(user) => {
            debug!("Existing user found: {}", user.email);
            user // User already exists
        },
        None => {
            debug!("User not found, creating new user for: {}", &google_user.email);
            // User does not exist, create a new one
            let username = match generate_unique_username(&db, &google_user.email).await {
                Ok(u) => u,
                Err(e) => {
                    error!("Failed to generate a unique username: {}", e);
                    return HttpResponse::InternalServerError().body("Failed to create username");
                }
            };
            
            // CreateUser now includes the profile picture directly.
            let new_user_data = CreateUser {
                first_name: google_user.given_name,
                last_name: google_user.family_name,
                username,
                password: "provided_by_google".to_string(), // Not used for OAuth logins
                email: google_user.email,
                provider: Some("google".to_string()),
                profile_pic: google_user.picture.clone(), // Set profile pic on creation
                provider_user_id: Some(google_user.id),
                access_token: Some(token.access_token().secret().to_string()),
                bio: None,
                interests: None,
                resume_url: None,
            };

            match create_user_internal(&db, new_user_data).await {
                Ok(user) => {
                    debug!("Successfully created new user with ID: {}", user.id);
                    user
                },
                Err(e) => {
                    error!("Database error while creating user: {}", e);
                    return HttpResponse::InternalServerError().body("Failed to create user");
                }
            }
        }
    };

    // **CRITICAL FIX:** This block now runs for EVERYONE.
    // It updates the profile picture if the one from Google is different from the one in our DB.
    if let Some(picture_url) = &google_user.picture {
        if user.profile_pic.as_deref() != Some(picture_url) {
            debug!("Updating profile picture for user ID: {}", user.id);
            if let Err(e) = update_profile_picture(&db, &user.id, picture_url).await {
                error!("Failed to update profile picture for user {}: {}", user.id, e);
            } else {
                user.profile_pic = Some(picture_url.clone()); // Update local `user` object for the JWT
            }
        }
    }

    // --- FIX #3: Generate JWT with the profile picture included ---
    let claims = Claims {
        email: user.email,
        name: Some(format!("{} {}", user.first_name, user.last_name)),
        profile_pic: user.profile_pic, // <-- INCLUDE THE PICTURE IN THE TOKEN
        exp: (Utc::now() + chrono::Duration::hours(24)).timestamp() as usize,
    };

    let jwt_secret = &config.jwt_secret;
    let token = match encode(&Header::default(), &claims, &EncodingKey::from_secret(jwt_secret.as_bytes())) {
        Ok(t) => t,
        Err(e) => {
            error!("JWT encoding error: {}", e);
            return HttpResponse::InternalServerError().body("Failed to generate token");
        }
    };
    
    // Redirect to your frontend with the complete token
    let frontend_url = format!("http://localhost:3000/auth/callback?token={}", token);
    debug!("Redirecting to frontend: {}", frontend_url);
    HttpResponse::Found()
        .append_header((header::LOCATION, frontend_url))
        .finish()
}

// --- HELPER FUNCTIONS (WITH IMPROVEMENTS) ---

async fn get_user_by_email_internal(db: &PgPool, email: &str) -> Option<User> {
    // `query_as!` requires the `User` struct to perfectly match the `SELECT *` columns.
    // This is fragile but maintained from your original code.
    sqlx::query_as!(User, "SELECT * FROM users WHERE LOWER(email) = $1", email.to_lowercase())
        .fetch_optional(db)
        .await
        .unwrap_or_else(|e| {
            error!("Database error fetching user by email: {}", e);
            None
        })
}

// FIX #4: `create_user` now includes `profile_pic` for an efficient single-query creation.
async fn create_user_internal(db: &PgPool, user_data: CreateUser) -> Result<User, sqlx::Error> {
    sqlx::query_as!(
        User,
        r#"
        INSERT INTO users (
            id, first_name, last_name, username, password, email, provider, 
            provider_user_id, access_token, profile_pic
        )
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
        RETURNING *
        "#,
        Uuid::new_v4(),
        user_data.first_name,
        user_data.last_name,
        user_data.username,
        user_data.password,
        user_data.email,
        user_data.provider,
        user_data.provider_user_id,
        user_data.access_token,
        user_data.profile_pic
    )
    .fetch_one(db)
    .await
}

async fn update_profile_picture(db: &PgPool, user_id: &Uuid, picture_url: &str) -> Result<(), sqlx::Error> {
    sqlx::query!(
        "UPDATE users SET profile_pic = $1, updated_at = NOW() WHERE id = $2",
        picture_url,
        user_id
    )
    .execute(db)
    .await?;
    Ok(())
}

async fn generate_unique_username(db: &PgPool, email: &str) -> Result<String, sqlx::Error> {
    let base = email.split('@').next().unwrap_or("user")
        .to_string()
        .replace(|c: char| !c.is_alphanumeric(), ""); // Sanitize base username

    let mut username = base.clone();
    let mut i = 0;
    loop {
        let exists: bool = sqlx::query_scalar!("SELECT EXISTS(SELECT 1 FROM users WHERE username = $1)", username)
            .fetch_one(db).await?.unwrap_or(false);
            
        if !exists { return Ok(username); }

        i += 1;
        if i > 10 { // Prevent infinite loops
             return Err(sqlx::Error::RowNotFound); // Or a custom error
        }

        // Append a random 4-digit number if the base username is taken
        let suffix: u16 = rand::thread_rng().gen_range(1000..=9999);
        username = format!("{}{}", if base.len() > 25 { &base[..25] } else { &base }, suffix);
    }
}