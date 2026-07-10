// L3 - AI Global Event Loop
// Live Dry-Run dengan optimal strategy:
// Session filter (hour 5,12,19 UTC), HV>0.5, contrarian, SL=$3 TP=$4.50

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
const TRADE_HOURS: &[u32] = &[5, 12, 19]; // UTC
const HV_THRESHOLD: f64 = 0.5;
const STOP_LOSS: f64 = 3.0;
const TAKE_PROFIT: f64 = 4.50;

pub struct EventLoop {
    symbol: String,
    state: StateManager,
    anti_paralysis: AntiParalysis,
    tick_counter: u64,
    data_feed: DataFeed,
    mt5: Option<Mt5Api>,
    /// Rolling price history for HV Z-Score computation
    prices: Vec<f64>,
    /// Current position state
    in_position: bool,
    position_type: String,
    entry_price: f64,
    sl_price: f64,
    tp_price: f64,
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
            prices: Vec::with_capacity(500),
            in_position: false,
            position_type: String::new(),
            entry_price: 0.0,
            sl_price: 0.0,
            tp_price: 0.0,
        }
    }

    pub fn data_feed(&self) -> &DataFeed {
        &self.data_feed
    }
    pub fn state(&self) -> &StateManager {
        &self.state
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

        if is_m1 {
            self.fetch_live_data().await;
        }

        if l0_infra::is_jam_hantu_now(&now) {
            log::warn!("Jam Hantu triggered");
            self.state.set_state(TradingState::ForceClose);
            l0_infra::force_close_all().await;
            return;
        }

        if !is_m1 && !is_m15 {
            return;
        }

        // Check position SL/TP every M1
        if self.in_position && is_m1 {
            if let Some(tick) = self.data_feed.last_tick() {
                self.check_position(tick.bid).await;
            }
        }

        // L3: Crisis adaptation
        if self.anti_paralysis.is_crisis() {
            self.state.set_state(TradingState::CrisisAdaptation);
        }

        // L2: News check
        let has_news = news_aggregator::has_red_folder_soon()
            .await
            .unwrap_or(false);
        if has_news {
            self.state.set_state(TradingState::NewsAnomalyMode);
            log::info!("News anomaly detected — no trades during news");
        } else if is_m15 {
            self.state.set_state(TradingState::NormalRegulerMode);
            self.run_strategy(now).await;
        }
    }

    /// Fetch live tick + DOM
    async fn fetch_live_data(&mut self) {
        if let Some(ref mt5) = self.mt5 {
            match mt5.get_tick().await {
                Ok(tick) => {
                    let price_tick = crate::core::l0_infra::PriceTick::from_tick_data(
                        &self.symbol,
                        tick.bid,
                        tick.ask,
                    );
                    self.data_feed.update_tick(price_tick);
                    self.prices.push(tick.bid);
                    if self.prices.len() > 500 {
                        self.prices.remove(0);
                    }

                    match mt5.get_dom_raw().await {
                        Ok(levels) => self.data_feed.update_dom(&levels),
                        Err(e) => log::debug!("DOM poll failed: {}", e),
                    }
                }
                Err(e) => log::warn!("Tick poll failed: {}", e),
            }
        }
    }

    /// Compute HV Z-Score from last 20 M15 periods
    fn compute_hv_zscore(&self) -> f64 {
        let n = self.prices.len();
        if n < 21 {
            return 0.0;
        }
        let window = &self.prices[n - 21..];
        let returns: Vec<f64> = window
            .windows(2)
            .map(|w| (w[1] - w[0]) / w[0].abs().max(0.001))
            .collect();
        if returns.is_empty() {
            return 0.0;
        }
        let mean = returns.iter().sum::<f64>() / returns.len() as f64;
        let var = returns.iter().map(|r| (r - mean).powi(2)).sum::<f64>() / returns.len() as f64;
        let std = var.sqrt();
        if std < 1e-10 {
            return 0.0;
        }
        *returns.last().unwrap_or(&0.0) / std
    }

    /// Main strategy logic — session filter + contrarian reversal
    async fn run_strategy(&mut self, now: chrono::DateTime<chrono::Utc>) {
        let hour = now.format("%H").to_string().parse::<u32>().unwrap_or(99);
        if !TRADE_HOURS.contains(&hour) {
            log::debug!("Outside trade hours ({})", hour);
            return;
        }

        if self.in_position {
            log::info!("Already in position, skipping new signal");
            return;
        }

        let hv = self.compute_hv_zscore();
        if hv <= HV_THRESHOLD {
            log::debug!("HV Z-Score {:.2} <= {:.1}, no trade", hv, HV_THRESHOLD);
            return;
        }

        let price = self.data_feed.last_price();
        if price <= 0.0 {
            return;
        }

        if self.prices.len() < 2 {
            return;
        }
        let prev_price = self.prices[self.prices.len() - 2];
        let trending_up = price > prev_price;
        let direction = if trending_up { "SELL" } else { "BUY" };

        let (sl, tp) = if direction == "BUY" {
            (price - STOP_LOSS, price + TAKE_PROFIT)
        } else {
            (price + STOP_LOSS, price - TAKE_PROFIT)
        };

        self.in_position = true;
        self.position_type = direction.to_string();
        self.entry_price = price;
        self.sl_price = sl;
        self.tp_price = tp;

        let msg = format!(
            "[AI SEITH] LIVE SIGNAL: {} XAUUSD\nEntry: {:.3}\nSL: {:.3} | TP: {:.3}\nHV: {:.2} | Hour: {} UTC",
            direction, price, sl, tp, hv, hour
        );
        log::info!("{}", msg);

        // Telegram notification
        let _ = shared::external::telegram_bridge::send_message(&msg).await;
    }

    /// Check SL/TP hit every M1 tick
    async fn check_position(&mut self, current_price: f64) {
        let hit_sl = if self.position_type == "BUY" {
            current_price <= self.sl_price
        } else {
            current_price >= self.sl_price
        };
        let hit_tp = if self.position_type == "BUY" {
            current_price >= self.tp_price
        } else {
            current_price <= self.tp_price
        };

        if hit_sl || hit_tp {
            let result = if hit_tp { "WIN" } else { "LOSS" };
            let pips = if self.position_type == "BUY" {
                current_price - self.entry_price
            } else {
                self.entry_price - current_price
            };

            let msg = format!(
                "[AI SEITH] CLOSED: {} {}\nEntry: {:.3} Exit: {:.3}\nP&L: ${:.2}\nResult: {}",
                self.position_type, result, self.entry_price, current_price, pips, result
            );
            log::info!("{}", msg);

            let _ = shared::external::telegram_bridge::send_message(&msg).await;

            self.in_position = false;
            self.position_type.clear();
        }
    }

    pub async fn run(&mut self) {
        log::info!(
            "EventLoop started for {} (interval={}s) — LIVE DRY-RUN MODE",
            self.symbol,
            TICK_INTERVAL_SECS
        );
        log::info!(
            "Trade hours: {:?} UTC | HV>{:.1} | SL=${} TP=${}",
            TRADE_HOURS,
            HV_THRESHOLD,
            STOP_LOSS,
            TAKE_PROFIT
        );

        let api = Mt5Api::new(&self.symbol);
        match api.connect().await {
            Ok(()) => {
                log::info!("MT5 connected successfully");
                self.mt5 = Some(api);
            }
            Err(e) => {
                log::error!("MT5 connection failed: {}", e);
            }
        }

        // Telegram startup message
        let _ = shared::external::telegram_bridge::send_message(
            &format!(
                "[AI SEITH] DRY-RUN STARTED\nInstrument: {}\nTrade hours: {:?} UTC\nSL=${} TP=${}\nStatus: Monitoring...",
                self.symbol, TRADE_HOURS, STOP_LOSS, TAKE_PROFIT
            )
        ).await;

        let mut ticker = interval(Duration::from_secs(TICK_INTERVAL_SECS));
        loop {
            ticker.tick().await;
            self.tick().await;
        }
    }
}
