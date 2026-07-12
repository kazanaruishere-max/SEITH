use crate::core::l3_engine::state_manager::StateManager;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum CrisisAction {
    None,
    RelaxOfs,
    CeilingReset,
}

pub struct AntiParalysis;

impl AntiParalysis {
    pub fn evaluate(state: &StateManager) -> CrisisAction {
        if state.skip_count >= 5 {
            CrisisAction::CeilingReset
        } else if state.skip_count >= 3 {
            CrisisAction::RelaxOfs
        } else {
            CrisisAction::None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::l3_engine::state_manager::StateManager;

    #[test]
    fn test_none_below_3() {
        let sm = StateManager::new();
        assert_eq!(AntiParalysis::evaluate(&sm), CrisisAction::None);
    }

    #[test]
    fn test_relax_ofs_at_3() {
        let mut sm = StateManager::new();
        sm.skip_count = 3;
        assert_eq!(AntiParalysis::evaluate(&sm), CrisisAction::RelaxOfs);
    }

    #[test]
    fn test_ceiling_at_5() {
        let mut sm = StateManager::new();
        sm.skip_count = 5;
        assert_eq!(AntiParalysis::evaluate(&sm), CrisisAction::CeilingReset);
    }
}
