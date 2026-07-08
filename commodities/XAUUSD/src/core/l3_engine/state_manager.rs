// L3 - State Manager
// Manage trading state (NORMAL_MODE vs NEWS_MODE)
// Stub only - no implementation yet

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TradingState {
    NormalMode,
    NewsMode,
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
        Self {
            current_state: TradingState::NormalMode,
        }
    }

    pub fn get_state(&self) -> TradingState {
        self.current_state
    }

    pub fn set_state(&mut self, state: TradingState) {
        log::info!("State changed: {:?} -> {:?}", self.current_state, state);
        self.current_state = state;
    }
}
