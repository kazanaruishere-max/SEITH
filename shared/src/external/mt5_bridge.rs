// MT5 API bridge via PyO3
// Calls python/seith_bridge/mt5.py for broker interaction

use anyhow::Result;

/// Raw DOM level from Python bridge: (price, volume, mt5_type)
pub type DomRawLevel = (f64, u64, i32);

/// Full tick data: (bid, ask, spread)
#[derive(Debug, Clone)]
pub struct TickData {
    pub bid: f64,
    pub ask: f64,
    pub spread: f64,
}

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

            let init_ok: bool = mt5
                .call_method1("init_mt5", (path,))
                .map_err(|e| anyhow::anyhow!("PyO3 init_mt5: {}", e))?
                .extract()?;
            if !init_ok {
                anyhow::bail!("Failed to initialize MetaTrader 5 terminal");
            }

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

    /// Fetch live tick with bid/ask/spread via JSON bridge
    pub async fn get_tick(&self) -> Result<TickData> {
        let json_str: Option<String> = pyo3::Python::with_gil(|py| {
            let mt5 = pyo3::types::PyModule::import(py, "seith_bridge.mt5")?;
            mt5.call_method1("get_tick_json", (&self.symbol,))?
                .extract()
        })?;
        match json_str {
            Some(s) => {
                let v: serde_json::Value = serde_json::from_str(&s)?;
                let bid = v["bid"].as_f64().unwrap_or(0.0);
                let ask = v["ask"].as_f64().unwrap_or(0.0);
                Ok(TickData {
                    bid,
                    ask,
                    spread: (ask - bid).max(0.0),
                })
            }
            None => anyhow::bail!("No tick data for {}", self.symbol),
        }
    }

    /// Fetch Depth of Market via JSON bridge.
    /// Returns Vec<(price, volume, mt5_type)> where type=1=ASK, type=2=BID.
    pub async fn get_dom_raw(&self) -> Result<Vec<DomRawLevel>> {
        let json_str: Option<String> = pyo3::Python::with_gil(|py| {
            let mt5 = pyo3::types::PyModule::import(py, "seith_bridge.mt5")?;
            mt5.call_method1("get_dom_json", (&self.symbol,))?.extract()
        })?;
        match json_str {
            Some(s) => {
                let v: serde_json::Value = serde_json::from_str(&s)?;
                let mut levels = Vec::new();
                if let Some(asks) = v["asks"].as_array() {
                    for ask in asks {
                        levels.push((
                            ask["price"].as_f64().unwrap_or(0.0),
                            ask["volume"].as_i64().unwrap_or(0) as u64,
                            1,
                        ));
                    }
                }
                if let Some(bids) = v["bids"].as_array() {
                    for bid in bids {
                        levels.push((
                            bid["price"].as_f64().unwrap_or(0.0),
                            bid["volume"].as_i64().unwrap_or(0) as u64,
                            2,
                        ));
                    }
                }
                Ok(levels)
            }
            None => anyhow::bail!("No DOM data for {}", self.symbol),
        }
    }

    pub async fn get_account(&self) -> Result<i64> {
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
            "BUY" => 0,
            "SELL" => 1,
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

    /// Place pending order (BUY_LIMIT / SELL_LIMIT / BUY_STOP / SELL_STOP).
    /// Hanya pending order — HARAM market order untuk PF 4.0.
    pub async fn place_pending_limit(
        &self,
        order_type: &str,
        volume: f64,
        price: f64,
        sl: f64,
        tp: f64,
    ) -> Result<u64> {
        let mt5_type = match order_type {
            "BUY_LIMIT" => 2,
            "SELL_LIMIT" => 3,
            "BUY_STOP" => 4,
            "SELL_STOP" => 5,
            _ => anyhow::bail!("Invalid pending order type: {}", order_type),
        };
        pyo3::Python::with_gil(|py| {
            let mt5 = pyo3::types::PyModule::import(py, "seith_bridge.mt5")?;
            let ticket: Option<i64> = mt5
                .call_method1(
                    "place_pending_order",
                    (&self.symbol, mt5_type, volume, price, sl, tp),
                )?
                .extract()?;
            ticket
                .map(|t| t as u64)
                .ok_or_else(|| anyhow::anyhow!("Pending order failed"))
        })
    }

    /// Fetch recent M1 OHLCV rates for chart data via JSON bridge
    pub async fn get_rates_raw(&self, count: i32) -> Result<Vec<(i64, f64, f64, f64, f64, f64)>> {
        let json_str: Option<String> = pyo3::Python::with_gil(|py| {
            let mt5 = pyo3::types::PyModule::import(py, "seith_bridge.mt5")?;
            mt5.call_method1("get_rates_json", (&self.symbol, count, 1))?
                .extract()
        })?;
        match json_str {
            Some(s) => {
                let parsed: serde_json::Value = serde_json::from_str(&s)?;
                let mut candles = Vec::new();
                if let Some(arr) = parsed.as_array() {
                    for item in arr {
                        candles.push((
                            item["time"].as_i64().unwrap_or(0),
                            item["open"].as_f64().unwrap_or(0.0),
                            item["high"].as_f64().unwrap_or(0.0),
                            item["low"].as_f64().unwrap_or(0.0),
                            item["close"].as_f64().unwrap_or(0.0),
                            item["volume"].as_f64().unwrap_or(0.0),
                        ));
                    }
                }
                Ok(candles)
            }
            None => anyhow::bail!("No rates for {}", self.symbol),
        }
    }
}
