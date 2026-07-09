// Load OHLCV M1 data from CSV
// Columns: time,open,high,low,close,volume

use anyhow::{Context, Result};

#[derive(Debug, Clone)]
pub struct M1Candle {
    pub time: i64,
    pub open: f64,
    pub high: f64,
    pub low: f64,
    pub close: f64,
    pub volume: f64,
}

pub fn load_ohlcv_csv(path: &str) -> Result<Vec<M1Candle>> {
    let content =
        std::fs::read_to_string(path).with_context(|| format!("Cannot read: {}", path))?;

    let mut candles = Vec::new();
    for line in content.lines().skip(1) {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        let parts: Vec<&str> = line.split(',').collect();
        if parts.len() < 6 {
            continue;
        }

        let time: i64 = parts[0]
            .trim()
            .parse()
            .or_else(|_| {
                chrono::DateTime::parse_from_rfc3339(parts[0].trim()).map(|dt| dt.timestamp())
            })
            .context("Invalid time")?;

        candles.push(M1Candle {
            time,
            open: parts[1].trim().parse()?,
            high: parts[2].trim().parse()?,
            low: parts[3].trim().parse()?,
            close: parts[4].trim().parse()?,
            volume: parts[5].trim().parse()?,
        });
    }

    if candles.is_empty() {
        anyhow::bail!("No candles loaded from {}", path);
    }
    candles.sort_by_key(|c| c.time);
    Ok(candles)
}
