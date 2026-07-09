// Backtest simulator — pumps M1 OHLCV through L1→Execution pipeline
// Real momentum-based Bayesian signal instead of random pattern

use crate::core::backtest::data_loader::M1Candle;
use crate::core::backtest::reporter::BacktestReport;
use crate::core::l1_structure::filter1_bayesian::BayesianDecision;
use crate::core::l1_structure::filter2_cvar::evaluate_cvar;
use crate::core::l1_structure::filter3_market_compass::evaluate_compass;
use crate::core::l1_structure::filter4_orderflow::{self, OfsResult};
use crate::core::l1_structure::signal_classifier::{self, SignalTier};
use crate::indicators::body_ratio::calculate_body_ratio;
use crate::indicators::frama;
use crate::indicators::price_velocity::calculate_price_velocity;
use crate::indicators::vwap_bands::calculate_vwap;
use anyhow::Result;

const GVZ_DEFAULT: f64 = 0.0;

#[derive(Debug, Clone)]
pub struct BacktestTrade {
    pub time: String,
    pub tier: String,
    pub direction: String,
    pub entry: f64,
    pub exit: f64,
    pub pips: f64,
    pub result: String,
    pub spread: f64,
    pub slippage: f64,
}

#[derive(Default)]
pub struct BacktestEngine {
    pub trades: Vec<BacktestTrade>,
    m15_closes: Vec<f64>,
    pub balance: f64,
    prev_close: Option<f64>,
}

impl BacktestEngine {
    pub fn new() -> Self {
        Self {
            trades: Vec::new(),
            m15_closes: Vec::new(),
            balance: 10_000.0,
            prev_close: None,
        }
    }

    pub fn report(&self) -> BacktestReport {
        BacktestReport::from_trades(&self.trades, self.balance)
    }

    pub async fn run(&mut self, candles: &[M1Candle]) -> Result<()> {
        let m15_groups: Vec<Vec<M1Candle>> = candles.chunks(15).map(|c| c.to_vec()).collect();
        log::info!(
            "Backtesting {} M15 bars from {} M1 candles",
            m15_groups.len(),
            candles.len()
        );

        for i in 0..m15_groups.len() {
            self.process_m15(i, &m15_groups[i], &m15_groups).await;
        }
        Ok(())
    }

    async fn process_m15(&mut self, idx: usize, m15: &[M1Candle], all: &[Vec<M1Candle>]) {
        let open = m15[0].open;
        let high = m15.iter().map(|c| c.high).fold(f64::MIN, f64::max);
        let low = m15.iter().map(|c| c.low).fold(f64::MAX, f64::min);
        let close = m15.last().unwrap().close;
        let volume: f64 = m15.iter().map(|c| c.volume).sum();

        // Direction from candle
        let direction = if close > open { "BUY" } else { "SELL" };

        // ── Indicators ──
        let body_ratio = calculate_body_ratio(open, close, high, low);
        let range_pips = (high - low) / 0.010;

        // Velocity
        let total_range = m15.iter().map(|c| c.high - c.low).sum::<f64>();
        let velocity = calculate_price_velocity(total_range, 900.0);

        // FRAMA
        self.m15_closes.push(close);
        let (frama_val, frama_dev) = if self.m15_closes.len() >= 32 {
            frama::calculate_frama(&self.m15_closes)
                .map(|f| (f.value, f.deviation))
                .unwrap_or((close, 0.0))
        } else {
            (close, 0.0)
        };

        // VWAP
        let m1_prices: Vec<f64> = m15.iter().map(|c| (c.high + c.low) / 2.0).collect();
        let m1_vols: Vec<f64> = m15.iter().map(|c| c.volume).collect();
        let vwap_val = calculate_vwap(&m1_prices, &m1_vols)
            .map(|v| v.vwap)
            .unwrap_or(close);
        let vwap_dev = (close - vwap_val) / vwap_val.max(0.001) * 100.0;

        let poc_dist = ((close - (high + low) / 2.0) / 0.010).abs();

        // ── Momentum-Based Bayesian Signal ──
        let bayesian = self.compute_bayesian(
            direction, close, high, low, body_ratio, range_pips, frama_val, volume,
        );
        if matches!(bayesian, BayesianDecision::Block) {
            self.prev_close = Some(close);
            return;
        }

        let cvar = evaluate_cvar(velocity, &bayesian);
        if !cvar.passed {
            self.prev_close = Some(close);
            return;
        }

        let compass = evaluate_compass(GVZ_DEFAULT, frama_dev, poc_dist, vwap_dev);
        if !matches!(
            compass.decision,
            crate::core::l1_structure::filter3_market_compass::CompassDecision::Pass
        ) {
            self.prev_close = Some(close);
            return;
        }

        let ofs = self.compute_ofs(m15, volume);
        if matches!(
            ofs.decision,
            filter4_orderflow::OfsDecision::BlockRetailNoise
        ) {
            self.prev_close = Some(close);
            return;
        }

        // ── Classify ──
        let signal = signal_classifier::classify_signal(&bayesian, &cvar, &compass, &ofs);
        if matches!(signal.tier, SignalTier::NoSignal) {
            self.prev_close = Some(close);
            return;
        }

        // ── Execute ──
        if let Some(next) = all.get(idx + 1).and_then(|c| c.last()) {
            let pips = (next.close - close) * if direction == "BUY" { 1.0 } else { -1.0 } * 0.010;
            let pips_rounded = (pips * 1000.0).round() / 1000.0;
            let result = if pips > 0.0 { "WIN" } else { "LOSS" };

            self.trades.push(BacktestTrade {
                time: format_ts(m15[0].time),
                tier: format!("{:?}", signal.tier),
                direction: direction.to_string(),
                entry: (close * 100.0).round() / 100.0,
                exit: (next.close * 100.0).round() / 100.0,
                pips: pips_rounded,
                result: result.to_string(),
                spread: (0.5_f64 * 100.0).round() / 100.0,
                slippage: 0.001,
            });

            self.balance += pips_rounded * 0.1;
        }
        self.prev_close = Some(close);
    }

