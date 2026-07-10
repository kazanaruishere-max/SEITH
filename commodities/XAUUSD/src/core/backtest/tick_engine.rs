// Tick-level backtest engine
// Processes ticks sequentially, detects micro-structure patterns, executes trades

use crate::core::backtest::reporter::BacktestReport;
use crate::core::backtest::tick_data::{Tick, TickStream};

#[derive(Debug, Clone)]
pub struct TickTrade {
    pub time: String,
    pub tick_entry: usize,
    pub tick_exit: usize,
    pub direction: String,
    pub entry_price: f64,
    pub exit_price: f64,
    pub pips: f64,
    pub result: String,
}

#[derive(Debug, Default, Clone)]
pub struct TickFeatures {
    pub freq: f64,
    pub velocity: f64,
    pub volume_imbalance: f64,
    pub spread: f64,
    pub mid: f64,
    pub atr: f64,
    pub hv_zscore: f64,
}

#[derive(Default)]
pub struct TickEngine {
    pub trades: Vec<TickTrade>,
    features: TickFeatures,
    tick_idx: usize,
    prices: Vec<f64>,
    spreads: Vec<f64>,
    vol_imbalances: Vec<f64>,
    in_position: bool,
    direction: String,
    entry_price: f64,
    sl_price: f64,
    tp_price: f64,
    entry_tick: usize,
}

impl TickEngine {
    pub fn new() -> Self {
        Self::default()
    }

    /// Run full tick-level backtest
    pub fn run(&mut self, stream: &mut TickStream) -> BacktestReport {
        log::info!("Running tick-level backtest on {} ticks", stream.len());

        while let Some(tick) = stream.read() {
            self.process_tick(tick);
            self.tick_idx += 1;
        }

        log::info!("Tick backtest complete: {} trades", self.trades.len());
        self.report()
    }

    fn process_tick(&mut self, tick: &Tick) {
        let mid = tick.mid();
        self.prices.push(mid);
        self.spreads.push(tick.spread);
        self.vol_imbalances.push(self.compute_imbalance(tick));

        if self.prices.len() > 200 {
            self.prices.remove(0);
            self.spreads.remove(0);
            self.vol_imbalances.remove(0);
        }

        self.update_features(tick);
        self.compute_hv_reference();

        if self.in_position {
            self.check_exit(tick);
            return;
        }

        if self.features.hv_zscore > 0.5 {
            if let Some(signal) = self.detect_reversal() {
                self.enter_trade(tick, &signal);
            }
        }
    }

    fn update_features(&mut self, tick: &Tick) {
        let n = self.prices.len();
        if n < 10 {
            return;
        }
        let freq = if n >= 2 {
            (self.prices[n - 1] - self.prices[n.saturating_sub(5.min(n))]).abs()
        } else {
            0.0
        };
        let velocity = if n >= 10 {
            (self.prices[n - 1] - self.prices[n - 10]).abs() / 10.0
        } else {
            0.0
        };
        let start = n.saturating_sub(50);
        let vol_imb: f64 =
            self.vol_imbalances[start..].iter().sum::<f64>() / (n - start).max(1) as f64;
        let atr_start = n.saturating_sub(20);
        let atr: f64 = if n - atr_start >= 2 {
            self.prices[atr_start..]
                .windows(2)
                .map(|w| (w[1] - w[0]).abs())
                .sum::<f64>()
                / (n - atr_start).max(1) as f64
        } else {
            0.001
        };
        self.features = TickFeatures {
            freq: freq.max(0.0),
            velocity,
            volume_imbalance: vol_imb,
            spread: tick.spread,
            mid: tick.mid(),
            atr: atr.max(0.001),
            hv_zscore: self.features.hv_zscore,
        };
    }

    fn compute_imbalance(&self, tick: &Tick) -> f64 {
        let total = tick.bid_vol + tick.ask_vol;
        if total == 0 {
            0.0
        } else {
            (tick.bid_vol - tick.ask_vol) as f64 / total as f64
        }
    }

    fn compute_hv_reference(&mut self) {
        let n = self.prices.len();
        if n < 50 {
            return;
        }
        let window = &self.prices[n - 50..];
        let mean = window.iter().sum::<f64>() / window.len() as f64;
        let var = window.iter().map(|p| (p - mean).powi(2)).sum::<f64>() / window.len() as f64;
        let std = var.sqrt();
        if std > 0.0 {
            let last_return = (window[window.len() - 1] - window[0]) / window[0].abs().max(0.001);
            self.features.hv_zscore = last_return / (std / mean.abs().max(0.001));
        }
    }

