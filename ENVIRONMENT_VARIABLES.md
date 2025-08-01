# Environment Variables Example

This document demonstrates how to use environment variables in PowerBrilliant backend application.

## Overview

The application uses the `dotenv` crate to load environment variables from `.env` files. The configuration is centralized in the `config` module for easy management.

## Environment Files

### Development (.env)
Used for local development. Contains development-specific settings with safe defaults.

### Production (.env.production)
Used for production deployment. Contains production-specific settings.

## Configuration Module Usage

### Basic Usage

```rust
use crate::config::{Config, load_config};

// Load configuration from environment
let config = load_config();

// Access configuration values
println!("Database URL: {}", config.database_url);
println!("Environment: {}", config.app_env);
```

### Environment Detection

```rust
use crate::config::load_config;

let config = load_config();

if config.is_production() {
    println!("Running in production mode");
    // Production-specific logic
} else if config.is_development() {
    println!("Running in development mode");
    // Development-specific logic
}
```

### Custom Configuration

```rust
use crate::config::Config;
use std::env;

// Create configuration from environment variables
let config = Config::from_env().unwrap_or_else(|err| {
    eprintln!("Configuration error: {}", err);
    Config::default()
});

// Use in database connection
let db = init_db(&config).await;

// Use in OAuth setup
let oauth_client = create_google_oauth_client(&config);
```

## Environment Variables Reference

| Variable | Description | Default | Required |
|----------|-------------|---------|----------|
| `DATABASE_URL` | PostgreSQL connection string | `postgres://postgres:1234@localhost/powerbrilliant` | Yes |
| `GOOGLE_CLIENT_ID` | Google OAuth client ID | - | Yes |
| `GOOGLE_CLIENT_SECRET` | Google OAuth client secret | - | Yes |
| `OAUTH_REDIRECT_URL` | OAuth callback URL | `http://localhost:8080/auth/google/callback` | Yes |
| `APP_ENV` | Application environment (development/production) | `development` | No |
| `API_PORT` | Server port | `8080` | No |
| `API_HOST` | Server host | `127.0.0.1` | No |
| `RUST_LOG` | Logging level | `debug` | No |
| `JWT_SECRET` | JWT signing secret | `default_dev_secret` | No |
| `API_KEY` | General API key | `default_dev_api_key` | No |

## Setup Instructions

### Development Setup

1. Copy the provided `.env` file to your project root
2. Update the values with your development credentials:
   ```bash
   GOOGLE_CLIENT_ID=your_actual_dev_client_id
   GOOGLE_CLIENT_SECRET=your_actual_dev_client_secret
   ```
3. Run the application: `cargo run`

### Production Setup

1. Copy `.env.production` to your production server
2. Update all values with your production credentials
3. Set the environment to load the production file:
   ```bash
   # Option 1: Rename the file
   mv .env.production .env
   
   # Option 2: Use environment-specific loading (if implemented)
   export APP_ENV=production
   ```

## Security Best Practices

1. **Never commit actual credentials** to version control
2. **Use strong, unique secrets** in production
3. **Rotate secrets regularly**
4. **Use different credentials** for different environments
5. **Restrict database access** by IP and user permissions

## Examples in Code

### Database Connection

```rust
// In db.rs
use crate::config::Config;

pub async fn init_db(config: &Config) -> PgPool {
    PgPoolOptions::new()
        .max_connections(5)
        .connect(&config.database_url)
        .await
        .expect("Failed to connect to database")
}
```

### OAuth Configuration

```rust
// In oauth.rs
use crate::config::Config;

pub fn create_google_oauth_client(config: &Config) -> BasicClient {
    let client_id = ClientId::new(config.google_client_id.clone());
    let client_secret = ClientSecret::new(config.google_client_secret.clone());
    // ... rest of the OAuth setup
}
```

### Environment-Specific Logic

```rust
// In handlers or services
use crate::config::load_config;

pub async fn some_handler() -> HttpResponse {
    let config = load_config();
    
    if config.is_production() {
        // Use production API endpoints
        // Enable production logging
        // Use production rate limits
    } else {
        // Use development/test endpoints
        // Enable debug logging  
        // Relaxed rate limits
    }
    
    HttpResponse::Ok().json("response")
}
```

## Troubleshooting

### Common Issues

1. **Missing environment variables**: Check if all required variables are set
   ```bash
   # Check if variable is set
   echo $DATABASE_URL
   ```

2. **Wrong database connection**: Verify DATABASE_URL format
   ```
   postgres://username:password@host:port/database_name
   ```

3. **OAuth errors**: Ensure GOOGLE_CLIENT_ID and GOOGLE_CLIENT_SECRET are valid

### Debug Configuration

Add this to your code to debug configuration loading:

```rust
use crate::config::load_config;

let config = load_config();
println!("Loaded configuration: {:#?}", config);
```