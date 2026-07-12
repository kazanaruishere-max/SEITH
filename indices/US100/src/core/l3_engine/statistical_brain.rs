use crate::config::settings;
use crate::config::thresholds;
use crate::core::l3_engine::state_manager::StateManager;

pub struct SynthesisOutput {
    pub entry: f64,
    pub sl: f64,
    pub tp: f64,
    pub lot: f64,
    pub confidence: f64,
}

pub struct StatisticalBrain;

impl StatisticalBrain {
    pub fn new() -> Self {
        Self
    }

    pub fn calculate_lot(
        equity: f64,
        risk_pct: f64,
        sl_points: f64,
        confidence_mult: f64,
        mode_mult: f64,
        macro_mult: f64,
    ) -> f64 {
        let base_unit = equity * risk_pct / (sl_points * settings::POINT_VALUE);
        base_unit * confidence_mult * mode_mult * macro_mult
    }

    pub fn confidence_multiplier(_gate_pass: u32, hv_z: f64, state: &StateManager) -> f64 {
        if state.is_crisis() {
            return settings::CONFIDENCE_CRISIS;
        }
        let mut mult = settings::CONFIDENCE_5_5;
        if hv_z > thresholds::HV_SWEET_SPOT_MAX && hv_z <= thresholds::HV_ELEVATED_MAX {
            mult *= settings::CONFIDENCE_HV_15_20;
        }
        mult
    }

    pub fn mode_multiplier(state: &StateManager) -> f64 {
        if state.is_crisis() || state.state == crate::core::l3_engine::state_manager::SystemState::PowerHour {
            settings::MODE_MULT_SCALP
        } else {
            settings::MODE_MULT_SNIPER
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::l3_engine::state_manager::StateManager;

    #[test]
    fn test_calculate_lot_sniper() {
        let lot = StatisticalBrain::calculate_lot(10_000.0, 0.0075, 10.0, 1.0, 1.0, 1.0);
        assert!((lot - 7.5).abs() < 1e-6);
    }

    #[test]
    fn test_calculate_lot_scalp() {
        let lot = StatisticalBrain::calculate_lot(10_000.0, 0.0050, 8.0, 1.0, 0.67, 1.0);
        assert!((lot - 4.1875).abs() < 1e-4);
    }

    #[test]
    fn test_confidence_normal_5_5() {
        let sm = StateManager::new();
        let mult = StatisticalBrain::confidence_multiplier(5, 0.0, &sm);
        assert!((mult - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_confidence_hv_elevated() {
        let sm = StateManager::new();
        let mult = StatisticalBrain::confidence_multiplier(5, 1.8, &sm);
        assert!((mult - 0.75).abs() < 1e-6);
    }

    #[test]
    fn test_confidence_crisis() {
        let mut sm = StateManager::new();
        sm.skip_count = 3;
        let mult = StatisticalBrain::confidence_multiplier(5, 0.0, &sm);
        assert!((mult - 0.5).abs() < 1e-6);
    }

    #[test]
    fn test_mode_multiplier_normal() {
        let sm = StateManager::new();
        let mult = StatisticalBrain::mode_multiplier(&sm);
        assert!((mult - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_mode_multiplier_crisis() {
        let mut sm = StateManager::new();
        sm.skip_count = 3;
        let mult = StatisticalBrain::mode_multiplier(&sm);
        assert!((mult - 0.67).abs() < 1e-6);
    }
}
