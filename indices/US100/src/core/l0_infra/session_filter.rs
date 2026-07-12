use crate::config::settings;
use chrono::{NaiveTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum SessionPhase {
    Open,
    Normal,
    Lunch,
    PowerHour,
    Close,
    Closed,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionState {
    pub phase: SessionPhase,
    pub in_session: bool,
    pub gap_detected: bool,
    pub has_open_position: bool,
    pub should_short_circuit: bool,
    pub gap_open_price: Option<f64>,
    pub prev_close_price: Option<f64>,
}

impl SessionState {
    pub fn new() -> Self {
        Self {
            phase: SessionPhase::Closed,
            in_session: false,
            gap_detected: false,
            has_open_position: false,
            should_short_circuit: false,
            gap_open_price: None,
            prev_close_price: None,
        }
    }

    pub fn detect_phase(now: NaiveTime) -> SessionPhase {
        if now < settings::OPEN_END { SessionPhase::Open }
        else if now < settings::LUNCH_START { SessionPhase::Normal }
        else if now < settings::LUNCH_END { SessionPhase::Lunch }
        else if now < settings::POWER_HOUR_START { SessionPhase::Normal }
        else if now < settings::CLOSE_START { SessionPhase::PowerHour }
        else if now < settings::SESSION_CLOSE { SessionPhase::Close }
        else { SessionPhase::Closed }
    }

    pub fn is_block_phase(&self) -> bool {
        self.phase == SessionPhase::Lunch || self.phase == SessionPhase::Close
    }

    pub fn check_gap(&mut self, open_price: f64, prev_close: f64) {
        self.gap_open_price = Some(open_price);
        self.prev_close_price = Some(prev_close);
        let gap_pct = ((open_price - prev_close) / prev_close).abs() * 100.0;
        self.gap_detected = gap_pct > settings::GAP_THRESHOLD_PCT;
        if self.gap_detected {
            log::info!("[Session] Gap detected: {:.2}% (threshold: {}%)", gap_pct, settings::GAP_THRESHOLD_PCT);
        }
    }

    pub fn update(&mut self) {
        let now = Utc::now().time();
        self.in_session = now >= settings::SESSION_OPEN && now < settings::SESSION_CLOSE;
        if !self.has_open_position {
            self.phase = Self::detect_phase(now);
            self.should_short_circuit = false;
        } else {
            self.should_short_circuit = true;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::NaiveTime;

    #[test]
    fn test_detect_phase_open() {
        let t = NaiveTime::from_hms_opt(14, 45, 0).unwrap();
        assert_eq!(SessionState::detect_phase(t), SessionPhase::Open);
    }

    #[test]
    fn test_detect_phase_normal() {
        let t = NaiveTime::from_hms_opt(15, 30, 0).unwrap();
        assert_eq!(SessionState::detect_phase(t), SessionPhase::Normal);
    }

    #[test]
    fn test_detect_phase_lunch() {
        let t = NaiveTime::from_hms_opt(17, 0, 0).unwrap();
        assert_eq!(SessionState::detect_phase(t), SessionPhase::Lunch);
    }

    #[test]
    fn test_detect_phase_power_hour() {
        let t = NaiveTime::from_hms_opt(20, 0, 0).unwrap();
        assert_eq!(SessionState::detect_phase(t), SessionPhase::PowerHour);
    }

    #[test]
    fn test_detect_phase_close() {
        let t = NaiveTime::from_hms_opt(20, 45, 0).unwrap();
        assert_eq!(SessionState::detect_phase(t), SessionPhase::Close);
    }

    #[test]
    fn test_detect_phase_closed() {
        let t = NaiveTime::from_hms_opt(21, 30, 0).unwrap();
        assert_eq!(SessionState::detect_phase(t), SessionPhase::Closed);
    }

    #[test]
    fn test_gap_detected() {
        let mut state = SessionState::new();
        state.check_gap(100.5, 100.0);
        assert!(state.gap_detected);
    }

    #[test]
    fn test_gap_not_detected() {
        let mut state = SessionState::new();
        state.check_gap(100.1, 100.0);
        assert!(!state.gap_detected);
    }

    #[test]
    fn test_block_phases() {
        let mut state = SessionState::new();
        state.phase = SessionPhase::Lunch;
        assert!(state.is_block_phase());
        state.phase = SessionPhase::Close;
        assert!(state.is_block_phase());
        state.phase = SessionPhase::Normal;
        assert!(!state.is_block_phase());
    }

    #[test]
    fn test_short_circuit_with_open_position() {
        let mut state = SessionState::new();
        state.has_open_position = true;
        state.update();
        assert!(state.should_short_circuit);
    }

    #[test]
    fn test_no_short_circuit_without_position() {
        let mut state = SessionState::new();
        state.has_open_position = false;
        state.update();
        assert!(!state.should_short_circuit);
    }
}
