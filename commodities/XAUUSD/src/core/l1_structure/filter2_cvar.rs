// Filter 2 — CVaR Risk Engine
// Price Velocity check: ≥ 250 pts/s → HIGH TAIL RISK (cut lot 50-75%)
// < 150 pts/s → NORMAL (100% lot)

use crate::core::l1_structure::filter1_bayesian::BayesianDecision;

const HIGH_VELOCITY_THRESHOLD: f64 = 250.0;
const NORMAL_VELOCITY_THRESHOLD: f64 = 150.0;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VelocityRegime {
    HighTailRisk,
    Normal,
}

#[derive(Debug, Clone)]
pub struct CvarResult {
    pub velocity: f64,
    pub regime: VelocityRegime,
    pub lot_multiplier: f64,
    pub passed: bool,
}

pub fn calculate_velocity(price_change: f64, time_secs: f64) -> f64 {
    if time_secs <= 0.0 {
        return 0.0;
    }
    (price_change / time_secs).abs()
}

pub fn evaluate_cvar(velocity: f64, decision: &BayesianDecision) -> CvarResult {
    if matches!(decision, BayesianDecision::Block) {
        return CvarResult {
            velocity,
            regime: VelocityRegime::Normal,
            lot_multiplier: 0.0,
            passed: false,
        };
    }

    let (regime, lot_multiplier) = if velocity >= HIGH_VELOCITY_THRESHOLD {
        (VelocityRegime::HighTailRisk, 0.375)
    } else if velocity < NORMAL_VELOCITY_THRESHOLD {
        (VelocityRegime::Normal, 1.0)
    } else {
        (VelocityRegime::Normal, 0.75)
    };

    CvarResult {
        velocity,
        regime,
        lot_multiplier,
        passed: true,
    }
}

pub fn adjust_lot(base_lot: f64, multiplier: f64) -> f64 {
    let adjusted = base_lot * multiplier;
    (adjusted * 100.0).round() / 100.0
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::l1_structure::filter1_bayesian::BayesianDecision;

    #[test]
    fn test_high_velocity_cuts_lot() {
        let d = evaluate_cvar(300.0, &BayesianDecision::Tier2Tactical);
        assert!(matches!(d.regime, VelocityRegime::HighTailRisk));
        assert!((d.lot_multiplier - 0.375).abs() < 0.01);
    }

    #[test]
    fn test_normal_velocity() {
        let d = evaluate_cvar(100.0, &BayesianDecision::Tier2Tactical);
        assert!(matches!(d.regime, VelocityRegime::Normal));
        assert!((d.lot_multiplier - 1.0).abs() < 0.01);
    }

    #[test]
    fn test_blocked_decision_zero_lot() {
        let d = evaluate_cvar(100.0, &BayesianDecision::Block);
        assert!((d.lot_multiplier - 0.0).abs() < 0.01);
        assert!(!d.passed);
    }

    #[test]
    fn test_adjust_lot() {
        let adjusted = adjust_lot(0.01, 0.375);
        assert!((adjusted - 0.0038).abs() < 0.01);
    }

    #[test]
    fn test_velocity_calculation() {
        let v = calculate_velocity(5.0, 0.02);
        assert!((v - 250.0).abs() < 0.01);
    }
}
