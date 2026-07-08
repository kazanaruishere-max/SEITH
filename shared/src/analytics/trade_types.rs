// Trade types shared across all instruments

#[derive(Debug, Clone)]
pub struct TradeRecord {
    pub id: String,
    pub instrument: String,
    pub signal_tier: String,
    pub order_type: String,
    pub direction: String,
    pub entry_price: f64,
    pub stop_loss: f64,
    pub take_profit: f64,
    pub volume: f64,
    pub status: String,
    pub result: Option<String>,
    pub slippage: Option<f64>,
    pub spread_open: Option<f64>,
    pub profit_cent: Option<f64>,
    pub opened_at: String,
    pub closed_at: Option<String>,
}

impl TradeRecord {
    pub fn new(
        instrument: &str,
        tier: &str,
        direction: &str,
        entry: f64,
        sl: f64,
        tp: f64,
        spread: f64,
    ) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            instrument: instrument.to_string(),
            signal_tier: tier.to_string(),
            order_type: String::new(),
            direction: direction.to_string(),
            entry_price: entry,
            stop_loss: sl,
            take_profit: tp,
            volume: 0.01,
            status: "PENDING".to_string(),
            result: None,
            slippage: None,
            spread_open: Some(spread),
            profit_cent: None,
            opened_at: chrono::Utc::now().to_rfc3339(),
            closed_at: None,
        }
    }
}
