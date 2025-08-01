use oauth2::{basic::BasicClient, AuthUrl, ClientId, ClientSecret, RedirectUrl, TokenUrl};
use std::env;

pub fn create_oauth_client() -> BasicClient {
    let client_id = ClientId::new(
        env::var("GOOGLE_CLIENT_ID").expect("Missing GOOGLE_CLIENT_ID environment variable.")
    );
    let client_secret = ClientSecret::new(
        env::var("GOOGLE_CLIENT_SECRET").expect("Missing GOOGLE_CLIENT_SECRET environment variable.")
    );
    let redirect_url = RedirectUrl::new(
        env::var("OAUTH_REDIRECT_URL").expect("Missing OAUTH_REDIRECT_URL environment variable.")
    ).expect("Invalid redirect URL");

    BasicClient::new(
        client_id,
        Some(client_secret),
        AuthUrl::new("https://accounts.google.com/o/oauth2/v2/auth".to_string())
            .expect("Invalid authorization endpoint URL"),
        Some(TokenUrl::new("https://oauth2.googleapis.com/token".to_string())
            .expect("Invalid token endpoint URL"))
    )
    .set_redirect_uri(redirect_url)
}