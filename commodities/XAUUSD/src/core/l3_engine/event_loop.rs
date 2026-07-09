// L3 - AI Global Event Loop
// Orchestrates L0 → L3 → L2 → L1 → Execution pipeline per M1/M15 tick
// Uses real DOM data for S_DOM and order_manager for execution

use std::time::Duration;
use tokio::time::interval;

use super::anti_paralysis::AntiParalysis;
use super::state_manager::{StateManager, TradingState};
use crate::core::l0_infra;
use crate::core::l0_infra::DataFeed;
use shared::external::mt5_bridge::Mt5Api;
use shared::external::news_aggregator;

const TICK_INTERVAL_SECS: u64 = 15;
const TICK_M1: u64 = 60;
const TICK_M15: u64 = 900;

pub struct EventLoop {
    symbol: String,
    state: StateManager,
    anti_paralysis: AntiParalysis,
    tick_counter: u64,
    data_feed: DataFeed,
    mt5: Option<Mt5Api>,
}

impl EventLoop {
    pub fn new(symbol: &str) -> Self {
        Self {
            symbol: symbol.to_string(),
            state: StateManager::new(),
            anti_paralysis: AntiParalysis::new(),
            tick_counter: 0,
            data_feed: DataFeed::new(symbol),
            mt5: None,
        }
    }

    pub fn data_feed(&self) -> &DataFeed {
        &self.data_feed
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

        // ── L0: Fetch live data on every M1 tick ──
        if is_m1 {
            self.fetch_live_data().await;
        }

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

        // L3: Crisis adaptation
        if self.anti_paralysis.is_crisis() {
            log::warn!(
                "Crisis adaptation active (skip_count={})",
                self.anti_paralysis.skip_count()
            );
            self.state.set_state(TradingState::CrisisAdaptation);
        }

        // L2: News check
        let has_news = news_aggregator::has_red_folder_soon()
            .await
            .unwrap_or(false);

        if has_news {
            self.state.set_state(TradingState::NewsAnomalyMode);
            log::info!("News anomaly detected, switching to NEWS mode");
        } else if is_m15 {
            // ── L1 Pipeline (M15 only) ──
            self.state.set_state(TradingState::NormalRegulerMode);
            self.run_l1_pipeline().await;
        }

        if is_m1 {
            log::debug!(
                "EventLoop tick #{} state={} spread={:.3}",
                self.tick_counter,
                self.state.get_state(),
                self.data_feed.spread(),
            );
        }
    }

    /// Fetch live tick + DOM from MT5 every M1 cycle
    async fn fetch_live_data(&mut self) {
        // Poll tick
        if let Some(ref mt5) = self.mt5 {
            match mt5.get_tick().await {
                Ok(tick) => {
                    let price_tick = crate::core::l0_infra::PriceTick::from_tick_data(
                        &self.symbol,
                        tick.bid,
                        tick.ask,
                    );
                    self.data_feed.update_tick(price_tick);

                    // Poll DOM
                    match mt5.get_dom_raw().await {
                        Ok(levels) => self.data_feed.update_dom(&levels),
                        Err(e) => log::debug!("DOM poll failed: {}", e),
                    }
                }
                Err(e) => log::warn!("Tick poll failed: {}", e),
            }
        }
    }

    /// Run L1 pipeline with current data_feed state
    async fn run_l1_pipeline(&mut self) {
        let spread = self.data_feed.spread();
        let price = self.data_feed.last_price();

        // S_DOM from real DOM data
        let s_dom_score = if let Some(dom) = self.data_feed.dom() {
            let bids: Vec<crate::core::l0_infra::DomLevel> = dom
                .bids
                .iter()
                .map(|l| crate::core::l0_infra::DomLevel {
                    price: l.price,
                    volume: l.volume,
                })
                .collect();
            let asks: Vec<crate::core::l0_infra::DomLevel> = dom
                .asks
                .iter()
                .map(|l| crate::core::l0_infra::DomLevel {
                    price: l.price,
                    volume: l.volume,
                })
                .collect();
            let snapshot = crate::core::l0_infra::DomSnapshot::new(&self.symbol, bids, asks, 3.5);
            let dom_result =
                crate::indicators::orderflow::s_dom::calculate_s_dom_from_snapshot(&snapshot);
            dom_result.heatmap_score
        } else {
            0
        };

        log::info!(
            "L1 pipeline: price={:.3} spread={:.3} dom={} s_dom={}",
            price,
            spread,
            self.data_feed
                .dom()
                .map(|d| d.asks.len() + d.bids.len())
                .unwrap_or(0),
            s_dom_score,
        );

        // TODO: Full L1 → Execution pipeline from live data
        // Will use: spread → risk_manager.can_trade()
        //          dom → s_dom → ofs → signal_classifier
        //          tick → bayesian (OANDA sentiment)
    }

    pub async fn run(&mut self) {
        log::info!(
            "EventLoop started for {} (interval={}s)",
            self.symbol,
            TICK_INTERVAL_SECS
        );

        // ── Connect to MT5 ──
        log::info!("Connecting to MT5...");
        let api = Mt5Api::new(&self.symbol);
        match api.connect().await {
            Ok(()) => {
                log::info!("MT5 connected successfully");
                self.mt5 = Some(api);
            }
            Err(e) => {
                log::error!("MT5 connection failed: {} — running in stub mode", e);
            }
        }

        let mut ticker = interval(Duration::from_secs(TICK_INTERVAL_SECS));
        loop {
            ticker.tick().await;
            self.tick().await;
        }
    }
}
