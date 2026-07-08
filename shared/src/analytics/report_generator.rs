// Report generator — format laporan ke Telegram / file

use crate::analytics::performance_tracker::PerformanceMetrics;
use crate::analytics::TradeRecord;

pub fn generate_summary(trades: &[TradeRecord]) -> String {
    let metrics = PerformanceMetrics::calculate(trades);
    format!(
        "📊 AI SEITH Performance Summary\n\
         ─────────────────────────\n\
         Trades:    {}\n\
         Win Rate:  {:.1}%\n\
         CW/CL:     {}/{}\n\
         Net P/L:   {:.2}\n\
         PF:        {:.2}\n\
         RF:        {:.2}",
        metrics.total_trades,
        metrics.win_rate * 100.0,
        metrics.consecutive_wins,
        metrics.consecutive_losses,
        metrics.net_profit,
        metrics.profit_factor,
        metrics.recovery_factor,
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty_summary() {
        let s = generate_summary(&[]);
        assert!(!s.is_empty());
    }

    #[test]
    fn test_summary_contains_metrics() {
        let t = vec![TradeRecord::new(
            "XAUUSD", "TIER_1", "BUY", 100.0, 99.0, 102.0, 0.5,
        )];
        let s = generate_summary(&t);
        assert!(s.contains("Win Rate"));
    }
}
