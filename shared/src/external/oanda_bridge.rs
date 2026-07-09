// OANDA REST API sentiment bridge via PyO3
// Fetches position book (% long/short) for Bayesian prior P(A)

use anyhow::Result;

/// Fetch OANDA sentiment as JSON string.
/// Returns: {"long_pct": 65.0, "short_pct": 35.0, "time": "..."}
pub async fn fetch_sentiment_json(instrument: &str) -> Result<String> {
    tokio::task::spawn_blocking({
        let inst = instrument.to_string();
        move || {
            pyo3::Python::with_gil(|py| {
                let oanda = pyo3::types::PyModule::import(py, "seith_bridge.oanda")
                    .map_err(|e| anyhow::anyhow!("PyO3 import oanda: {}", e))?;

                let result: String = oanda.call_method1("get_sentiment", (&inst,))?.extract()?;

                Ok(result)
            })
        }
    })
    .await?
}

/// Initialize OANDA client (called once at startup). Sync, run in spawn_blocking.
pub async fn init_oanda(token: &str, account_id: &str, practice: bool) -> Result<bool> {
    let token = token.to_string();
    let account_id = account_id.to_string();
    tokio::task::spawn_blocking(move || {
        pyo3::Python::with_gil(|py| {
            let oanda = pyo3::types::PyModule::import(py, "seith_bridge.oanda")?;
            let ok: bool = oanda
                .call_method1("init_oanda", (token, account_id, practice))?
                .extract()?;
            Ok(ok)
        })
    })
    .await?
}
