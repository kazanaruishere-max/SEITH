use serde::{Deserialize, Serialize};

use crate::config::env_loader;

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
        env_loader::load_env()?;

        Ok(Self {
            broker: BrokerSettings {
                name: env_loader::get_env_required("MT5_SERVER"),
                account_type: env_loader::get_env_required("MT5_ACCOUNT"),
            },
            telegram: TelegramSettings {
                bot_token: env_loader::get_env_required("TELEGRAM_BOT_TOKEN"),
                chat_id: env_loader::get_env_required("TELEGRAM_CHAT_ID"),
            },
            database: DatabaseSettings {
                url: env_loader::get_env_required("DATABASE_URL"),
            },
        })
    }
}
