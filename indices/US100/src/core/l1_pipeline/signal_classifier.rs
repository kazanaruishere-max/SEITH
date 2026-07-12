use crate::core::l1_pipeline::pipeline_router::PipelineResult;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SignalVerdict {
    Valid,
    Skip,
}

pub struct SignalClassifier;

impl SignalClassifier {
    pub fn new() -> Self {
        Self
    }

    pub fn classify(result: &PipelineResult, is_crisis: bool) -> SignalVerdict {
        if is_crisis {
            if result.gates_passed >= 5 || (result.gates_passed >= 4 && result.gate3) {
                return SignalVerdict::Valid;
            }
            return SignalVerdict::Skip;
        }
        if result.gates_passed == 5 {
            SignalVerdict::Valid
        } else {
            SignalVerdict::Skip
        }
    }

    pub fn should_increment_skip(verdict: SignalVerdict) -> bool {
        matches!(verdict, SignalVerdict::Skip)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::l1_pipeline::macro_gate::GateResult;
    use crate::core::l1_pipeline::pipeline_router::PipelineResult;

    fn make_result(gates_passed: u32, gate3: bool) -> PipelineResult {
        PipelineResult {
            gate0: Some(GateResult::Pass),
            gate1: gates_passed >= 2,
            gate2: gates_passed >= 3,
            gate3,
            gate4: gates_passed >= 5,
            gates_passed,
            reduce_lot: false,
        }
    }

    #[test]
    fn test_classify_valid_5_5() {
        let r = make_result(5, true);
        assert_eq!(SignalClassifier::classify(&r, false), SignalVerdict::Valid);
    }

    #[test]
    fn test_classify_skip_4_5() {
        let r = make_result(4, true);
        assert_eq!(SignalClassifier::classify(&r, false), SignalVerdict::Skip);
    }

    #[test]
    fn test_classify_skip_3_5() {
        let r = make_result(3, true);
        assert_eq!(SignalClassifier::classify(&r, false), SignalVerdict::Skip);
    }

    #[test]
    fn test_crisis_relax_5_5() {
        let r = make_result(5, true);
        assert_eq!(SignalClassifier::classify(&r, true), SignalVerdict::Valid);
    }

    #[test]
    fn test_crisis_relax_4_5_with_gate3() {
        let r = make_result(4, true);
        assert_eq!(SignalClassifier::classify(&r, true), SignalVerdict::Valid);
    }

    #[test]
    fn test_crisis_relax_4_5_no_gate3() {
        let r = make_result(4, false);
        assert_eq!(SignalClassifier::classify(&r, true), SignalVerdict::Skip);
    }

    #[test]
    fn test_crisis_relax_3_5() {
        let r = make_result(3, true);
        assert_eq!(SignalClassifier::classify(&r, true), SignalVerdict::Skip);
    }

    #[test]
    fn test_should_increment_skip_valid() {
        assert!(!SignalClassifier::should_increment_skip(SignalVerdict::Valid));
    }

    #[test]
    fn test_should_increment_skip_skip() {
        assert!(SignalClassifier::should_increment_skip(SignalVerdict::Skip));
    }
}
