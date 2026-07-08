// XAUUSD-specific settings
// Stub only - no implementation yet

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct XauusdSettings {
    pub symbol: String,
    pub spread_tolerance: f64,
    pub digit_multiplier: f64,
    pub max_lot: f64,
    pub min_lot: f64,
}

impl Default for XauusdSettings {
    fn default() -> Self {
        Self {
            symbol: "XAUUSDm".to_string(),
            spread_tolerance: 3.0,
            digit_multiplier: 0.010,
            max_lot: 0.01,
            min_lot: 0.01,
        }
    }
}
