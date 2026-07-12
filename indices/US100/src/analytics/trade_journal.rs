use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TradeRecord {
    pub id: String,
    pub symbol: String,
    pub direction: String,
    pub entry_price: f64,
    pub exit_price: Option<f64>,
    pub sl: f64,
    pub tp: f64,
    pub slippage: Option<f64>,
    pub spread: Option<f64>,
    pub pl: Option<f64>,
    pub confidence: f64,
    pub gate_result: u32,
    pub mode: String,
    pub session_phase: String,
    pub entry_reason: Option<String>,
    pub exit_reason: Option<String>,
}

pub struct TradeJournal {
    pub trades: Vec<TradeRecord>,
}

impl TradeJournal {
    pub fn new() -> Self {
        Self { trades: Vec::with_capacity(100) }
    }

    pub fn record_trade(&mut self, record: TradeRecord) {
        log::info!("[Journal] Recording trade {} {} entry={}", record.id, record.direction, record.entry_price);
        self.trades.push(record);
    }

    pub fn extract_slippage(entry_price: f64, filled_price: f64) -> f64 {
        ((filled_price - entry_price) / entry_price).abs() * 100.0
    }

    pub fn extract_spread(ask: f64, bid: f64) -> f64 {
        (ask - bid).abs()
    }

    pub fn extract_pl(entry: f64, exit: f64, direction: &str, lot: f64) -> f64 {
        let diff = match direction {
            "buy" => exit - entry,
            "sell" => entry - exit,
            _ => 0.0,
        };
        diff * lot
    }

    pub fn trade_count(&self) -> usize {
        self.trades.len()
    }

    pub fn recent_trades(&self, n: usize) -> &[TradeRecord] {
        let len = self.trades.len();
        let start = len.saturating_sub(n);
        &self.trades[start..]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_record_trade() {
        let mut journal = TradeJournal::new();
        let record = TradeRecord {
            id: "t1".into(), symbol: "US100.cash".into(), direction: "buy".into(),
            entry_price: 100.0, exit_price: None, sl: 99.0, tp: 103.0,
            slippage: None, spread: None, pl: None,
            confidence: 1.0, gate_result: 5, mode: "SNIPER".into(),
            session_phase: "NORMAL".into(), entry_reason: Some("breakout".into()),
            exit_reason: None,
        };
        journal.record_trade(record);
        assert_eq!(journal.trade_count(), 1);
    }

    #[test]
    fn test_extract_slippage() {
        let s = TradeJournal::extract_slippage(100.0, 100.5);
        assert!((s - 0.5).abs() < 1e-6);
    }

    #[test]
    fn test_extract_spread() {
        let s = TradeJournal::extract_spread(100.5, 100.3);
        assert!((s - 0.2).abs() < 1e-6);
    }

    #[test]
    fn test_extract_pl_buy() {
        let pl = TradeJournal::extract_pl(100.0, 103.0, "buy", 1.0);
        assert!((pl - 3.0).abs() < 1e-6);
    }

    #[test]
    fn test_extract_pl_sell() {
        let pl = TradeJournal::extract_pl(100.0, 97.0, "sell", 1.0);
        assert!((pl - 3.0).abs() < 1e-6);
    }

    #[test]
    fn test_extract_pl_loss() {
        let pl = TradeJournal::extract_pl(100.0, 97.0, "buy", 1.0);
        assert!((pl + 3.0).abs() < 1e-6);
    }

    #[test]
    fn test_recent_trades() {
        let mut journal = TradeJournal::new();
        for i in 0..5 {
            journal.record_trade(TradeRecord {
                id: format!("t{}", i), symbol: "US100.cash".into(), direction: "buy".into(),
                entry_price: 100.0 + i as f64, exit_price: None, sl: 99.0, tp: 103.0,
                slippage: None, spread: None, pl: None, confidence: 1.0, gate_result: 5,
                mode: "SNIPER".into(), session_phase: "NORMAL".into(),
                entry_reason: None, exit_reason: None,
            });
        }
        assert_eq!(journal.recent_trades(3).len(), 3);
    }
}
