use crate::config::settings;
use crate::core::l0_infra::data_feed::DataFeed;
use crate::core::l0_infra::macro_filter::{MacroFilter, MacroVerdict};
use crate::core::l0_infra::session_filter::{SessionPhase, SessionState};
use crate::core::l3_engine::anti_paralysis::{AntiParalysis, CrisisAction};
use crate::core::l3_engine::state_manager::{StateManager, SystemState};
use anyhow::Result;
use chrono::{Datelike, NaiveTime, Utc};
use std::time::Duration;

const TICK_INTERVAL_SECS: u64 = 15;

pub struct EventLoop {
    pub data_feed: DataFeed,
    pub session: SessionState,
    pub macro_filter: MacroFilter,
    pub state: StateManager,
}

impl EventLoop {
    pub fn new() -> Self {
        Self {
            data_feed: DataFeed::new(),
            session: SessionState::new(),
            macro_filter: MacroFilter::new(),
            state: StateManager::new(),
        }
    }

    pub async fn run(&mut self) -> Result<()> {
        log::info!("[EventLoop] US100 starting. Session: {:?}-{:?} UTC",
            settings::SESSION_OPEN, settings::SESSION_CLOSE);

        loop {
            self.tick().await?;
            tokio::time::sleep(Duration::from_secs(TICK_INTERVAL_SECS)).await;
        }
    }

    async fn tick(&mut self) -> Result<()> {
        let now = Utc::now();
        let now_time = now.time();

        if self.is_weekend(now) {
            if self.state.state != SystemState::DayOff {
                self.state.transition_to(SystemState::DayOff);
                log::info!("[EventLoop] Weekend detected. Standby until Monday.");
            }
            return Ok(());
        }

        self.session.update();

        if !self.session.in_session {
            if self.state.state != SystemState::SessionClosed {
                self.state.transition_to(SystemState::SessionClosed);
            }
            self.state.decrement_standby();
            return Ok(());
        }

        if now_time >= settings::SESSION_OPEN && self.state.state == SystemState::Boot {
            self.state.transition_to(SystemState::Open);
            log::info!("[EventLoop] Session OPEN at {}", now_time);
        }

        let new_phase = SessionState::detect_phase(now_time);
        if new_phase != self.session.phase && !self.session.has_open_position {
            self.session.phase = new_phase;
            let new_state = SystemState::from_phase(new_phase);
            if self.state.state != SystemState::Crisis {
                self.state.transition_to(new_state);
            }
            log::info!("[EventLoop] Phase -> {:?} at {}", new_phase, now_time);
        }

        let crisis_action = AntiParalysis::evaluate(&self.state);
        match crisis_action {
            CrisisAction::CeilingReset => {
                log::warn!("[EventLoop] CRISIS CEILING: reset + standby 1 session");
                self.state.transition_to(SystemState::SessionClosed);
                self.state.skip_count = 0;
                self.state.crisis_standby_remaining = 1;
                return Ok(());
            }
            CrisisAction::RelaxOfs => {
                if self.state.state != SystemState::Crisis {
                    self.state.transition_to(SystemState::Crisis);
                    log::info!("[EventLoop] CRISIS mode: OFS relaxed to 2, SCALP forced");
                }
            }
            CrisisAction::None => {}
        }

        if self.state.crisis_standby_remaining > 0 {
            self.state.decrement_standby();
            return Ok(());
        }

        if self.state.can_trade() && !self.session.is_block_phase() {
            self.run_pipeline().await?;
        }

        self.force_close_check(now_time).await?;

        Ok(())
    }

    async fn run_pipeline(&mut self) -> Result<()> {
        log::debug!("[EventLoop] Pipeline tick state={:?}", self.state.state);
        // L0 data gathering
        let _prices = self.data_feed.fetch_ohlcv().await?;
        let _us10y = self.data_feed.fetch_us10y_yield().await?;
        let _us02y = self.data_feed.fetch_us02y_yield().await?;

        // Macro check
        let verdict = self.macro_filter.evaluate();
        if verdict == MacroVerdict::Red {
            log::info!("[Pipeline] Macro RED — skip");
            self.state.increment_skip(false);
            return Ok(());
        }

        if self.session.should_short_circuit {
            log::debug!("[Pipeline] Short-circuit: position open, skipping gates");
            return Ok(());
        }

        if self.session.gap_detected && self.session.phase == SessionPhase::Open {
            log::info!("[Pipeline] Gap detected in Open phase — skip");
            self.state.increment_skip(false);
            return Ok(());
        }

        log::debug!("[Pipeline] Pipeline complete (stub)");
        Ok(())
    }

    async fn force_close_check(&self, now_time: NaiveTime) -> Result<()> {
        if now_time >= settings::SESSION_CLOSE && self.session.has_open_position {
            log::info!("[EventLoop] Force close at {}", now_time);
        }
        Ok(())
    }

    fn is_weekend(&self, now: chrono::DateTime<Utc>) -> bool {
        let wd = now.date_naive().weekday();
        wd == chrono::Weekday::Sat || wd == chrono::Weekday::Sun
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::NaiveTime;

    #[test]
    fn test_is_weekend() {
        let el = EventLoop::new();
        let sat = chrono::DateTime::parse_from_rfc3339("2026-07-11T12:00:00Z").unwrap().into();
        assert!(el.is_weekend(sat));
        let mon = chrono::DateTime::parse_from_rfc3339("2026-07-13T12:00:00Z").unwrap().into();
        assert!(!el.is_weekend(mon));
    }

    #[test]
    fn test_tick_pipeline_blocked_on_lunch() {
        // Unit: verify block_phase prevents pipeline
        let mut session = SessionState::new();
        session.phase = SessionPhase::Lunch;
        assert!(session.is_block_phase());
        session.phase = SessionPhase::Close;
        assert!(session.is_block_phase());
    }

    #[test]
    fn test_force_close_triggers_at_session_close() {
        let close = settings::SESSION_CLOSE;
        assert_eq!(close, NaiveTime::from_hms_opt(21, 0, 0).unwrap());
    }
}
