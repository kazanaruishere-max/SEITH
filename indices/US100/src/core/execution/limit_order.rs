use anyhow::Result;

#[derive(Debug, Clone)]
pub struct LimitOrderParams {
    pub side: String,
    pub limit_price: f64,
    pub sl: f64,
    pub tp: f64,
    pub lot: f64,
}

pub struct LimitOrder;

impl LimitOrder {
    pub fn new() -> Self {
        Self
    }

    pub fn calculate_pullback_entry(frama_level: f64, support: f64, buffer: f64) -> (f64, f64) {
        let buy_limit = (frama_level.max(support)) - buffer;
        let sell_limit = (frama_level.min(support)) + buffer;
        (buy_limit, sell_limit)
    }

    pub async fn place(&self, params: &LimitOrderParams) -> Result<()> {
        log::info!("[LimitOrder] Placing {} limit: price={}, SL={}, TP={}, lot={}",
            params.side, params.limit_price, params.sl, params.tp, params.lot);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_calculate_pullback_entry_buy() {
        let (buy, _) = LimitOrder::calculate_pullback_entry(100.0, 98.0, 1.0);
        assert!((buy - 99.0).abs() < 1e-6);
    }

    #[test]
    fn test_calculate_pullback_entry_sell() {
        let (_, sell) = LimitOrder::calculate_pullback_entry(100.0, 102.0, 1.0);
        assert!((sell - 101.0).abs() < 1e-6);
    }

    #[test]
    fn test_limit_order_params() {
        let p = LimitOrderParams { side: "sell".into(), limit_price: 101.0, sl: 103.0, tp: 99.0, lot: 1.0 };
        assert_eq!(p.side, "sell");
    }
}
