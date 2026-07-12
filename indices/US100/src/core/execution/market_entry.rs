use anyhow::Result;

#[derive(Debug, Clone)]
pub struct MarketOrderParams {
    pub side: String,
    pub sl: f64,
    pub tp: f64,
    pub lot: f64,
}

pub struct MarketEntry;

impl MarketEntry {
    pub fn new() -> Self {
        Self
    }

    pub async fn execute(&self, params: &MarketOrderParams) -> Result<()> {
        log::info!("[MarketEntry] Executing {} market: SL={}, TP={}, lot={}",
            params.side, params.sl, params.tp, params.lot);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_market_order_params() {
        let p = MarketOrderParams { side: "buy".into(), sl: 99.0, tp: 105.0, lot: 0.5 };
        assert_eq!(p.side, "buy");
        assert!((p.lot - 0.5).abs() < 1e-6);
    }
}
