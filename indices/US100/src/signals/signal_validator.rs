use crate::core::l1_pipeline::pipeline_router::PipelineResult;
use crate::core::l1_pipeline::signal_classifier::{SignalClassifier, SignalVerdict};

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ValidationResult {
    Valid,
    Invalid { reason: &'static str },
}

pub struct SignalValidator;

impl SignalValidator {
    pub fn new() -> Self {
        Self
    }

    pub fn validate(result: &PipelineResult, is_crisis: bool) -> ValidationResult {
        let verdict = SignalClassifier::classify(result, is_crisis);
        match verdict {
            SignalVerdict::Valid => ValidationResult::Valid,
            SignalVerdict::Skip => {
                if is_crisis && result.gates_passed >= 4 && result.gate3 {
                    ValidationResult::Valid
                } else {
                    ValidationResult::Invalid { reason: "Insufficient gates passed" }
                }
            }
        }
    }

    pub fn is_sniper_valid(result: &PipelineResult, is_crisis: bool) -> bool {
        if is_crisis {
            return false;
        }
        result.gates_passed == 5 && !result.reduce_lot
    }

    pub fn is_scalp_valid(result: &PipelineResult, is_crisis: bool) -> bool {
        if is_crisis {
            return result.gates_passed >= 4 && result.gate3;
        }
        result.gates_passed == 5
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::l1_pipeline::macro_gate::GateResult;
    use crate::core::l1_pipeline::pipeline_router::PipelineResult;

    fn make_result(gates_passed: u32, gate3: bool, reduce_lot: bool) -> PipelineResult {
        PipelineResult {
            gate0: Some(GateResult::Pass),
            gate1: gates_passed >= 2,
            gate2: gates_passed >= 3,
            gate3,
            gate4: gates_passed >= 5,
            gates_passed, reduce_lot,
        }
    }

    #[test]
    fn test_validate_valid() {
        let r = make_result(5, true, false);
        assert_eq!(SignalValidator::validate(&r, false), ValidationResult::Valid);
    }

    #[test]
    fn test_validate_invalid() {
        let r = make_result(3, true, false);
        assert!(matches!(SignalValidator::validate(&r, false), ValidationResult::Invalid { .. }));
    }

    #[test]
    fn test_is_sniper_valid() {
        let r = make_result(5, true, false);
        assert!(SignalValidator::is_sniper_valid(&r, false));
        assert!(!SignalValidator::is_sniper_valid(&r, true));
    }

    #[test]
    fn test_sniper_invalid_with_reduce_lot() {
        let r = make_result(5, true, true);
        assert!(!SignalValidator::is_sniper_valid(&r, false));
    }

    #[test]
    fn test_scalp_valid_crisis() {
        let r = make_result(4, true, false);
        assert!(SignalValidator::is_scalp_valid(&r, true));
    }

    #[test]
    fn test_scalp_invalid_crisis_no_gate3() {
        let r = make_result(4, false, false);
        assert!(!SignalValidator::is_scalp_valid(&r, true));
    }
}
