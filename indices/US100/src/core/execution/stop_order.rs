use anyhow::Result;

#[derive(Debug, Clone)]
pub struct StopOrderParams {
    pub side: String,
    pub entry: f64,
    pub sl: f64,
    pub tp: f64,
    pub lot: f64,
}

pub struct StopOrder;

impl StopOrder {
    pub fn new() -> Self {
        Self
    }

    pub fn calculate_breakout_entry(consolidation_high: f64, consolidation_low: f64, buffer: f64) -> (f64, f64) {
        let buy_stop = consolidation_high + buffer;
        let sell_stop = consolidation_low - buffer;
        (buy_stop, sell_stop)
    }

    pub async fn place(&self, params: &StopOrderParams) -> Result<()> {
        log::info!("[StopOrder] Placing {} stop: entry={}, SL={}, TP={}, lot={}",
            params.side, params.entry, params.sl, params.tp, params.lot);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_calculate_breakout_entry() {
        let (buy, sell) = StopOrder::calculate_breakout_entry(100.0, 98.0, 2.0);
        assert!((buy - 102.0).abs() < 1e-6);
        assert!((sell - 96.0).abs() < 1e-6);
    }

    #[test]
    fn test_calculate_breakout_no_buffer() {
        let (buy, sell) = StopOrder::calculate_breakout_entry(100.0, 98.0, 0.0);
        assert!((buy - 100.0).abs() < 1e-6);
        assert!((sell - 98.0).abs() < 1e-6);
    }

    #[test]
    fn test_stop_order_params() {
        let p = StopOrderParams { side: "buy".into(), entry: 102.0, sl: 99.0, tp: 106.0, lot: 1.0 };
        assert_eq!(p.side, "buy");
        assert!((p.entry - 102.0).abs() < 1e-6);
    }
}
