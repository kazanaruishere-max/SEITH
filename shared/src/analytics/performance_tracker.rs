// Performance tracking — hitung DD, WR, PF, RF, CW, CL

use crate::analytics::TradeRecord;

#[derive(Debug, Clone, Default)]
pub struct PerformanceMetrics {
    pub max_drawdown: f64,
    pub win_rate: f64,
    pub consecutive_wins: u32,
    pub consecutive_losses: u32,
    pub recovery_factor: f64,
    pub profit_factor: f64,
    pub total_trades: u32,
    pub net_profit: f64,
}

impl PerformanceMetrics {
    pub fn calculate(trades: &[TradeRecord]) -> Self {
        let total = trades.len() as u32;
        if total == 0 {
            return Self::default();
        }

        let (_wins, _losses, profit, loss_amt) =
            trades
                .iter()
                .fold((0u32, 0u32, 0.0f64, 0.0f64), |(w, l, p, la), t| {
                    match t.result.as_deref() {
                        Some("WIN") => (w + 1, l, p + t.profit_cent.unwrap_or(0.0), la),
                        Some("LOSS") => (w, l + 1, p, la + t.profit_cent.unwrap_or(0.0).abs()),
                        _ => (w, l, p, la),
                    }
                });
        let net = profit - loss_amt;
        let win_rate = if total > 0 {
            _wins as f64 / total as f64
        } else {
            0.0
        };

        let (cw, cl, _, _) = trades.iter().fold(
            (0u32, 0u32, 0u32, 0u32),
            |(max_w, max_l, cur_w, cur_l), t| match t.result.as_deref() {
                Some("WIN") => (max_w.max(cur_w + 1), max_l, cur_w + 1, 0),
                Some("LOSS") => (max_w, max_l.max(cur_l + 1), 0, cur_l + 1),
                _ => (max_w, max_l, cur_w, cur_l),
            },
        );

        Self {
            total_trades: total,
            win_rate,
            consecutive_wins: cw,
            consecutive_losses: cl,
            net_profit: net,
            profit_factor: if loss_amt > 0.0 {
                profit / loss_amt
            } else {
                profit.max(1.0)
            },
            ..Default::default()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn trade(result: &str, profit: f64) -> TradeRecord {
        let mut t = TradeRecord::new("XAUUSD", "TIER_1", "BUY", 100.0, 99.0, 102.0, 0.5);
        t.result = Some(result.to_string());
        t.profit_cent = Some(profit);
        t
    }

    #[test]
    fn test_empty() {
        let m = PerformanceMetrics::calculate(&[]);
        assert_eq!(m.total_trades, 0);
    }

    #[test]
    fn test_all_wins() {
        let t = vec![trade("WIN", 10.0); 10];
        let m = PerformanceMetrics::calculate(&t);
        assert!((m.win_rate - 1.0).abs() < 0.01);
        assert_eq!(m.consecutive_wins, 10);
    }
}
