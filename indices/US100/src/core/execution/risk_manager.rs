use crate::config::settings;

#[derive(Debug, Clone)]
pub struct LotSize {
    pub base_unit: f64,
    pub confidence_mult: f64,
    pub mode_mult: f64,
    pub macro_mult: f64,
    pub final_lot: f64,
}

pub struct RiskLimitViolation {
    pub reason: String,
}

pub struct RiskManager;

impl RiskManager {
    pub fn new() -> Self {
        Self
    }

    pub fn calculate_base_unit(equity: f64, risk_pct: f64, sl_points: f64) -> f64 {
        if sl_points < 1.0 || equity < 1.0 {
            return 0.0;
        }
        equity * risk_pct / (sl_points * settings::POINT_VALUE)
    }

    pub fn calculate_lot(
        equity: f64,
        risk_pct: f64,
        sl_points: f64,
        confidence_mult: f64,
        mode_mult: f64,
        macro_mult: f64,
    ) -> LotSize {
        let base_unit = Self::calculate_base_unit(equity, risk_pct, sl_points);
        let final_lot = base_unit * confidence_mult * mode_mult * macro_mult;
        LotSize { base_unit, confidence_mult, mode_mult, macro_mult, final_lot }
    }

    pub fn validate_risk(
        _equity: f64,
        risk_pct: f64,
        daily_loss_pct: f64,
        weekly_loss_pct: f64,
        spread: f64,
        current_positions: u32,
    ) -> Result<(), RiskLimitViolation> {
        if risk_pct > settings::MAX_DAILY_LOSS_PCT / 100.0 {
            return Err(RiskLimitViolation { reason: "Risk exceeds max daily loss limit".into() });
        }
        if daily_loss_pct >= settings::MAX_DAILY_LOSS_PCT {
            return Err(RiskLimitViolation { reason: "Daily loss limit reached".into() });
        }
        if weekly_loss_pct >= settings::MAX_WEEKLY_LOSS_PCT {
            return Err(RiskLimitViolation { reason: "Weekly loss limit reached".into() });
        }
        if spread > settings::SPREAD_TOLERANCE {
            return Err(RiskLimitViolation { reason: format!("Spread {} exceeds tolerance {}", spread, settings::SPREAD_TOLERANCE) });
        }
        if current_positions >= settings::MAX_OPEN_POSITIONS {
            return Err(RiskLimitViolation { reason: "Max open positions reached".into() });
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_calculate_base_unit_sniper() {
        let bu = RiskManager::calculate_base_unit(10_000.0, 0.0075, 10.0);
        assert!((bu - 7.5).abs() < 1e-6);
    }

    #[test]
    fn test_calculate_base_unit_scalp() {
        let bu = RiskManager::calculate_base_unit(10_000.0, 0.0050, 8.0);
        assert!((bu - 6.25).abs() < 1e-6);
    }

    #[test]
    fn test_calculate_lot_full() {
        let lot = RiskManager::calculate_lot(10_000.0, 0.0075, 10.0, 1.0, 1.0, 1.0);
        assert!((lot.final_lot - 7.5).abs() < 1e-6);
    }

    #[test]
    fn test_calculate_lot_scalp() {
        let lot = RiskManager::calculate_lot(10_000.0, 0.0050, 8.0, 1.0, 0.67, 1.0);
        assert!((lot.final_lot - 4.1875).abs() < 1e-4);
    }

    #[test]
    fn test_calculate_lot_with_multipliers() {
        let lot = RiskManager::calculate_lot(10_000.0, 0.0075, 10.0, 0.75, 1.0, 0.5);
        assert!((lot.final_lot - 2.8125).abs() < 1e-4);
    }

    #[test]
    fn test_validate_risk_ok() {
        let r = RiskManager::validate_risk(10_000.0, 0.0075, 1.0, 2.0, 0.5, 0);
        assert!(r.is_ok());
    }

    #[test]
    fn test_validate_risk_daily_loss_exceeded() {
        let r = RiskManager::validate_risk(10_000.0, 0.0075, 3.0, 2.0, 0.5, 0);
        assert!(r.is_err());
    }

    #[test]
    fn test_validate_risk_spread_exceeded() {
        let r = RiskManager::validate_risk(10_000.0, 0.0075, 1.0, 2.0, 2.0, 0);
        assert!(r.is_err());
    }

    #[test]
    fn test_validate_risk_max_position() {
        let r = RiskManager::validate_risk(10_000.0, 0.0075, 1.0, 2.0, 0.5, 1);
        assert!(r.is_err());
    }

    #[test]
    fn test_base_unit_zero_for_invalid_sl() {
        let bu = RiskManager::calculate_base_unit(10_000.0, 0.0075, 0.0);
        assert!((bu - 0.0).abs() < 1e-6);
    }
}
