use std::env;

/// Load environment variables from .env file (absolute path safe)
pub fn load_env() -> anyhow::Result<()> {
    let env_path = std::path::Path::new("C:/Users/Lenovo/PROJECT/AI SEITH/.env");
    if env_path.exists() {
        dotenvy::from_path(env_path).ok();
    }
    log::info!("Environment variables loaded");
    Ok(())
}

/// Get environment variable with fallback
pub fn get_env_or(key: &str, default: &str) -> String {
    env::var(key).unwrap_or_else(|_| default.to_string())
}

/// Get required environment variable (returns error if not found)
pub fn get_env_required(key: &str) -> anyhow::Result<String> {
    env::var(key).map_err(|_| {
        anyhow::anyhow!(
            "Required environment variable '{}' not found. Check .env file.",
            key
        )
    })
}
