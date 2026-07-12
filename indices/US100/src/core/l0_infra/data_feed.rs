use crate::config::settings::SYMBOL;
use crate::external::oanda_feed::OandaFeed;
use anyhow::Result;
use std::time::Duration;

const MAX_RETRIES: u32 = 3;
const STANDBY_SECS: u64 = 60;

pub enum DataFeedStatus {
    Connected,
    Standby(String),
    Disconnected,
}

pub struct DataFeed {
    feed: OandaFeed,
    status: DataFeedStatus,
    prices: Vec<f64>,
}

impl DataFeed {
    pub fn new() -> Self {
        Self {
            feed: OandaFeed::new(),
            status: DataFeedStatus::Standby("initializing".into()),
            prices: Vec::with_capacity(100),
        }
    }

    pub async fn connect(&mut self) -> Result<()> {
        for attempt in 1..=MAX_RETRIES {
            match self.feed.connect().await {
                Ok(()) => {
                    self.status = DataFeedStatus::Connected;
                    log::info!("[DataFeed] Connected to OANDA {} (attempt {})", SYMBOL, attempt);
                    return Ok(());
                }
                Err(e) => {
                    log::warn!("[DataFeed] Connect attempt {}/{} failed: {}", attempt, MAX_RETRIES, e);
                    if attempt < MAX_RETRIES {
                        tokio::time::sleep(Duration::from_secs(2u64.pow(attempt))).await;
                    }
                }
            }
        }
        self.status = DataFeedStatus::Disconnected;
        log::error!("[DataFeed] All {} connect attempts failed. Entering standby.", MAX_RETRIES);
        tokio::time::sleep(Duration::from_secs(STANDBY_SECS)).await;
        Err(anyhow::anyhow!("OANDA connection failed after {} retries", MAX_RETRIES))
    }

    pub fn status(&self) -> &DataFeedStatus {
        &self.status
    }

    pub async fn fetch_ohlcv(&mut self) -> Result<Vec<f64>> {
        let prices = self.feed.get_prices().await?;
        self.prices.extend(&prices);
        if self.prices.len() > 1000 {
            self.prices.drain(0..self.prices.len() - 500);
        }
        Ok(prices)
    }

    pub async fn fetch_us10y_yield(&self) -> Result<f64> {
        self.feed.get_yield("US10YB").await
    }

    pub async fn fetch_us02y_yield(&self) -> Result<f64> {
        self.feed.get_yield("US02YB").await
    }

    pub fn get_price_history(&self) -> &[f64] {
        &self.prices
    }

    pub fn compute_hv_zscore(&self, window: usize) -> f64 {
        let prices = &self.prices;
        if prices.len() < window + 1 {
            return 0.0;
        }
        let returns: Vec<f64> = prices.windows(2)
            .map(|w| (w[1] - w[0]) / w[0])
            .collect();
        let recent = &returns[returns.len().saturating_sub(window)..];
        let n = recent.len() as f64;
        if n < 2.0 {
            return 0.0;
        }
        let mean = recent.iter().sum::<f64>() / n;
        let variance = recent.iter().map(|r| (r - mean).powi(2)).sum::<f64>() / (n - 1.0);
        let std_dev = variance.sqrt();
        if std_dev < 1e-10 {
            return 0.0;
        }
        let last_return = *recent.last().unwrap_or(&0.0);
        (last_return - mean) / std_dev
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hv_zscore_empty() {
        let feed = DataFeed::new();
        assert!((feed.compute_hv_zscore(10) - 0.0).abs() < 1e-6);
    }

    #[test]
    fn test_hv_zscore_constant() {
        let mut feed = DataFeed::new();
        feed.prices = vec![100.0; 20];
        assert!((feed.compute_hv_zscore(10) - 0.0).abs() < 1e-6);
    }

    #[test]
    fn test_hv_zscore_trend() {
        let mut feed = DataFeed::new();
        feed.prices = (0..20).map(|i| 100.0 + i as f64).collect();
        let z = feed.compute_hv_zscore(10);
        assert!(z.is_finite());
    }
}
