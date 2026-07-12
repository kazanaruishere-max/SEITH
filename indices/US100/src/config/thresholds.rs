/// HV Z-Score thresholds
pub const HV_SKIP_LOW: f64 = -1.0;
pub const HV_SWEET_SPOT_MAX: f64 = 1.5;
pub const HV_ELEVATED_MAX: f64 = 2.0;
pub const HV_SKIP_HIGH: f64 = 2.0;

/// FRAMA
pub const FRAMA_OVEREXTENDED: f64 = 0.5;

/// OFS thresholds per phase
pub const OFS_NORMAL: f64 = 3.0;
pub const OFS_POWER_HOUR: f64 = 2.0;
pub const OFS_CRISIS_RELAX: f64 = 2.0;
pub const OFS_NOISE_MAX: f64 = 1.0;

/// VWAP band width per phase
pub const VWAP_BAND_NORMAL: f64 = 2.5;
pub const VWAP_BAND_POWER_HOUR: f64 = 2.0;

/// Yield Z-Score thresholds
pub const YIELD_BEARISH: f64 = 1.5;
pub const YIELD_BULLISH: f64 = -1.5;

/// RR ratios
pub const RR_SNIPER_TARGET: f64 = 1.5;
pub const RR_SCALP_TARGET: f64 = 0.5;

/// Scalp SL/TP range
pub const SCALP_SL_MIN: f64 = 5.0;
pub const SCALP_SL_MAX: f64 = 10.0;
pub const SCALP_TP_MIN: f64 = 8.0;
pub const SCALP_TP_MAX: f64 = 15.0;

/// VIX yfinance fallback
pub const VIX_DEFAULT_BASELINE: f64 = 18.0;

/// Batch recalibration
pub const SCALP_BATCH_SIZE: u32 = 2;