    fn detect_reversal(&self) -> Option<(String, f64)> {
        let n = self.prices.len();
        if n < 20 {
            return None;
        }

        // Collect all pattern signals with confidence
        let mut signals: Vec<(String, f64)> = Vec::new();

        if let Some((p, c)) = self.detect_absorption() {
            signals.push((p, c));
        }
        if let Some((p, c)) = self.detect_spread_exhaustion() {
            signals.push((p, c));
        }
        if let Some((p, c)) = self.detect_cvd_divergence() {
            signals.push((p, c));
        }
        if let Some((p, c)) = self.detect_trend_reversal() {
            signals.push((p, c));
        }

        // Sort by confidence descending
        signals.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

        // Take highest confidence signal
        signals.into_iter().next()
    }

    /// Phase 2: Simple reversal heuristic (works on synthetic data)
    fn detect_trend_reversal(&self) -> Option<(String, f64)> {
        let n = self.prices.len();
        if n < 20 {
            return None;
        }
        let last_5 = &self.prices[n.saturating_sub(5)..];
        let price_change =
            last_5.last().copied().unwrap_or(0.0) - last_5.first().copied().unwrap_or(0.0);
        if price_change.abs() < self.features.atr * 0.3 {
            return None;
        }
        let trend = if price_change > 0.0 { "UP" } else { "DOWN" };

        if self.features.hv_zscore > 1.5 {
            let dir = if trend == "UP" { "SHORT" } else { "LONG" };
            let strength = (self.features.hv_zscore / 3.0).min(1.0);
            let confidence = 50.0 + strength * 30.0; // 50-80%
            return Some((format!("HV_REV_{}", dir), confidence));
        }
        None
    }

    /// Phase 3: Bid absorption with confidence scoring
    fn detect_absorption(&self) -> Option<(String, f64)> {
        let n = self.prices.len();
        if n < 20 {
            return None;
        }
        let recent = &self.prices[n.saturating_sub(10)..];
        if recent.len() < 6 {
            return None;
        }
        let mid = recent.len() / 2;
        let dropped = recent[0] > recent[mid.saturating_sub(1)].max(recent[mid]);
        let flattened =
            (recent.last().copied().unwrap_or(0.0) - recent[mid]).abs() < self.features.atr * 0.3;
        if dropped && flattened {
            // Confidence based on volume imbalance strength
            let confidence = 60.0 + (self.features.volume_imbalance * 40.0).min(35.0);
            return Some(("ABSORPTION_LONG".to_string(), confidence.min(95.0)));
        }
        None
    }

    /// Phase 3: Spread exhaustion with confidence
    fn detect_spread_exhaustion(&self) -> Option<(String, f64)> {
        if self.spreads.len() < 15 {
            return None;
        }
        let s = &self.spreads[self.spreads.len().saturating_sub(10)..];
        if s.len() < 8 {
            return None;
        }
        let avg = s.iter().sum::<f64>() / s.len() as f64;
        let max = s.iter().copied().fold(0.0f64, f64::max);
        let curr = s.last().copied().unwrap_or(0.0);
        if max > avg * 1.5 && curr < avg * 1.1 {
            let spread_ratio = max / avg.max(0.001);
            let confidence = 55.0 + ((spread_ratio - 1.5) * 20.0).min(35.0);
            let trend = self.prices.last().copied().unwrap_or(0.0)
                - self
                    .prices
                    .get(self.prices.len().saturating_sub(5))
                    .copied()
                    .unwrap_or(0.0);
            let dir = if trend < 0.0 { "LONG" } else { "SHORT" };
            return Some((format!("SPREAD_EXHAUST_{}", dir), confidence.min(90.0)));
        }
        None
    }

    /// Phase 3: CVD divergence with confidence
    fn detect_cvd_divergence(&self) -> Option<(String, f64)> {
        if self.vol_imbalances.len() < 30 {
            return None;
        }
        let v = &self.vol_imbalances[self.vol_imbalances.len().saturating_sub(20)..];
        let avg_imb = v.iter().sum::<f64>() / v.len().max(1) as f64;
        let dropping = self.prices.last().copied().unwrap_or(0.0)
            < self
                .prices
                .get(self.prices.len().saturating_sub(10))
                .copied()
                .unwrap_or(0.0);
        if dropping && avg_imb > 0.2 {
            let confidence = 60.0 + (avg_imb * 50.0).min(30.0);
            return Some(("CVD_DIVERGENCE_LONG".to_string(), confidence.min(90.0)));
        }
        None
    }

