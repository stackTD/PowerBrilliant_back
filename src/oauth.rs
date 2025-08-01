use oauth2::{
    basic::BasicClient, AuthUrl, ClientId, ClientSecret, RedirectUrl, TokenUrl,
};
use crate::config::Config;

pub fn create_google_oauth_client(config: &Config) -> BasicClient {
    let google_client_id = ClientId::new(config.google_client_id.clone());
    let google_client_secret = ClientSecret::new(config.google_client_secret.clone());
    
    let auth_url = AuthUrl::new("https://accounts.google.com/o/oauth2/v2/auth".to_string())
        .expect("Invalid auth URL");
    let token_url = TokenUrl::new("https://oauth2.googleapis.com/token".to_string())
        .expect("Invalid token URL");
    let redirect_url = RedirectUrl::new(config.oauth_redirect_url.clone())
        .expect("Invalid redirect URL");

    BasicClient::new(
        google_client_id,
        Some(google_client_secret),
        auth_url,
        Some(token_url)
    )
    .set_redirect_uri(redirect_url)
}