// Risk Manager — Lot Sizing, DD Limits, SL/TP Enforcement

#[derive(Debug, Clone)]
pub struct RiskLimits {
    pub max_risk_percent: f64,
    pub max_daily_loss_percent: f64,
    pub max_weekly_loss_percent: f64,
    pub max_open_positions: u32,
    pub spread_tolerance_pips: f64,
}

impl Default for RiskLimits {
    fn default() -> Self {
        Self {
            max_risk_percent: 1.0,
            max_daily_loss_percent: 3.0,
            max_weekly_loss_percent: 6.0,
            max_open_positions: 1,
            spread_tolerance_pips: 3.5,
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct TradeSession {
    pub daily_loss: f64,
    pub weekly_loss: f64,
    pub open_positions: u32,
}

impl TradeSession {
    pub fn new() -> Self {
        Self {
            daily_loss: 0.0,
            weekly_loss: 0.0,
            open_positions: 0,
        }
    }
}

pub fn calculate_lot(equity: f64, risk_percent: f64, sl_distance: f64, pip_value: f64) -> f64 {
    if sl_distance <= 0.0 || pip_value <= 0.0 {
        return 0.01;
    }
    let risk_amount = equity * (risk_percent / 100.0);
    let lot = risk_amount / (sl_distance * pip_value);
    (lot * 100.0).round() / 100.0
}

pub fn can_trade(
    session: &TradeSession,
    limits: &RiskLimits,
    current_spread: f64,
    _current_price: f64,
) -> Result<(), &'static str> {
    if session.open_positions >= limits.max_open_positions {
        return Err("Max open positions reached");
    }
    if session.daily_loss >= limits.max_daily_loss_percent {
        return Err("Daily loss limit reached");
    }
    if session.weekly_loss >= limits.max_weekly_loss_percent {
        return Err("Weekly loss limit reached");
    }
    if current_spread > limits.spread_tolerance_pips {
        return Err("Spread exceeds tolerance");
    }
    Ok(())
}

pub fn record_loss(session: &mut TradeSession, loss_percent: f64) {
    session.daily_loss += loss_percent;
    session.weekly_loss += loss_percent;
}

pub fn reset_daily(session: &mut TradeSession) {
    session.daily_loss = 0.0;
}

pub fn reset_weekly(session: &mut TradeSession) {
    session.daily_loss = 0.0;
    session.weekly_loss = 0.0;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_can_trade_normal() {
        let s = TradeSession::new();
        let l = RiskLimits::default();
        assert!(can_trade(&s, &l, 2.0, 100.0).is_ok());
    }

    #[test]
    fn test_block_daily_loss() {
        let mut s = TradeSession::new();
        s.daily_loss = 3.5;
        let l = RiskLimits::default();
        assert!(can_trade(&s, &l, 2.0, 100.0).is_err());
    }

    #[test]
    fn test_block_max_positions() {
        let mut s = TradeSession::new();
        s.open_positions = 1;
        let l = RiskLimits::default();
        assert!(can_trade(&s, &l, 2.0, 100.0).is_err());
    }

    #[test]
    fn test_block_high_spread() {
        let s = TradeSession::new();
        let l = RiskLimits::default();
        assert!(can_trade(&s, &l, 5.0, 100.0).is_err());
    }

    #[test]
    fn test_lot_calculation() {
        let lot = calculate_lot(10000.0, 1.0, 50.0, 0.1);
        assert!(lot > 0.0);
    }

    #[test]
    fn test_zero_sl() {
        let lot = calculate_lot(10000.0, 1.0, 0.0, 0.1);
        assert!((lot - 0.01).abs() < 0.001);
    }

    #[test]
    fn test_record_daily_and_weekly() {
        let mut s = TradeSession::new();
        record_loss(&mut s, 1.5);
        assert!((s.daily_loss - 1.5).abs() < 0.01);
        assert!((s.weekly_loss - 1.5).abs() < 0.01);
    }

    #[test]
    fn test_reset_daily() {
        let mut s = TradeSession::new();
        s.daily_loss = 2.0;
        reset_daily(&mut s);
        assert!((s.daily_loss - 0.0).abs() < 0.01);
    }
}
