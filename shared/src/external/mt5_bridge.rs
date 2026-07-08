// MT5 API bridge via PyO3
// Calls python/seith_bridge/mt5.py for broker interaction

use anyhow::Result;

pub struct Mt5Api {
    symbol: String,
}

impl Mt5Api {
    pub fn new(symbol: &str) -> Self {
        Self {
            symbol: symbol.to_string(),
        }
    }

    pub async fn connect(&self) -> Result<()> {
        log::info!("MT5 connecting to {}", self.symbol);

        let account_str = std::env::var("MT5_ACCOUNT")?;
        let account: i64 = account_str.parse()?;
        let password = std::env::var("MT5_PASSWORD")?;
        let server = std::env::var("MT5_SERVER")?;
        let path = std::env::var("MT5_PATH").unwrap_or_default();

        pyo3::Python::with_gil(|py| {
            let mt5 = pyo3::types::PyModule::import(py, "seith_bridge.mt5")
                .map_err(|e| anyhow::anyhow!("PyO3 import mt5: {}", e))?;

            // 1. Initialize
            let init_ok: bool = mt5
                .call_method1("init_mt5", (path,))
                .map_err(|e| anyhow::anyhow!("PyO3 init_mt5: {}", e))?
                .extract()?;
            if !init_ok {
                anyhow::bail!("Failed to initialize MetaTrader 5 terminal");
            }

            // 2. Login
            let login_ok: bool = mt5
                .call_method1("login", (account, password, server))
                .map_err(|e| anyhow::anyhow!("PyO3 login: {}", e))?
                .extract()?;
            if !login_ok {
                anyhow::bail!("Failed to login to MT5 account");
            }

            Ok::<_, anyhow::Error>(())
        })?;

        log::info!("MT5 connected and authorized");
        Ok(())
    }

    pub async fn get_price(&self) -> Result<f64> {
        pyo3::Python::with_gil(|py| {
            let mt5 = pyo3::types::PyModule::import(py, "seith_bridge.mt5")?;
            let price: Option<f64> = mt5.call_method1("get_price", (&self.symbol,))?.extract()?;
            price.ok_or_else(|| anyhow::anyhow!("No price data for {}", self.symbol))
        })
    }

    pub async fn get_account(&self) -> Result<i64> {
        // Expose username/account from settings
        let account_str = std::env::var("MT5_ACCOUNT")?;
        let account: i64 = account_str.parse()?;
        Ok(account)
    }

    pub async fn place_order(
        &self,
        order_type: &str,
        volume: f64,
        price: f64,
        sl: f64,
        tp: f64,
    ) -> Result<u64> {
        let mt5_type = match order_type {
            "BUY" => 0,  // mt5.ORDER_TYPE_BUY
            "SELL" => 1, // mt5.ORDER_TYPE_SELL
            _ => anyhow::bail!("Invalid order type: {}", order_type),
        };
        pyo3::Python::with_gil(|py| {
            let mt5 = pyo3::types::PyModule::import(py, "seith_bridge.mt5")?;
            let ticket: Option<i64> = mt5
                .call_method1(
                    "place_order",
                    (&self.symbol, mt5_type, volume, price, sl, tp),
                )?
                .extract()?;
            ticket
                .map(|t| t as u64)
                .ok_or_else(|| anyhow::anyhow!("Order placement returned None"))
        })
    }
}
