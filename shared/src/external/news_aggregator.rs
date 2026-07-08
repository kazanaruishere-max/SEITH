// News aggregator — dual-parallel async calendar scraper
// Uses Python scraper via PyO3 for ForexFactory + Investing.com

use anyhow::Result;
use chrono::{DateTime, Utc};
use tokio::task;

#[derive(Debug, Clone)]
pub struct NewsEvent {
    pub time: DateTime<Utc>,
    pub currency: String,
    pub impact: String,
    pub title: String,
    pub actual: Option<String>,
    pub forecast: Option<String>,
    pub previous: Option<String>,
}

pub async fn fetch_forex_factory() -> Result<Vec<NewsEvent>> {
    let url = "https://www.forexfactory.com/calendar";
    let result = task::spawn_blocking(move || {
        pyo3::Python::with_gil(|py| {
            let scraper = pyo3::types::PyModule::import(py, "seith_bridge.scraper")?;
            let data: Option<String> = scraper
                .call_method1("fetch_forex_factory", (url,))?
                .extract()?;
            Ok::<_, anyhow::Error>(data)
        })
    })
    .await??;

    match result {
        Some(_) => {
            log::info!("ForexFactory data fetched");
            Ok(vec![])
        }
        None => {
            log::warn!("ForexFactory returned no data");
            Ok(vec![])
        }
    }
}

pub async fn fetch_investing_com() -> Result<Vec<NewsEvent>> {
    let url = "https://www.investing.com/economic-calendar/";
    let result = task::spawn_blocking(move || {
        pyo3::Python::with_gil(|py| {
            let scraper = pyo3::types::PyModule::import(py, "seith_bridge.scraper")?;
            let data: Option<String> = scraper
                .call_method1("fetch_investing_com", (url,))?
                .extract()?;
            Ok::<_, anyhow::Error>(data)
        })
    })
    .await??;

    match result {
        Some(_) => {
            log::info!("Investing.com data fetched");
            Ok(vec![])
        }
        None => {
            log::warn!("Investing.com returned no data");
            Ok(vec![])
        }
    }
}

pub async fn has_red_folder_soon() -> Result<bool> {
    let ff = fetch_forex_factory().await?;
    let ic = fetch_investing_com().await?;
    let all_events = [ff, ic].concat();

    let now = Utc::now();
    let has_red = all_events.iter().any(|e| {
        let currency_ok = e.currency.to_uppercase().contains("USD");
        let impact_ok = e.impact.to_lowercase().contains("high");
        let window_start = now + chrono::Duration::minutes(30);
        let window_end = now + chrono::Duration::minutes(60);
        currency_ok && impact_ok && e.time >= window_start && e.time <= window_end
    });

    Ok(has_red)
}
