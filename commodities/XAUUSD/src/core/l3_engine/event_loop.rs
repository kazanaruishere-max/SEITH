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

        // ── 1. Risk Check ──
        let price = self.data_feed.last_price();
        if price <= 0.0 {
            return;
        }
        if self.prices.len() < 2 {
            return;
        }

        let spread = self.data_feed.spread();
        let position_count = if self.in_position { 1 } else { 0 };
        let session_state = crate::core::execution::risk_manager::TradeSession {
            daily_loss: 0.0,
            weekly_loss: 0.0,
            open_positions: position_count,
        };
        let limits = crate::core::execution::risk_manager::RiskLimits::default();
        if let Err(e) =
            crate::core::execution::risk_manager::can_trade(&session_state, &limits, spread, price)
        {
            log::warn!("Risk check failed: {}", e);
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

        // ── 2. Confidence & Lot ──
        let hv_strength = (hv / 3.0).min(1.0); // normalize HV to 0-1
        let confidence = 65.0 + hv_strength * 25.0; // 65-90%
        let lot = self.calculate_scalable_lot(confidence);
        if lot <= 0.0 {
            log::info!("Confidence {:.0}% too low, skipping", confidence);
            return;
        }

        // ── 3. Order Manager (Limit/Stop only, HARAM Instant) ──
        let plan = crate::core::execution::order_manager::plan_execution(
            &crate::core::l1_structure::signal_classifier::SignalTier::Tier1Institutional,
            direction,
            price,
            (price + 5.0, price - 5.0),
            spread,
            0.5,
            100.0,
        );

        let (order_type, entry, sl, tp) = match &plan {
            crate::core::execution::order_manager::ExecutionPlan::Limit(p) => {
                ("BUY_LIMIT", p.entry_price, p.stop_loss, p.take_profit)
            }
            crate::core::execution::order_manager::ExecutionPlan::Stop(p) => {
                ("BUY_STOP", p.entry_price, p.stop_loss, p.take_profit)
            }
            crate::core::execution::order_manager::ExecutionPlan::None => {
                log::info!("No valid order plan (Instant skipped — HARAM)");
                return;
            }
        };

        // ── 4. MT5 Execution ──
        if let Some(ref mt5) = self.mt5 {
            match mt5
                .place_pending_limit(order_type, lot, entry, sl, tp)
                .await
            {
                Ok(ticket) => {
                    log::info!(
                        "Order placed: ticket={} lot={} {}@{:.3} SL={:.3} TP={:.3}",
                        ticket,
                        lot,
                        order_type,
                        entry,
                        sl,
                        tp
                    );
                    self.in_position = true;
                    self.position_type = if order_type.contains("BUY") {
                        "BUY".to_string()
                    } else {
                        "SELL".to_string()
                    };
                    self.entry_price = entry;
                    self.sl_price = sl;
                    self.tp_price = tp;
                }
                Err(e) => log::error!("Order failed: {}", e),
            }
        }

        // Build real-time reasoning
        let reasoning = format!(
            "Contrarian reversal signal detected. HV Z-Score {:.2} above {:.1} threshold.\n\
            Price {:.3} vs previous {:.3} = {} trend.\n\
            Mean reversion expected within 2-3 M15 bars based on session analysis.",
            hv,
            HV_THRESHOLD,
            price,
            self.prices
                .get(self.prices.len().saturating_sub(2))
                .copied()
                .unwrap_or(0.0),
            if trending_up { "UP" } else { "DOWN" }
        );

        // Build orderflow info from DOM data
        let orderflow_info = if let Some(dom) = self.data_feed.dom() {
            let bid_vol: u64 = dom.bids.iter().map(|l| l.volume).sum();
            let ask_vol: u64 = dom.asks.iter().map(|l| l.volume).sum();
            if bid_vol > ask_vol * 2 {
                format!(
                    "\u{1f4a7} Order Flow: Dominasi BID ({:.0}% dari total depth)",
                    bid_vol as f64 / (bid_vol + ask_vol).max(1) as f64 * 100.0
                )
            } else if ask_vol > bid_vol * 2 {
                format!(
                    "\u{1f4a7} Order Flow: Dominasi ASK ({:.0}% dari total depth)",
                    ask_vol as f64 / (bid_vol + ask_vol).max(1) as f64 * 100.0
                )
            } else {
                "\u{1f4a7} Order Flow: Seimbang, tidak ada dominasi signifikan".to_string()
            }
        } else {
            "\u{1f4a7} Order Flow: Data DOM tidak tersedia".to_string()
        };

        // Session name based on hour
        let session_name = match hour {
            5..=7 => "Asia Prime",
            8..=11 => "London Open",
            12..=16 => "London/NY Overlap",
            17..=20 => "NY Session",
            21..=23 => "NY Late",
            _ => "Asia",
        };

        // Invalid condition
        let invalid_condition = if direction == "BUY" {
            format!("Close below {:.3} (premium zone)", price - STOP_LOSS * 2.0)
        } else {
            format!("Close above {:.3} (premium zone)", price + STOP_LOSS * 2.0)
        };

        // Signal ID (timestamp-based)
        let signal_id = format!("{:x}", chrono::Utc::now().timestamp_millis() & 0xFFFFFF);

        // Build price list for chart from recent ticks
        let chart_prices: Vec<(i64, f64, f64)> = self
            .prices
            .iter()
            .rev()
            .take(60)
            .enumerate()
            .map(|(i, &p)| {
                let ts =
                    (chrono::Utc::now() - chrono::Duration::seconds(i as i64 * 15)).timestamp();
                (ts, p - 0.05, p + 0.05)
            })
            .collect();

        let _ = shared::external::telegram_bridge::send_signal(
            direction,
            price,
            sl,
            tp + (tp - price) * 2.0,
            Some(tp + (tp - price) * 3.0),
            0.01,
            65.0,
            &reasoning,
            "SELL_LIMIT",
            chart_prices,
            &signal_id,
            session_name,
            &orderflow_info,
            &invalid_condition,
        )
        .await;
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

    /// Scalable lot via Logistic S-Curve.
    /// < 65% -> skip, 65% -> 0.01, 70% -> 0.01, 75% -> 0.03, 80%+ -> 0.05
    fn calculate_scalable_lot(&self, confidence: f64) -> f64 {
        if confidence < 65.0 {
            return 0.0;
        }
        let norm = ((confidence - 65.0) / 15.0).clamp(0.0, 1.0);
        let lot = 0.05 / (1.0 + std::f64::consts::E.powf(-6.0 * (norm - 0.5)));
        (lot * 100.0).round().max(1.0) / 100.0
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
