use std::env;

#[derive(Debug, Clone)]
pub struct Config {
    pub database_url: String,
    pub google_client_id: String,
    pub google_client_secret: String,
    pub oauth_redirect_url: String,
    pub app_env: String,
    pub api_port: u16,
    pub api_host: String,
    pub rust_log: String,
    pub jwt_secret: String,
    pub api_key: String,
}

impl Config {
    pub fn from_env() -> Result<Self, env::VarError> {
        Ok(Config {
            database_url: env::var("DATABASE_URL")?,
            google_client_id: env::var("GOOGLE_CLIENT_ID")?,
            google_client_secret: env::var("GOOGLE_CLIENT_SECRET")?,
            oauth_redirect_url: env::var("OAUTH_REDIRECT_URL")?,
            app_env: env::var("APP_ENV").unwrap_or_else(|_| "development".to_string()),
            api_port: env::var("API_PORT")
                .unwrap_or_else(|_| "8080".to_string())
                .parse()
                .unwrap_or(8080),
            api_host: env::var("API_HOST").unwrap_or_else(|_| "127.0.0.1".to_string()),
            rust_log: env::var("RUST_LOG").unwrap_or_else(|_| "debug".to_string()),
            jwt_secret: env::var("JWT_SECRET").unwrap_or_else(|_| "default_dev_secret".to_string()),
            api_key: env::var("API_KEY").unwrap_or_else(|_| "default_dev_api_key".to_string()),
        })
    }

    pub fn is_production(&self) -> bool {
        self.app_env == "production"
    }

    pub fn is_development(&self) -> bool {
        self.app_env == "development"
    }
}

impl Default for Config {
    fn default() -> Self {
        Config {
            database_url: "postgres://postgres:1234@localhost/powerbrilliant".to_string(),
            google_client_id: "".to_string(),
            google_client_secret: "".to_string(),
            oauth_redirect_url: "http://localhost:8080/auth/google/callback".to_string(),
            app_env: "development".to_string(),
            api_port: 8080,
            api_host: "127.0.0.1".to_string(),
            rust_log: "debug".to_string(),
            jwt_secret: "default_dev_secret".to_string(),
            api_key: "default_dev_api_key".to_string(),
        }
    }
}

// Example usage function
pub fn load_config() -> Config {
    dotenv::dotenv().ok();
    Config::from_env().unwrap_or_else(|err| {
        eprintln!("Warning: Failed to load some environment variables: {}", err);
        eprintln!("Using default configuration for missing variables");
        Config::default()
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;

    #[test]
    fn test_config_default() {
        let config = Config::default();
        assert_eq!(config.app_env, "development");
        assert_eq!(config.api_port, 8080);
        assert!(config.is_development());
        assert!(!config.is_production());
    }

    #[test]
    fn test_config_production_detection() {
        env::set_var("APP_ENV", "production");
        let config = Config::from_env().unwrap_or_default();
        assert!(config.is_production());
        assert!(!config.is_development());
        env::remove_var("APP_ENV");
    }
}