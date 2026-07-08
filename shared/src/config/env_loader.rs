use dotenvy::dotenv;
use std::env;

/// Load environment variables from .env file
pub fn load_env() -> anyhow::Result<()> {
    dotenv().ok();
    log::info!("Environment variables loaded");
    Ok(())
}

/// Get environment variable with fallback
pub fn get_env_or(key: &str, default: &str) -> String {
    env::var(key).unwrap_or_else(|_| default.to_string())
}

/// Get required environment variable (panic if not found)
pub fn get_env_required(key: &str) -> String {
    env::var(key).unwrap_or_else(|_| panic!("Required environment variable '{}' not found", key))
}
