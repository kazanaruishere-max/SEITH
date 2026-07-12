use crate::core::l0_infra::session_filter::SessionPhase;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum SystemState {
    Boot,
    Open,
    Normal,
    Lunch,
    PowerHour,
    Close,
    SessionClosed,
    NoTrade,
    Crisis,
    DayOff,
}

impl SystemState {
    pub fn from_phase(phase: SessionPhase) -> Self {
        match phase {
            SessionPhase::Open => SystemState::Open,
            SessionPhase::Normal => SystemState::Normal,
            SessionPhase::Lunch => SystemState::Lunch,
            SessionPhase::PowerHour => SystemState::PowerHour,
            SessionPhase::Close => SystemState::Close,
            SessionPhase::Closed => SystemState::SessionClosed,
        }
    }

    pub fn can_trade(&self) -> bool {
        matches!(self, SystemState::Normal | SystemState::PowerHour)
    }
}

#[derive(Debug, Clone)]
pub struct StateManager {
    pub state: SystemState,
    pub skip_count: u32,
    pub crisis_standby_remaining: u32,
}

impl StateManager {
    pub fn new() -> Self {
        Self {
            state: SystemState::Boot,
            skip_count: 0,
            crisis_standby_remaining: 0,
        }
    }

    pub fn transition_to(&mut self, new_state: SystemState) {
        log::info!("[State] {:?} -> {:?}", self.state, new_state);
        self.state = new_state;
    }

    pub fn is_crisis(&self) -> bool {
        self.skip_count >= 3
    }

    pub fn is_ceiling(&self) -> bool {
        self.skip_count >= 5
    }

    pub fn is_canary(&self) -> bool {
        !self.can_trade()
    }

    pub fn can_trade(&self) -> bool {
        self.state.can_trade() && self.crisis_standby_remaining == 0
    }

    pub fn increment_skip(&mut self, is_signal_reject: bool) {
        if is_signal_reject {
            self.skip_count += 1;
            log::debug!("[AntiParalysis] skip_count -> {} (signal reject)", self.skip_count);
            if self.is_crisis() {
                self.state = SystemState::Crisis;
            }
            if self.is_ceiling() {
                log::warn!("[AntiParalysis] CEILING reached. Resetting, standby 1 session.");
                self.skip_count = 0;
                self.state = SystemState::SessionClosed;
                self.crisis_standby_remaining = 1;
            }
        }
    }

    pub fn reset_skip(&mut self) {
        self.skip_count = 0;
        self.state = SystemState::Normal;
        log::debug!("[AntiParalysis] skip_count reset to 0 after successful trade");
    }

    pub fn decrement_standby(&mut self) {
        if self.crisis_standby_remaining > 0 {
            self.crisis_standby_remaining -= 1;
            if self.crisis_standby_remaining == 0 {
                log::info!("[AntiParalysis] Standby session complete. Returning to Normal.");
                self.state = SystemState::Normal;
            }
        }
    }

    pub fn ofs_threshold(&self) -> f64 {
        if self.state == SystemState::Crisis {
            crate::config::thresholds::OFS_CRISIS_RELAX
        } else if self.state == SystemState::PowerHour {
            crate::config::thresholds::OFS_POWER_HOUR
        } else {
            crate::config::thresholds::OFS_NORMAL
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_initial_state() {
        let sm = StateManager::new();
        assert_eq!(sm.state, SystemState::Boot);
        assert_eq!(sm.skip_count, 0);
    }

    #[test]
    fn test_can_trade_normal() {
        let mut sm = StateManager::new();
        sm.state = SystemState::Normal;
        assert!(sm.can_trade());
    }

    #[test]
    fn test_can_trade_lunch_blocked() {
        let mut sm = StateManager::new();
        sm.state = SystemState::Lunch;
        assert!(!sm.can_trade());
    }

    #[test]
    fn test_increment_skip_signal_reject() {
        let mut sm = StateManager::new();
        sm.increment_skip(true);
        assert_eq!(sm.skip_count, 1);
    }

    #[test]
    fn test_increment_skip_market_condition() {
        let mut sm = StateManager::new();
        sm.increment_skip(false);
        assert_eq!(sm.skip_count, 0);
    }

    #[test]
    fn test_crisis_at_3() {
        let mut sm = StateManager::new();
        for _ in 0..3 { sm.increment_skip(true); }
        assert!(sm.is_crisis());
        assert_eq!(sm.state, SystemState::Crisis);
    }

    #[test]
    fn test_ceiling_resets_at_5() {
        let mut sm = StateManager::new();
        for _ in 0..5 { sm.increment_skip(true); }
        assert_eq!(sm.skip_count, 0);
        assert_eq!(sm.crisis_standby_remaining, 1);
        assert_eq!(sm.state, SystemState::SessionClosed);
    }

    #[test]
    fn test_ofs_threshold_crisis() {
        let mut sm = StateManager::new();
        sm.state = SystemState::Crisis;
        assert!((sm.ofs_threshold() - 2.0).abs() < 1e-6);
    }

    #[test]
    fn test_ofs_threshold_power_hour() {
        let mut sm = StateManager::new();
        sm.state = SystemState::PowerHour;
        assert!((sm.ofs_threshold() - 2.0).abs() < 1e-6);
    }

    #[test]
    fn test_ofs_threshold_normal() {
        let mut sm = StateManager::new();
        sm.state = SystemState::Normal;
        assert!((sm.ofs_threshold() - 3.0).abs() < 1e-6);
    }

    #[test]
    fn test_reset_skip() {
        let mut sm = StateManager::new();
        sm.skip_count = 3;
        sm.state = SystemState::Crisis;
        sm.reset_skip();
        assert_eq!(sm.skip_count, 0);
        assert_eq!(sm.state, SystemState::Normal);
    }

    #[test]
    fn test_from_phase_mapping() {
        assert_eq!(SystemState::from_phase(SessionPhase::Open), SystemState::Open);
        assert_eq!(SystemState::from_phase(SessionPhase::Normal), SystemState::Normal);
        assert_eq!(SystemState::from_phase(SessionPhase::Lunch), SystemState::Lunch);
        assert_eq!(SystemState::from_phase(SessionPhase::PowerHour), SystemState::PowerHour);
        assert_eq!(SystemState::from_phase(SessionPhase::Close), SystemState::Close);
        assert_eq!(SystemState::from_phase(SessionPhase::Closed), SystemState::SessionClosed);
    }

    #[test]
    fn test_decrement_standby() {
        let mut sm = StateManager::new();
        sm.crisis_standby_remaining = 1;
        sm.state = SystemState::SessionClosed;
        sm.decrement_standby();
        assert_eq!(sm.crisis_standby_remaining, 0);
        assert_eq!(sm.state, SystemState::Normal);
    }

    #[test]
    fn test_standby_noop_when_zero() {
        let mut sm = StateManager::new();
        sm.state = SystemState::Normal;
        sm.decrement_standby();
        assert_eq!(sm.state, SystemState::Normal);
    }
}
