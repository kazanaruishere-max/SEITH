// L2 - Regime Switch
// Controls NORMAL ↔ NEWS_ANOMALY mode transition

use crate::core::l3_engine::state_manager::{StateManager, TradingState};

pub fn activate_news_mode(state: &mut StateManager) -> bool {
    state.set_state(TradingState::NewsAnomalyMode)
}

pub fn deactivate_news_mode(state: &mut StateManager) -> bool {
    state.set_state(TradingState::NormalRegulerMode)
}

pub fn should_disable_technical_engine(state: &StateManager) -> bool {
    state.is_news()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::l3_engine::state_manager::StateManager;

    #[test]
    fn test_activate_news() {
        let mut sm = StateManager::new();
        assert!(activate_news_mode(&mut sm));
        assert!(sm.is_news());
    }

    #[test]
    fn test_deactivate_news() {
        let mut sm = StateManager::new();
        activate_news_mode(&mut sm);
        assert!(deactivate_news_mode(&mut sm));
        assert!(sm.is_normal());
    }

    #[test]
    fn test_disable_technical() {
        let mut sm = StateManager::new();
        assert!(!should_disable_technical_engine(&sm));
        activate_news_mode(&mut sm);
        assert!(should_disable_technical_engine(&sm));
    }
}
