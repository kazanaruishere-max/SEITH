use crate::core::l0_infra::macro_filter::MacroFilter;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum GateResult {
    Pass,
    PassReduceLot,
    Block,
}

pub struct MacroGate;

impl MacroGate {
    pub fn new() -> Self {
        Self
    }

    pub fn evaluate(&self, filter: &MacroFilter) -> GateResult {
        let verdict = filter.verdict;
        match verdict {
            crate::core::l0_infra::macro_filter::MacroVerdict::Red => GateResult::Block,
            crate::core::l0_infra::macro_filter::MacroVerdict::Orange => GateResult::PassReduceLot,
            crate::core::l0_infra::macro_filter::MacroVerdict::Warning => GateResult::PassReduceLot,
            crate::core::l0_infra::macro_filter::MacroVerdict::Green => GateResult::Pass,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::l0_infra::macro_filter::{MacroEvent, MacroFilter, MacroVerdict};

    #[test]
    fn test_green_pass() {
        let mut filter = MacroFilter::new();
        filter.verdict = MacroVerdict::Green;
        assert_eq!(MacroGate::evaluate(&MacroGate, &filter), GateResult::Pass);
    }

    #[test]
    fn test_red_block() {
        let mut filter = MacroFilter::new();
        filter.verdict = MacroVerdict::Red;
        assert_eq!(MacroGate::evaluate(&MacroGate, &filter), GateResult::Block);
    }

    #[test]
    fn test_orange_reduce_lot() {
        let mut filter = MacroFilter::new();
        filter.verdict = MacroVerdict::Orange;
        assert_eq!(MacroGate::evaluate(&MacroGate, &filter), GateResult::PassReduceLot);
    }

    #[test]
    fn test_warning_reduce_lot() {
        let mut filter = MacroFilter::new();
        filter.verdict = MacroVerdict::Warning;
        assert_eq!(MacroGate::evaluate(&MacroGate, &filter), GateResult::PassReduceLot);
    }
}
