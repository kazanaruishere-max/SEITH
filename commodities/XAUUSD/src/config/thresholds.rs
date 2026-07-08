// XAUUSD trading thresholds
// Stub only - no implementation yet

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Thresholds {
    pub net_dev_min: f64,
    pub ofs_min: i32,
    pub gvz_zscore_threshold: f64,
    pub bayesian_min_probability: f64,
    pub frama_deviation_max: f64,
}

impl Default for Thresholds {
    fn default() -> Self {
        Self {
            net_dev_min: 2.0,
            ofs_min: 2,
            gvz_zscore_threshold: 1.0,
            bayesian_min_probability: 0.60,
            frama_deviation_max: 0.5,
        }
    }
}
