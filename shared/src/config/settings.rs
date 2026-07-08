use serde::{Deserialize, Serialize};

/// Global settings untuk AI SEITH
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Settings {
    pub broker: BrokerSettings,
    pub telegram: TelegramSettings,
    pub database: DatabaseSettings,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BrokerSettings {
    pub name: String,
    pub account_type: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TelegramSettings {
    pub bot_token: String,
    pub chat_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatabaseSettings {
    pub url: String,
}

impl Settings {
    /// Load settings from environment variables
    pub fn from_env() -> anyhow::Result<Self> {
        todo!("Load settings from .env file")
    }
}
