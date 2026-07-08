// News aggregator — economic calendar via Python bridge
// Multi-source: cloudscraper (ForexFactory) → TradingEconomics API → schedule fallback

use anyhow::Result;
use chrono::{DateTime, NaiveDateTime, TimeZone, Utc};
use serde::Deserialize;
use tokio::task;

#[derive(Debug, Clone, Deserialize)]
pub struct RawCalendarEvent {
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

fn parse_calendar_json(json_str: &str) -> Result<Vec<NewsEvent>> {
    let raw: Vec<RawCalendarEvent> = serde_json::from_str(json_str)?;
    let now = Utc::now();
    Ok(raw
        .into_iter()
        .map(|r| {
            let parsed = parse_ff_time(&r.time).unwrap_or(now);
            NewsEvent {
                time: parsed,
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

fn parse_ff_time(s: &str) -> Option<DateTime<Utc>> {
    let now = Utc::now();
    let today = now.format("%Y-%m-%d").to_string();
    let s = s.trim();

    // "2026-07-08 8:45p" or "2026-07-08 10:00"
    if let Ok(dt) = NaiveDateTime::parse_from_str(s, "%Y-%m-%d %H:%M") {
        return Some(Utc.from_utc_datetime(&dt));
    }
    if let Ok(dt) = NaiveDateTime::parse_from_str(&format!("{} {}", today, s), "%Y-%m-%d %H:%M") {
        return Some(Utc.from_utc_datetime(&dt));
    }
    // "8:45p" → 20:45
    if let Ok(dt) = NaiveDateTime::parse_from_str(&format!("{} {}", today, s), "%Y-%m-%d %I:%M%p") {
        return Some(Utc.from_utc_datetime(&dt));
    }
    if let Ok(dt) = NaiveDateTime::parse_from_str(&format!("{} {}", today, s), "%Y-%m-%d %H:%M") {
        return Some(Utc.from_utc_datetime(&dt));
    }
    None
}

pub async fn fetch_calendar() -> Result<Vec<NewsEvent>> {
    let result = task::spawn_blocking(|| {
        pyo3::Python::with_gil(|py| {
            let cal = pyo3::types::PyModule::import(py, "seith_bridge.calendar")?;
            let json_str: Option<String> =
                cal.call_method0("fetch_economic_calendar")?.extract()?;
            Ok::<_, anyhow::Error>(json_str)
        })
    })
    .await??;

    match result {
        Some(json) => {
            let events = parse_calendar_json(&json)?;
            log::info!("Calendar: {} events fetched", events.len());
            Ok(events)
        }
        None => {
            log::warn!("Calendar returned no data");
            Ok(vec![])
        }
    }
}

pub async fn has_red_folder_soon() -> Result<bool> {
    let events = fetch_calendar().await?;
    let now = Utc::now();
    let has_red = events.iter().any(|e| {
        let window_start = now + chrono::Duration::minutes(30);
        let window_end = now + chrono::Duration::minutes(60);
        e.time >= window_start && e.time <= window_end
    });
    Ok(has_red)
}
