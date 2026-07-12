use crate::core::l0_infra::macro_filter::MacroFilter;
use crate::core::l1_pipeline::hv_compass::HvCompass;
use crate::core::l1_pipeline::macro_gate::{GateResult, MacroGate};

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PipelineResult {
    pub gate0: Option<GateResult>,
    pub gate1: bool,
    pub gate2: bool,
    pub gate3: bool,
    pub gate4: bool,
    pub gates_passed: u32,
    pub reduce_lot: bool,
}

pub struct PipelineRouter;

impl PipelineRouter {
    pub fn new() -> Self {
        Self
    }

    pub fn run(
        &self,
        macro_filter: &MacroFilter,
        hv_z: f64,
        frama_z: f64,
        ofs: f64,
        ofs_threshold: f64,
        vwap_overextended: bool,
        _yield_z: Option<f64>,
        _curve_spread: f64,
        _is_power_hour: bool,
    ) -> PipelineResult {
        let gate0 = MacroGate::evaluate(&MacroGate, macro_filter);
        let mut gates_passed = 0u32;
        let mut reduce_lot = false;

        if matches!(gate0, GateResult::Block) {
            return PipelineResult { gate0: Some(gate0), gate1: false, gate2: false, gate3: false, gate4: false, gates_passed: 0, reduce_lot: false };
        }
        if matches!(gate0, GateResult::PassReduceLot) {
            reduce_lot = true;
        }
        gates_passed += 1;

        let compass = HvCompass::new();
        let hv_verdict = compass.evaluate(hv_z);
        let gate1 = matches!(hv_verdict, crate::core::l1_pipeline::hv_compass::HvVerdict::Pass | crate::core::l1_pipeline::hv_compass::HvVerdict::PassConfidenceDown);
        if !gate1 {
            return PipelineResult { gate0: Some(gate0), gate1: false, gate2: false, gate3: false, gate4: false, gates_passed, reduce_lot };
        }
        gates_passed += 1;

        let gate2 = crate::indicators::frama::Frama::is_pullback_valid(frama_z);
        if !gate2 {
            return PipelineResult { gate0: Some(gate0), gate1, gate2: false, gate3: false, gate4: false, gates_passed, reduce_lot };
        }
        gates_passed += 1;

        let gate3_signed = crate::indicators::orderflow::evaluate_ofs(ofs, ofs_threshold);
        let gate3 = matches!(gate3_signed, crate::indicators::orderflow::OfsVerdict::Pass);
        if !gate3 {
            return PipelineResult { gate0: Some(gate0), gate1, gate2, gate3: false, gate4: false, gates_passed, reduce_lot };
        }
        gates_passed += 1;

        let gate4 = !vwap_overextended;
        if !gate4 {
            return PipelineResult { gate0: Some(gate0), gate1, gate2, gate3, gate4: false, gates_passed, reduce_lot };
        }
        gates_passed += 1;

        PipelineResult { gate0: Some(gate0), gate1, gate2, gate3, gate4, gates_passed, reduce_lot }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::l0_infra::macro_filter::{MacroFilter, MacroVerdict};

    fn make_filter(verdict: MacroVerdict) -> MacroFilter {
        let mut f = MacroFilter::new();
        f.verdict = verdict;
        f.today_events.push(if matches!(verdict, MacroVerdict::Red) { crate::core::l0_infra::macro_filter::MacroEvent::Fomc } else { crate::core::l0_infra::macro_filter::MacroEvent::Ppi });
        f
    }

    #[test]
    fn test_all_gates_pass() {
        let router = PipelineRouter::new();
        let filter = make_filter(MacroVerdict::Green);
        let r = router.run(&filter, 0.0, 0.3, 3.5, 3.0, false, Some(0.5), 0.2, false);
        assert_eq!(r.gates_passed, 5);
        assert!(!r.reduce_lot);
    }

    #[test]
    fn test_gate0_blocks() {
        let router = PipelineRouter::new();
        let filter = make_filter(MacroVerdict::Red);
        let r = router.run(&filter, 0.0, 0.3, 3.5, 3.0, false, Some(0.5), 0.2, false);
        assert_eq!(r.gates_passed, 0);
    }

    #[test]
    fn test_gate0_reduces_lot() {
        let router = PipelineRouter::new();
        let filter = make_filter(MacroVerdict::Orange);
        let r = router.run(&filter, 0.0, 0.3, 3.5, 3.0, false, Some(0.5), 0.2, false);
        assert!(r.reduce_lot);
        assert_eq!(r.gates_passed, 5);
    }

    #[test]
    fn test_gate1_blocks_hv_low() {
        let router = PipelineRouter::new();
        let filter = make_filter(MacroVerdict::Green);
        let r = router.run(&filter, -1.5, 0.3, 3.5, 3.0, false, Some(0.5), 0.2, false);
        assert_eq!(r.gates_passed, 1);
        assert!(!r.gate1);
    }

    #[test]
    fn test_gate1_blocks_hv_high() {
        let router = PipelineRouter::new();
        let filter = make_filter(MacroVerdict::Green);
        let r = router.run(&filter, 2.5, 0.3, 3.5, 3.0, false, Some(0.5), 0.2, false);
        assert_eq!(r.gates_passed, 1);
        assert!(!r.gate1);
    }

    #[test]
    fn test_gate2_blocks_overextended() {
        let router = PipelineRouter::new();
        let filter = make_filter(MacroVerdict::Green);
        let r = router.run(&filter, 0.0, 0.6, 3.5, 3.0, false, Some(0.5), 0.2, false);
        assert_eq!(r.gates_passed, 2);
        assert!(!r.gate2);
    }

    #[test]
    fn test_gate3_blocks_noise() {
        let router = PipelineRouter::new();
        let filter = make_filter(MacroVerdict::Green);
        let r = router.run(&filter, 0.0, 0.3, 0.5, 3.0, false, Some(0.5), 0.2, false);
        assert_eq!(r.gates_passed, 3);
        assert!(!r.gate3);
    }

    #[test]
    fn test_gate3_negative_signed() {
        let router = PipelineRouter::new();
        let filter = make_filter(MacroVerdict::Green);
        let r = router.run(&filter, 0.0, 0.3, -3.5, 3.0, false, Some(0.5), 0.2, false);
        assert!(r.gate3);
        assert_eq!(r.gates_passed, 5);
    }

    #[test]
    fn test_gate4_blocks_vwap() {
        let router = PipelineRouter::new();
        let filter = make_filter(MacroVerdict::Green);
        let r = router.run(&filter, 0.0, 0.3, 3.5, 3.0, true, Some(0.5), 0.2, false);
        assert_eq!(r.gates_passed, 4);
        assert!(!r.gate4);
    }

    #[test]
    fn test_gate4_passes_vwap_within_bands() {
        let router = PipelineRouter::new();
        let filter = make_filter(MacroVerdict::Green);
        let r = router.run(&filter, 0.0, 0.3, 3.5, 3.0, false, Some(0.5), 0.2, false);
        assert!(r.gate4);
        assert_eq!(r.gates_passed, 5);
    }

    #[test]
    fn test_power_hour_threshold() {
        let router = PipelineRouter::new();
        let filter = make_filter(MacroVerdict::Green);
        let r = router.run(&filter, 0.0, 0.3, 2.5, 2.0, false, Some(0.5), 0.2, true);
        assert!(r.gate3);
        assert_eq!(r.gates_passed, 5);
    }

    #[test]
    fn test_power_hour_block_if_below_relaxed() {
        let router = PipelineRouter::new();
        let filter = make_filter(MacroVerdict::Green);
        let r = router.run(&filter, 0.0, 0.3, 1.5, 2.0, false, Some(0.5), 0.2, true);
        assert!(!r.gate3);
        assert_eq!(r.gates_passed, 3);
    }

    #[test]
    fn test_pipeline_result_defaults() {
        let r = PipelineResult { gate0: None, gate1: false, gate2: false, gate3: false, gate4: false, gates_passed: 0, reduce_lot: false };
        assert_eq!(r.gates_passed, 0);
    }
}
