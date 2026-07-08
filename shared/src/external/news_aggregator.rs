// News aggregator — dual-parallel async calendar scraper
// Uses Python scraper via PyO3 for ForexFactory + Investing.com

use anyhow::Result;
use chrono::{DateTime, TimeZone, Utc};
use serde::Deserialize;
use tokio::task;

#[derive(Debug, Clone, Deserialize)]
pub struct RawNewsEvent {
    pub time: String,
    pub currency: String,
    pub impact: String,
    pub title: String,
    pub actual: String,
    pub forecast: String,
    pub previous: String,
}

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

fn parse_news_json(json_str: &str) -> Result<Vec<NewsEvent>> {
    let raw: Vec<RawNewsEvent> = serde_json::from_str(json_str)?;
    let now = Utc::now();
    Ok(raw
        .into_iter()
        .map(|r| {
            let parsed_time = chrono::NaiveDateTime::parse_from_str(&r.time, "%Y-%m-%d %H:%M:%S")
                .ok()
                .and_then(|t| Utc.from_local_datetime(&t).single())
                .unwrap_or(now);
            NewsEvent {
                time: parsed_time,
                currency: r.currency,
                impact: r.impact,
                title: r.title,
                actual: if r.actual.is_empty() || r.actual == "-" {
                    None
                } else {
                    Some(r.actual)
                },
                forecast: if r.forecast.is_empty() || r.forecast == "-" {
                    None
                } else {
                    Some(r.forecast)
                },
                previous: if r.previous.is_empty() || r.previous == "-" {
                    None
                } else {
                    Some(r.previous)
                },
            }
        })
        .collect())
}

pub async fn fetch_forex_factory() -> Result<Vec<NewsEvent>> {
    let url = "https://www.forexfactory.com/calendar";
    let result = task::spawn_blocking(move || {
        pyo3::Python::with_gil(|py| {
            let scraper = pyo3::types::PyModule::import(py, "seith_bridge.scraper")?;
            let json_str: Option<String> = scraper
                .call_method1("fetch_forex_factory", (url,))?
                .extract()?;
            Ok::<_, anyhow::Error>(json_str)
        })
    })
    .await??;

    match result {
        Some(json) => {
            let events = parse_news_json(&json)?;
            log::info!("ForexFactory: {} events fetched", events.len());
            Ok(events)
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
            let json_str: Option<String> = scraper
                .call_method1("fetch_investing_com", (url,))?
                .extract()?;
            Ok::<_, anyhow::Error>(json_str)
        })
    })
    .await??;

    match result {
        Some(json) => {
            let events = parse_news_json(&json)?;
            log::info!("Investing.com: {} events fetched", events.len());
            Ok(events)
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
