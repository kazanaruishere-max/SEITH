// L3 - State Manager
// Manage trading state machine with SQLite persistence

use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TradingState {
    NormalRegulerMode,
    NewsAnomalyMode,
    CrisisAdaptation,
    ForceClose,
}

impl fmt::Display for TradingState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NormalRegulerMode => write!(f, "NORMAL_REGULER_MODE"),
            Self::NewsAnomalyMode => write!(f, "NEWS_ANOMALY_MODE"),
            Self::CrisisAdaptation => write!(f, "CRISIS_ADAPTATION"),
            Self::ForceClose => write!(f, "FORCE_CLOSE"),
        }
    }
}

#[derive(Debug)]
pub enum StateTransition {
    Allowed,
    Blocked(&'static str),
}

pub struct StateManager {
    current_state: TradingState,
}

impl Default for StateManager {
    fn default() -> Self {
        Self::new()
    }
}

impl StateManager {
    pub fn new() -> Self {
        log::info!("StateManager: initial state = NORMAL_REGULER_MODE");
        Self {
            current_state: TradingState::NormalRegulerMode,
        }
    }

    pub fn get_state(&self) -> TradingState {
        self.current_state
    }

    pub fn check_transition(&self, target: TradingState) -> StateTransition {
        use TradingState::*;
        match (self.current_state, target) {
            (NormalRegulerMode, NewsAnomalyMode) => StateTransition::Allowed,
            (NormalRegulerMode, CrisisAdaptation) => StateTransition::Allowed,
            (NormalRegulerMode, ForceClose) => StateTransition::Allowed,
            (NewsAnomalyMode, NormalRegulerMode) => StateTransition::Allowed,
            (NewsAnomalyMode, ForceClose) => StateTransition::Allowed,
            (CrisisAdaptation, NormalRegulerMode) => StateTransition::Allowed,
            (ForceClose, NormalRegulerMode) => StateTransition::Allowed,
            _ => StateTransition::Blocked("invalid state transition"),
        }
    }

    pub fn set_state(&mut self, target: TradingState) -> bool {
        match self.check_transition(target) {
            StateTransition::Allowed => {
                log::info!("State: {} → {}", self.current_state, target);
                self.current_state = target;
                true
            }
            StateTransition::Blocked(reason) => {
                log::warn!(
                    "State transition blocked: {} → {} ({})",
                    self.current_state,
                    target,
                    reason
                );
                false
            }
        }
    }

    pub fn is_normal(&self) -> bool {
        matches!(self.current_state, TradingState::NormalRegulerMode)
    }

    pub fn is_news(&self) -> bool {
        matches!(self.current_state, TradingState::NewsAnomalyMode)
    }

    pub fn is_crisis(&self) -> bool {
        matches!(self.current_state, TradingState::CrisisAdaptation)
    }

    pub fn is_force_close(&self) -> bool {
        matches!(self.current_state, TradingState::ForceClose)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_initial_state() {
        let sm = StateManager::new();
        assert!(matches!(sm.get_state(), TradingState::NormalRegulerMode));
    }

    #[test]
    fn test_normal_to_news() {
        let mut sm = StateManager::new();
        assert!(sm.set_state(TradingState::NewsAnomalyMode));
        assert!(sm.is_news());
    }

    #[test]
    fn test_normal_to_crisis() {
        let mut sm = StateManager::new();
        assert!(sm.set_state(TradingState::CrisisAdaptation));
        assert!(sm.is_crisis());
    }

    #[test]
    fn test_news_to_normal() {
        let mut sm = StateManager::new();
        sm.set_state(TradingState::NewsAnomalyMode);
        assert!(sm.set_state(TradingState::NormalRegulerMode));
        assert!(sm.is_normal());
    }

    #[test]
    fn test_blocked_crisis_to_news() {
        let mut sm = StateManager::new();
        sm.set_state(TradingState::CrisisAdaptation);
        assert!(!sm.set_state(TradingState::NewsAnomalyMode));
        assert!(sm.is_crisis());
    }

    #[test]
    fn test_blocked_force_close_to_news() {
        let mut sm = StateManager::new();
        sm.set_state(TradingState::ForceClose);
        assert!(!sm.set_state(TradingState::NewsAnomalyMode));
    }
}
