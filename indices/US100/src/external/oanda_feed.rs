use anyhow::Result;

pub struct OandaFeed;

impl OandaFeed {
    pub fn new() -> Self {
        Self
    }

    pub async fn connect(&mut self) -> Result<()> {
        Ok(())
    }

    pub async fn get_balance(&self) -> Result<f64> {
        Ok(0.0)
    }

    pub async fn get_prices(&self) -> Result<Vec<f64>> {
        Ok(Vec::new())
    }

    pub async fn get_yield(&self, _symbol: &str) -> Result<f64> {
        Ok(0.0)
    }

    pub async fn place_order(&self) -> Result<()> {
        Ok(())
    }

    pub async fn cancel_order(&self) -> Result<()> {
        Ok(())
    }

    pub async fn get_open_trades(&self) -> Result<Vec<String>> {
        Ok(Vec::new())
    }
}
