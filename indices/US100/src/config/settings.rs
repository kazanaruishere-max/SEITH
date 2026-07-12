use chrono::NaiveTime;

/// US100.cash — OANDA symbol
pub const SYMBOL: &str = "US100.cash";

/// Point value for US100 (Nasdaq)
pub const POINT_VALUE: f64 = 1.0;

/// Decimal places for price normalization
pub const DECIMAL_PLACES: u32 = 2;

/// Session hours UTC
pub const SESSION_OPEN: NaiveTime = NaiveTime::from_hms_opt(14, 30, 0).unwrap();
pub const SESSION_CLOSE: NaiveTime = NaiveTime::from_hms_opt(21, 0, 0).unwrap();

/// Phase time boundaries UTC
pub const OPEN_END: NaiveTime = NaiveTime::from_hms_opt(15, 0, 0).unwrap();
pub const LUNCH_START: NaiveTime = NaiveTime::from_hms_opt(16, 30, 0).unwrap();
pub const LUNCH_END: NaiveTime = NaiveTime::from_hms_opt(18, 0, 0).unwrap();
pub const POWER_HOUR_START: NaiveTime = NaiveTime::from_hms_opt(19, 30, 0).unwrap();
pub const CLOSE_START: NaiveTime = NaiveTime::from_hms_opt(20, 30, 0).unwrap();

/// Gap threshold for open skip (0.3%)
pub const GAP_THRESHOLD_PCT: f64 = 0.3;

/// Risk limits
pub const RISK_SNIPER: f64 = 0.0075;
pub const RISK_SCALP: f64 = 0.0050;
pub const MAX_DAILY_LOSS_PCT: f64 = 2.5;
pub const MAX_WEEKLY_LOSS_PCT: f64 = 5.0;
pub const MAX_OPEN_POSITIONS: u32 = 1;
pub const SPREAD_TOLERANCE: f64 = 1.5;

/// Force close time
pub const FORCE_CLOSE_MINUTES_BEFORE_END: i64 = 0;
pub const AUTO_KILL_DELAY_MINUTES: i64 = 5;

/// Lot sizing defaults
pub const BASE_UNIT_SNIPER: f64 = 0.0075;
pub const BASE_UNIT_SCALP: f64 = 0.0050;
pub const MODE_MULT_SNIPER: f64 = 1.0;
pub const MODE_MULT_SCALP: f64 = 0.67;

/// Confidence multipliers
pub const CONFIDENCE_5_5: f64 = 1.0;
pub const CONFIDENCE_HV_15_20: f64 = 0.75;
pub const CONFIDENCE_CRISIS: f64 = 0.5;

/// Macro multipliers
pub const MACRO_GREEN: f64 = 1.0;
pub const MACRO_ORANGE: f64 = 0.5;
pub const MACRO_RED: f64 = 0.0;

/// Slippage threshold for force immediate recalibration
pub const SLIPPAGE_FORCE_IMMEDIATE: f64 = 0.50;

/// Skip strike limits
pub const CRISIS_THRESHOLD: u32 = 3;
pub const CRISIS_CEILING: u32 = 5;
