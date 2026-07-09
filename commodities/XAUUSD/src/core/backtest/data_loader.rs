// Load OHLCV M1 data from CSV with train/test split
// Expected columns: time,open,high,low,close,volume
// Optional: load CVD from separate CSV (minute_ts,cvd)

use anyhow::{Context, Result};
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct M1Candle {
    pub time: i64,
    pub open: f64,
    pub high: f64,
    pub low: f64,
    pub close: f64,
    pub volume: f64,
    /// Cumulative volume delta from tick-level bid/ask classification.
    /// None if CVD data not available.
    pub cvd: Option<i32>,
}

#[derive(Debug, Clone)]
pub struct BacktestData {
    pub all: Vec<M1Candle>,
    pub train: Vec<M1Candle>,
    pub test: Vec<M1Candle>,
    pub split_time: i64,
    pub cvd_available: bool,
}

/// Load OHLCV and split chronologically: train_ratio (0.0-1.0) goes to train.
pub fn load_ohlcv_csv(path: &str, train_ratio: f64) -> Result<BacktestData> {
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
            cvd: None,
        });
    }

    if candles.is_empty() {
        anyhow::bail!("No candles loaded from {}", path);
    }
    candles.sort_by_key(|c| c.time);

    // Optional: load CVD data
    let cvd_map = load_cvd_csv(
        "C:/Users/Lenovo/PROJECT/AI SEITH/jupyter/backtest_analysis/xauusd_m1_cvd.csv",
    );

    if !cvd_map.is_empty() {
        for candle in &mut candles {
            // Match M1 bar to closest minute
            let minute_ts = candle.time / 60;
            if let Some(&cvd_val) = cvd_map.get(&minute_ts) {
                candle.cvd = Some(cvd_val);
            }
        }
    }

    let cvd_available = candles.iter().any(|c| c.cvd.is_some());

    // Chronological split
    let split_idx = (candles.len() as f64 * train_ratio).floor() as usize;
    let split_idx = split_idx.max(1).min(candles.len() - 1);
    let split_time = candles[split_idx].time;

    let train = candles[..split_idx].to_vec();
    let test = candles[split_idx..].to_vec();

    Ok(BacktestData {
        all: candles,
        train,
        test,
        split_time,
        cvd_available,
    })
}

fn load_cvd_csv(path: &str) -> HashMap<i64, i32> {
    let content = match std::fs::read_to_string(path) {
        Ok(c) => c,
        Err(_) => return HashMap::new(),
    };

    let mut map = HashMap::new();
    for line in content.lines().skip(1) {
        let parts: Vec<&str> = line.split(',').collect();
        if parts.len() < 2 {
            continue;
        }
        if let (Ok(ts), Ok(val)) = (
            parts[0].trim().parse::<i64>(),
            parts[1].trim().parse::<i32>(),
        ) {
            map.insert(ts, val);
        }
    }
    map
}
