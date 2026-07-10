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
            if let Some(pattern) = self.detect_reversal() {
                self.enter_trade(tick, &pattern);
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

    fn detect_reversal(&self) -> Option<String> {
        let n = self.prices.len();
        if n < 20 {
            return None;
        }

        // Phase 3: Micro-structure patterns (need real tick data with bid/ask volume)
        if let Some(p) = self.detect_absorption() {
            return Some(p);
        }
        if let Some(p) = self.detect_spread_exhaustion() {
            return Some(p);
        }
        if let Some(p) = self.detect_cvd_divergence() {
            return Some(p);
        }

        // Phase 2: Simple reversal heuristic (works on synthetic data)
        let last_5 = &self.prices[n.saturating_sub(5)..];
        let price_change =
            last_5.last().copied().unwrap_or(0.0) - last_5.first().copied().unwrap_or(0.0);
        if price_change.abs() < self.features.atr * 0.3 {
            return None;
        }
        let trend = if price_change > 0.0 { "UP" } else { "DOWN" };
        if self.features.hv_zscore > 1.0 {
            let dir = if trend == "UP" { "SHORT" } else { "LONG" };
            return Some(format!("REVERSAL_{}", dir));
        }
        None
    }

    /// Phase 3: Bid absorption — dropping price stalls with buying volume.
    fn detect_absorption(&self) -> Option<String> {
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
        if dropped && flattened && self.features.volume_imbalance > 0.2 {
            return Some("ABSORPTION_LONG".to_string());
        }
        None
    }

    /// Phase 3: Spread exhaustion — spread widens then narrows with reversal.
    fn detect_spread_exhaustion(&self) -> Option<String> {
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
            let trend = self.prices.last().copied().unwrap_or(0.0)
                - self
                    .prices
                    .get(self.prices.len().saturating_sub(5))
                    .copied()
                    .unwrap_or(0.0);
            if trend < 0.0 {
                Some("SPREAD_EXHAUST_LONG".to_string())
            } else {
                Some("SPREAD_EXHAUST_SHORT".to_string())
            }
        } else {
            None
        }
    }

    /// Phase 3: CVD divergence — price drops but CVD rises (accumulation).
    /// Requires Dukascopy tick-level bid/ask volume data.
    fn detect_cvd_divergence(&self) -> Option<String> {
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
            return Some("CVD_DIVERGENCE_LONG".to_string());
        }
        None
    }

    fn enter_trade(&mut self, tick: &Tick, pattern: &str) {
        let direction = if pattern.contains("LONG") || pattern == "BID_ABSORPTION" {
            "BUY"
        } else {
            "SELL"
        };
        let atr = self.features.atr.max(0.05);
        let entry = tick.mid();
        let (sl, tp) = if direction == "BUY" {
            (entry - atr * 2.0, entry + atr * 3.0)
        } else {
            (entry + atr * 2.0, entry - atr * 3.0)
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