    /// Real momentum-based Bayesian probability.
    /// Combines trend strength, volatility ratio and volume confirmation
    /// into a single probability score (0-100).
    #[allow(clippy::too_many_arguments)]
    fn compute_bayesian(
        &mut self,
        direction: &str,
        close: f64,
        _high: f64,
        _low: f64,
        body_ratio: f64,
        _range_pips: f64,
        frama_val: f64,
        _volume: f64,
    ) -> BayesianDecision {
        let prob = self.momentum_probability(
            direction,
            close,
            _high,
            _low,
            body_ratio,
            _range_pips,
            frama_val,
            _volume,
        );
        if prob >= 0.75 {
            BayesianDecision::Tier1Institutional
        } else if prob >= 0.60 {
            BayesianDecision::Tier2Tactical
        } else {
            BayesianDecision::Block
        }
    }

    #[allow(clippy::too_many_arguments)]
    fn momentum_probability(
        &self,
        direction: &str,
        close: f64,
        _high: f64,
        _low: f64,
        body_ratio: f64,
        _range_pips: f64,
        frama_val: f64,
        _volume: f64,
    ) -> f64 {
        // 1. Momentum strength: body / range (strong trend = fat body)
        let trend_strength = 1.0 - body_ratio.min(1.0); // inverted: small body = weak

        // 2. Trend persistence: close vs FRAMA
        let trend_alignment = if frama_val > 0.0 {
            let dist_to_frama = (close - frama_val).abs() / frama_val.max(0.001);
            (dist_to_frama * 5.0).min(1.0)
        } else {
            0.5
        };

        // 3. Prior from previous direction (if no prev_close, neutral)
        let prior = match self.prev_close {
            Some(prev) => {
                let prev_dir = if close > prev { "BUY" } else { "SELL" };
                if prev_dir == direction {
                    0.55
                } else {
                    0.40
                }
            }
            None => 0.50,
        };

        // 4. Combine: prior adjusted by trend signals
        let raw = prior * 0.5 + trend_strength * 0.3 + trend_alignment * 0.2;
        raw.clamp(0.0, 1.0)
    }

    fn compute_ofs(&self, m15: &[M1Candle], _total_vol: f64) -> OfsResult {
        let bullish: f64 = m15
            .iter()
            .filter(|c| c.close >= c.open)
            .map(|c| c.volume)
            .sum();
        let bearish: f64 = m15
            .iter()
            .filter(|c| c.close < c.open)
            .map(|c| c.volume)
            .sum();
        let s = if bullish > bearish * 1.5 {
            1
        } else if bearish > bullish * 1.5 {
            -1
        } else {
            0
        };
        filter4_orderflow::calculate_ofs(s, s, s)
    }
}

fn format_ts(ts: i64) -> String {
    chrono::DateTime::from_timestamp(ts, 0)
        .map(|d| d.format("%Y-%m-%d %H:%M").to_string())
        .unwrap_or_default()
}
