// Custom error types using thiserror
// Stub only - no implementation yet

use thiserror::Error;

#[derive(Error, Debug)]
pub enum SeithError {
    #[error("Configuration error: {0}")]
    ConfigError(String),

    #[error("MT5 connection error: {0}")]
    Mt5Error(String),

    #[error("Database error: {0}")]
    DatabaseError(String),

    #[error("Trading error: {0}")]
    TradingError(String),

    #[error("Network error: {0}")]
    NetworkError(String),
}

pub type SeithResult<T> = Result<T, SeithError>;