    fn enter_trade(&mut self, tick: &Tick, signal: &(String, f64)) {
        let (pattern, confidence) = signal;
        // Only trade if confidence >= threshold (env-overridable)
        let min_conf: f64 = std::env::var("BT_MIN_CONF")
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or(60.0);
        if *confidence < min_conf {
            return;
        }

        let direction = if pattern.contains("LONG") || pattern.contains("ABSORPTION") {
            "BUY"
        } else {
            "SELL"
        };
        let atr = self.features.atr.max(0.05);

        // Adaptive SL/TP: higher confidence → tighter SL, wider RR
        let (sl_mult, rr) = if *confidence > 80.0 {
            (1.5, 2.5) // High confidence: aggressive
        } else if *confidence > 65.0 {
            (2.0, 2.0) // Medium confidence
        } else {
            (2.5, 1.5) // Low confidence: conservative
        };

        let entry = tick.mid();
        let sl_dist = atr * sl_mult;
        let tp_dist = sl_dist * rr;
        let (sl, tp) = if direction == "BUY" {
            (entry - sl_dist, entry + tp_dist)
        } else {
            (entry + sl_dist, entry - tp_dist)
        };

        self.in_position = true;
        self.direction = direction.to_string();
        self.entry_price = entry;
        self.sl_price = sl;
        self.tp_price = tp;
        self.entry_tick = self.tick_idx;
    }

    fn check_exit(&mut self, tick: &Tick) {
        let mid = tick.mid();
        let hit_sl = if self.direction == "BUY" {
            mid <= self.sl_price
        } else {
            mid >= self.sl_price
        };
        let hit_tp = if self.direction == "BUY" {
            mid >= self.tp_price
        } else {
            mid <= self.tp_price
        };

        if hit_sl {
            let pips = if self.direction == "BUY" {
                self.sl_price - self.entry_price
            } else {
                self.entry_price - self.sl_price
            };
            self.trades.push(TickTrade {
                time: format_ts(tick.time_ms / 1000),
                tick_entry: self.entry_tick,
                tick_exit: self.tick_idx,
                direction: self.direction.clone(),
                entry_price: (self.entry_price * 1000.0).round() / 1000.0,
                exit_price: (self.sl_price * 1000.0).round() / 1000.0,
                pips: (pips * 1000.0).round() / 1000.0,
                result: "LOSS".to_string(),
            });
            self.in_position = false;
        } else if hit_tp {
            let pips = if self.direction == "BUY" {
                self.tp_price - self.entry_price
            } else {
                self.entry_price - self.tp_price
            };
            self.trades.push(TickTrade {
                time: format_ts(tick.time_ms / 1000),
                tick_entry: self.entry_tick,
                tick_exit: self.tick_idx,
                direction: self.direction.clone(),
                entry_price: (self.entry_price * 1000.0).round() / 1000.0,
                exit_price: (self.tp_price * 1000.0).round() / 1000.0,
                pips: (pips * 1000.0).round() / 1000.0,
                result: "WIN".to_string(),
            });
            self.in_position = false;
        }
    }

    fn report(&self) -> BacktestReport {
        let total = self.trades.len() as u32;
        let wins = self.trades.iter().filter(|t| t.result == "WIN").count() as u32;
        let losses = self.trades.iter().filter(|t| t.result == "LOSS").count() as u32;
        let win_rate = if total > 0 {
            wins as f64 / total as f64 * 100.0
        } else {
            0.0
        };
        let net_pips: f64 = self.trades.iter().map(|t| t.pips).sum();
        let gross_profit: f64 = self
            .trades
            .iter()
            .filter(|t| t.pips > 0.0)
            .map(|t| t.pips)
            .sum();
        let gross_loss: f64 = self
            .trades
            .iter()
            .filter(|t| t.pips < 0.0)
            .map(|t| t.pips.abs())
            .sum();
        let pf = if gross_loss > 0.0 {
            gross_profit / gross_loss
        } else {
            gross_profit.max(1.0)
        };

        let r = self
            .trades
            .iter()
            .fold((0u32, 0u32, 0u32, 0u32), |(mw, ml, cw, cl), t| {
                match t.result.as_str() {
                    "WIN" => (mw.max(cw + 1), ml, cw + 1, 0),
                    "LOSS" => (mw, ml.max(cl + 1), 0, cl + 1),
                    _ => (mw, ml, cw, cl),
                }
            });

        BacktestReport {
            total_trades: total,
            wins,
            losses,
            win_rate: (win_rate * 10.0).round() / 10.0,
            net_pips: (net_pips * 1000.0).round() / 1000.0,
            profit_factor: (pf * 100.0).round() / 100.0,
            max_consecutive_wins: r.0,
            max_consecutive_losses: r.1,
            max_drawdown_pct: 0.0,
            avg_spread: 0.0,
            final_balance: 10000.0 + net_pips,
            pnl: net_pips,
            sortino_ratio: 0.0,
            recovery_factor: 0.0,
            is_trades: 0,
            is_wr: 0.0,
            is_pf: 0.0,
            is_dd: 0.0,
            is_sortino: 0.0,
            oos_wr: 0.0,
            oos_pf: 0.0,
            oos_dd: 0.0,
            oos_sortino: 0.0,
            oos_trades: 0,
            oos_net_pips: 0.0,
        }
    }
}

fn format_ts(ts: i64) -> String {
    chrono::DateTime::from_timestamp(ts, 0)
        .map(|d| d.format("%Y-%m-%d %H:%M:%S").to_string())
        .unwrap_or_default()
}
