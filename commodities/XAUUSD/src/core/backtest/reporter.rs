// Backtest report — full calibration-grade metrics
// Sortino Ratio, separate IS/OOS, SL/TP tracking

use crate::core::backtest::simulator::BacktestTrade;
use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
pub struct BacktestReport {
    // Overall
    pub total_trades: u32,
    pub wins: u32,
    pub losses: u32,
    pub win_rate: f64,
    pub net_pips: f64,
    pub profit_factor: f64,
    pub max_consecutive_wins: u32,
    pub max_consecutive_losses: u32,
    pub max_drawdown_pct: f64,
    pub avg_spread: f64,
    pub final_balance: f64,
    pub pnl: f64,
    pub sortino_ratio: f64,
    pub recovery_factor: f64,
    // In-Sample (training)
    pub is_trades: u32,
    pub is_wr: f64,
    pub is_pf: f64,
    pub is_dd: f64,
    pub is_sortino: f64,
    // Out-of-Sample (validation)
    pub oos_trades: u32,
    pub oos_wr: f64,
    pub oos_pf: f64,
    pub oos_dd: f64,
    pub oos_sortino: f64,
    pub oos_net_pips: f64,
}

impl BacktestReport {
    pub fn from_trades(trades: &[BacktestTrade], final_balance: f64, _split_time: i64) -> Self {
        let total = trades.len() as u32;

        if total == 0 {
            return Self::empty();
        }

        let wins = trades.iter().filter(|t| t.result == "WIN").count() as u32;
        let losses = trades.iter().filter(|t| t.result == "LOSS").count() as u32;
        let win_rate = if total > 0 {
            wins as f64 / total as f64 * 100.0
        } else {
            0.0
        };
        let net_pips: f64 = trades.iter().map(|t| t.pips).sum();

        let gross_profit: f64 = trades.iter().filter(|t| t.pips > 0.0).map(|t| t.pips).sum();
        let gross_loss: f64 = trades
            .iter()
            .filter(|t| t.pips < 0.0)
            .map(|t| t.pips.abs())
            .sum();
        let pf = if gross_loss > 0.0 {
            gross_profit / gross_loss
        } else {
            gross_profit.max(1.0)
        };

        let mut running = 0.0_f64;
        let mut peak = 0.0_f64;
        let mut max_dd = 0.0_f64;
        let mut returns: Vec<f64> = Vec::new();
        for t in trades {
            running += t.pips;
            returns.push(t.pips);
            if running > peak {
                peak = running;
            }
            let dd = if peak > 0.0 {
                (peak - running) / peak
            } else {
                0.0
            };
            if dd > max_dd {
                max_dd = dd;
            }
        }

        let max_dd_pct = (max_dd * 100.0 * 100.0).round() / 100.0;
        let rf = if max_dd > 0.0 {
            net_pips / (max_dd * 100.0).max(0.001)
        } else {
            0.0
        };

        // Sortino: (mean return) / (downside deviation)
        let mean_ret = if !returns.is_empty() {
            returns.iter().sum::<f64>() / returns.len() as f64
        } else {
            0.0
        };
        let downside_var: f64 = returns
            .iter()
            .filter(|r| **r < 0.0)
            .map(|r| (r - mean_ret).powi(2))
            .sum::<f64>()
            / returns.len().max(1) as f64;
        let downside_std = downside_var.sqrt();
        let sortino = if downside_std > 0.0 {
            mean_ret / downside_std
        } else {
            0.0
        };

        let avg_spread = if total > 0 {
            trades.iter().map(|t| t.spread).sum::<f64>() / total as f64
        } else {
            0.0
        };

        let result = trades
            .iter()
            .fold((0u32, 0u32, 0u32, 0u32), |(mw, ml, cw, cl), t| {
                match t.result.as_str() {
                    "WIN" => (mw.max(cw + 1), ml, cw + 1, 0),
                    "LOSS" => (mw, ml.max(cl + 1), 0, cl + 1),
                    _ => (mw, ml, cw, cl),
                }
            });
        let (cw, cl) = (result.0, result.1);

        // Split into IS/OOS
        let is_trades: Vec<&BacktestTrade> = trades.iter().filter(|t| t.in_sample).collect();
        let oos_trades: Vec<&BacktestTrade> = trades.iter().filter(|t| !t.in_sample).collect();

        let (is_wr, is_pf, is_dd, is_sort, oos_wr, oos_pf, oos_dd, oos_sort, oos_net) =
            Self::split_metrics(&is_trades, &oos_trades);

        Self {
            total_trades: total,
            wins,
            losses,
            win_rate: (win_rate * 10.0).round() / 10.0,
            net_pips: (net_pips * 1000.0).round() / 1000.0,
            profit_factor: (pf * 100.0).round() / 100.0,
            max_consecutive_wins: cw,
            max_consecutive_losses: cl,
            max_drawdown_pct: max_dd_pct,
            avg_spread: (avg_spread * 100.0).round() / 100.0,
            final_balance: (final_balance * 100.0).round() / 100.0,
            pnl: ((final_balance - 10_000.0) * 100.0).round() / 100.0,
            sortino_ratio: (sortino * 100.0).round() / 100.0,
            recovery_factor: (rf * 100.0).round() / 100.0,
            is_trades: is_trades.len() as u32,
            is_wr,
            is_pf,
            is_dd: (is_dd * 100.0).round() / 100.0,
            is_sortino: (is_sort * 100.0).round() / 100.0,
            oos_trades: oos_trades.len() as u32,
            oos_wr,
            oos_pf,
            oos_dd: (oos_dd * 100.0).round() / 100.0,
            oos_sortino: (oos_sort * 100.0).round() / 100.0,
            oos_net_pips: (oos_net * 1000.0).round() / 1000.0,
        }
    }

