use oauth2::{
    basic::BasicClient, AuthUrl, ClientId, ClientSecret, RedirectUrl, TokenUrl,
};

pub fn create_google_oauth_client() -> BasicClient {
    let google_client_id = ClientId::new(
        std::env::var("GOOGLE_CLIENT_ID").expect("Missing GOOGLE_CLIENT_ID")
    );
    let google_client_secret = ClientSecret::new(
        std::env::var("GOOGLE_CLIENT_SECRET").expect("Missing GOOGLE_CLIENT_SECRET")
    );
    let auth_url = AuthUrl::new("https://accounts.google.com/o/oauth2/v2/auth".to_string())
        .expect("Invalid auth URL");
    let token_url = TokenUrl::new("https://oauth2.googleapis.com/token".to_string())
        .expect("Invalid token URL");
    let redirect_url = RedirectUrl::new(
        std::env::var("OAUTH_REDIRECT_URL").expect("Missing OAUTH_REDIRECT_URL")
    ).expect("Invalid redirect URL");

    BasicClient::new(
        google_client_id,
        Some(google_client_secret),
        auth_url,
        Some(token_url)
    )
    .set_redirect_uri(redirect_url)
}