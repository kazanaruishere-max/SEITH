// L0 - Data Feed
// Raw price streaming from MT5
// Stub only - no implementation yet

use anyhow::Result;

#[derive(Debug, Clone)]
pub struct PriceTick {
    pub symbol: String,
    pub bid: f64,
    pub ask: f64,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

/// Initialize data feed
pub async fn init() -> Result<()> {
    log::info!("L0 Data Feed initialized (stub)");
    Ok(())
}

/// Get current price tick
pub async fn get_current_tick(symbol: &str) -> Result<PriceTick> {
    log::debug!("Getting current tick for {} (stub)", symbol);
    todo!("Implement MT5 price streaming")
}
