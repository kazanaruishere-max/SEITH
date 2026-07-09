// Backtest report — performance summary

use crate::core::backtest::simulator::BacktestTrade;
use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
pub struct BacktestReport {
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
    pub avg_slippage: f64,
    pub final_balance: f64,
    pub pnl: f64,
}

impl BacktestReport {
    pub fn from_trades(trades: &[BacktestTrade], final_balance: f64) -> Self {
        let total = trades.len() as u32;
        if total == 0 {
            return Self {
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
                avg_slippage: 0.0,
                final_balance,
                pnl: 0.0,
            };
        }

        let wins = trades.iter().filter(|t| t.result == "WIN").count() as u32;
        let losses = trades.iter().filter(|t| t.result == "LOSS").count() as u32;
        let win_rate = wins as f64 / total as f64 * 100.0;
        let net_pips: f64 = trades.iter().map(|t| t.pips).sum();

        let gross_profit: f64 = trades.iter().filter(|t| t.pips > 0.0).map(|t| t.pips).sum();
        let gross_loss: f64 = trades
            .iter()
            .filter(|t| t.pips < 0.0)
            .map(|t| t.pips.abs())
            .sum();
        let profit_factor = if gross_loss > 0.0 {
            gross_profit / gross_loss
        } else {
            gross_profit.max(1.0)
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
        let (max_cw, max_cl) = (result.0, result.1);

        // Drawdown: track running equity from pips
        let mut peak = 0.0_f64;
        let mut running = 0.0_f64;
        let mut max_dd = 0.0_f64;
        for t in trades {
            running += t.pips;
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

        let avg_spread = trades.iter().map(|t| t.spread).sum::<f64>() / total as f64;
        let avg_slippage = trades.iter().map(|t| t.slippage).sum::<f64>() / total as f64;

        Self {
            total_trades: total,
            wins,
            losses,
            win_rate,
            net_pips: (net_pips * 1000.0).round() / 1000.0,
            profit_factor: (profit_factor * 100.0).round() / 100.0,
            max_consecutive_wins: max_cw,
            max_consecutive_losses: max_cl,
            max_drawdown_pct: (max_dd * 100.0 * 100.0).round() / 100.0,
            avg_spread: (avg_spread * 100.0).round() / 100.0,
            avg_slippage: (avg_slippage * 1000.0).round() / 1000.0,
            final_balance: (final_balance * 100.0).round() / 100.0,
            pnl: ((final_balance - 10_000.0) * 100.0).round() / 100.0,
        }
    }
}