    fn split_metrics(
        is_trades: &[&BacktestTrade],
        oos_trades: &[&BacktestTrade],
    ) -> (f64, f64, f64, f64, f64, f64, f64, f64, f64) {
        fn calc(group: &[&BacktestTrade]) -> (f64, f64, f64, f64, f64) {
            let total = group.len() as f64;
            if total == 0.0 {
                return (0.0, 0.0, 0.0, 0.0, 0.0);
            }
            let wins = group.iter().filter(|t| t.result == "WIN").count() as f64;
            let wr = wins / total * 100.0;
            let gp: f64 = group.iter().filter(|t| t.pips > 0.0).map(|t| t.pips).sum();
            let gl: f64 = group
                .iter()
                .filter(|t| t.pips < 0.0)
                .map(|t| t.pips.abs())
                .sum();
            let pf = if gl > 0.0 { gp / gl } else { gp.max(1.0) };
            let mut run = 0.0_f64;
            let mut peak = 0.0_f64;
            let mut dd = 0.0_f64;
            let mut rets: Vec<f64> = Vec::new();
            for t in group.iter() {
                run += t.pips;
                rets.push(t.pips);
                if run > peak {
                    peak = run;
                }
                let d = if peak > 0.0 { (peak - run) / peak } else { 0.0 };
                if d > dd {
                    dd = d;
                }
            }
            let mean_r = if !rets.is_empty() {
                rets.iter().sum::<f64>() / rets.len() as f64
            } else {
                0.0
            };
            let dsv: f64 = rets
                .iter()
                .filter(|r| **r < 0.0)
                .map(|r| (r - mean_r).powi(2))
                .sum::<f64>()
                / rets.len().max(1) as f64;
            let sort = if dsv.sqrt() > 0.0 {
                mean_r / dsv.sqrt()
            } else {
                0.0
            };
            let net: f64 = group.iter().map(|t| t.pips).sum();
            (wr, pf, dd, sort, net)
        }
        let (iw, ip, id, is, _) = calc(is_trades);
        let (ow, op, od, os, on) = calc(oos_trades);
        (iw, ip, id, is, ow, op, od, os, on)
    }

    fn empty() -> Self {
        Self {
            total_trades: 0,
            wins: 0,
            losses: 0,
            win_rate: 0.0,
            net_pips: 0.0,
            profit_factor: 0.0,
            max_consecutive_wins: 0,
            max_consecutive_losses: 0,
            max_drawdown_pct: 0.0,
            avg_spread: 0.0,
            final_balance: 0.0,
            pnl: 0.0,
            sortino_ratio: 0.0,
            recovery_factor: 0.0,
            is_trades: 0,
            is_wr: 0.0,
            is_pf: 0.0,
            is_dd: 0.0,
            is_sortino: 0.0,
            oos_trades: 0,
            oos_wr: 0.0,
            oos_pf: 0.0,
            oos_dd: 0.0,
            oos_sortino: 0.0,
            oos_net_pips: 0.0,
        }
    }
}
