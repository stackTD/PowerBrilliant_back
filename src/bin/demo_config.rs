// This is a demonstration script showing how environment variables work
// Run with: cargo run --bin demo_config

use std::env;

// Since we can't use external modules in bin files easily, let's inline a simple version
use dotenv;

fn main() {
    println!("🔧 Environment Variables Demo");
    println!("=============================\n");

    // Load environment variables from .env file
    dotenv::dotenv().ok();

    // Show current environment
    let current_env = env::var("APP_ENV").unwrap_or_else(|_| "development".to_string());
    println!("📊 Current Environment: {}", current_env);

    // Try to load individual environment variables
    println!("✅ Loading environment variables...\n");
    
    println!("🔍 Environment Variables:");
    check_env_var("DATABASE_URL", "PostgreSQL connection string");
    check_env_var("GOOGLE_CLIENT_ID", "Google OAuth client ID");
    check_env_var("GOOGLE_CLIENT_SECRET", "Google OAuth client secret");
    check_env_var("OAUTH_REDIRECT_URL", "OAuth callback URL");
    check_env_var("APP_ENV", "Application environment");
    check_env_var("API_PORT", "Server port");
    check_env_var("API_HOST", "Server host");
    check_env_var("RUST_LOG", "Logging level");
    check_env_var("JWT_SECRET", "JWT signing secret");
    check_env_var("API_KEY", "General API key");
    
    println!("\n🔍 Environment Detection:");
    if current_env == "production" {
        println!("  ⚡ Running in PRODUCTION mode");
        println!("  🔒 Security features enabled");
        println!("  📊 Production logging level");
    } else {
        println!("  🛠️  Running in DEVELOPMENT mode");
        println!("  🔓 Debug features enabled");
        println!("  📝 Verbose logging level");
    }

    println!("\n📋 Configuration Files:");
    println!("  .env - Development environment variables");
    println!("  .env.production - Production environment variables");
    println!("  .env.* - Environment-specific files (ignored by git)");

    println!("\n💡 Usage Tips:");
    println!("  - Copy .env file and update with your values");
    println!("  - Use .env.production for production deployment"); 
    println!("  - Never commit real credentials to version control");
    println!("  - Set APP_ENV=production to enable production mode");
    println!("  - The application loads configuration automatically on startup");
}

fn check_env_var(name: &str, description: &str) {
    match env::var(name) {
        Ok(value) => {
            let masked = mask_sensitive(&value);
            println!("  ✅ {} = {} ({})", name, masked, description);
        },
        Err(_) => {
            println!("  ❌ {} = <not set> ({})", name, description);
        }
    }
}

fn mask_sensitive(value: &str) -> String {
    if value.is_empty() || value.len() <= 8 {
        "*".repeat(value.len().max(3))
    } else {
        format!("{}...{}", &value[..4], "*".repeat(value.len() - 4))
    }
}