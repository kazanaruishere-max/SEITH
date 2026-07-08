// L3 - AI Global Event Loop
// Orchestrates L0 → L3 → L2 → L1 → Execution pipeline per M1/M15 tick

use std::time::Duration;
use tokio::time::interval;

use super::anti_paralysis::AntiParalysis;
use super::state_manager::{StateManager, TradingState};
use crate::core::l0_infra;

const TICK_INTERVAL_SECS: u64 = 15;
const TICK_M1: u64 = 60;
const TICK_M15: u64 = 900;

pub struct EventLoop {
    symbol: String,
    state: StateManager,
    anti_paralysis: AntiParalysis,
    tick_counter: u64,
}

impl EventLoop {
    pub fn new(symbol: &str) -> Self {
        Self {
            symbol: symbol.to_string(),
            state: StateManager::new(),
            anti_paralysis: AntiParalysis::new(),
            tick_counter: 0,
        }
    }

    pub fn state(&self) -> &StateManager {
        &self.state
    }

    pub fn anti_paralysis(&self) -> &AntiParalysis {
        &self.anti_paralysis
    }

    pub fn tick_count(&self) -> u64 {
        self.tick_counter
    }

    async fn tick(&mut self) {
        self.tick_counter += 1;
        let now = chrono::Utc::now();
        let interval_secs = TICK_INTERVAL_SECS;
        let is_m1 = self.tick_counter.is_multiple_of(TICK_M1 / interval_secs);
        let is_m15 = self.tick_counter.is_multiple_of(TICK_M15 / interval_secs);

        // L0: Jam Hantu check
        if l0_infra::is_jam_hantu_now(&now) {
            log::warn!("Jam Hantu triggered");
            self.state.set_state(TradingState::ForceClose);
            l0_infra::force_close_all().await;
            return;
        }

        // Skip processing on high-frequency ticks
        if !is_m1 && !is_m15 {
            return;
        }

        // L3: Read skip_strike_count (already tracked in memory)
        if self.anti_paralysis.is_crisis() {
            log::warn!(
                "Crisis adaptation active (skip_count={})",
                self.anti_paralysis.skip_count()
            );
            self.state.set_state(TradingState::CrisisAdaptation);
        }

        // L2: News check (stub - will be implemented in Phase 4)
        // TODO: call news_aggregator::has_red_folder_soon()
        let has_news = false;

        if has_news {
            self.state.set_state(TradingState::NewsAnomalyMode);
            // L2 News pipeline → will be implemented in Phase 4
        } else if is_m15 {
            self.state.set_state(TradingState::NormalRegulerMode);
            // L1 4 Filters → will be implemented in Phase 5
        }

        // Statistical Brain update (stub)
        if is_m1 {
            log::debug!(
                "EventLoop tick #{} state={}",
                self.tick_counter,
                self.state.get_state()
            );
        }
    }

    pub async fn run(&mut self) {
        log::info!(
            "EventLoop started for {} (interval={}s)",
            self.symbol,
            TICK_INTERVAL_SECS
        );
        let mut ticker = interval(Duration::from_secs(TICK_INTERVAL_SECS));

        loop {
            ticker.tick().await;
            self.tick().await;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_event_loop_initial_state() {
        let el = EventLoop::new("XAUUSD");
        assert_eq!(el.tick_count(), 0);
        assert!(el.state().is_normal());
    }

    #[tokio::test]
    async fn test_crisis_adaptation() {
        let mut ap = AntiParalysis::new();
        ap.increment();
        ap.increment();
        ap.increment();
        assert!(ap.is_crisis());
    }
}
