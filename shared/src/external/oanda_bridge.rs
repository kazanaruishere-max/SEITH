// Sentiment bridge via PyO3
// Fetches % long/short from Myfxbook (free) or OANDA API for Bayesian prior

use anyhow::Result;

/// Fetch market sentiment as JSON string.
/// Returns: {"long_pct": 65.0, "short_pct": 35.0, "source": "myfxbook"}
pub async fn fetch_sentiment_json(instrument: &str) -> Result<String> {
    let inst = instrument.to_string();
    tokio::task::spawn_blocking(move || {
        pyo3::Python::with_gil(|py| {
            let mod_sentiment = pyo3::types::PyModule::import(py, "seith_bridge.oanda")
                .map_err(|e| anyhow::anyhow!("PyO3 import oanda: {}", e))?;

            let result: String = mod_sentiment
                .call_method1("get_sentiment", (&inst,))?
                .extract()?;

            Ok(result)
        })
    })
    .await?
}
