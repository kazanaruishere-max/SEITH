// Backtest engine — replay historical OHLCV through L1→Execution pipeline
// Full train/test split + tick-level backtest engine

mod data_loader;
mod reporter;
mod simulator;
pub mod tick_data;
pub mod tick_engine;

pub use data_loader::{load_ohlcv_csv, BacktestData};
pub use reporter::BacktestReport;
pub use simulator::BacktestEngine;

use anyhow::Result;
use std::path::Path;

/// Run backtest on specific data segment.
/// segment: "all", "train", or "test"
pub async fn run_backtest_segment(
    csv_path: &str,
    output_dir: &str,
    train_ratio: f64,
    segment: &str,
) -> Result<BacktestReport> {
    let data = load_ohlcv_csv(csv_path, train_ratio)?;
    let candles = match segment {
        "train" => &data.train,
        "test" => &data.test,
        _ => &data.all,
    };

    log::info!(
        "Backtest {} on {} M1 candles (split at {})",
        segment,
        candles.len(),
        format_ts(data.split_time),
    );

    let mut engine = BacktestEngine::new();
    let is = segment != "test"; // train and all are in-sample
    engine.run_on(candles, is).await;

    let report = engine.report(&data.all);
    let report_path = Path::new(output_dir).join(format!("backtest_report_{}.json", segment));
    let json = serde_json::to_string_pretty(&report)?;
    std::fs::write(&report_path, &json)?;
    log::info!("Report written to {}", report_path.display());

    let csv_path = Path::new(output_dir).join(format!("trades_{}.csv", segment));
    let mut csv =
        String::from("time,tier,direction,entry,exit,pips,result,spread,slippage,in_sample\n");
    for t in &engine.trades {
        csv.push_str(&format!(
            "{},{},{},{},{},{},{},{},{},{}\n",
            t.time,
            t.tier,
            t.direction,
            t.entry,
            t.exit,
            t.pips,
            t.result,
            t.spread,
            t.slippage,
            t.in_sample as u8,
        ));
    }
    std::fs::write(&csv_path, csv)?;
    log::info!("Trade log written to {}", csv_path.display());

    Ok(report)
}

/// Legacy: run full backtest on all data
pub async fn run_backtest(
    csv_path: &str,
    output_dir: &str,
    train_ratio: f64,
) -> Result<BacktestReport> {
    run_backtest_segment(csv_path, output_dir, train_ratio, "all").await
}

fn format_ts(ts: i64) -> String {
    chrono::DateTime::from_timestamp(ts, 0)
        .map(|d| d.format("%Y-%m-%d").to_string())
        .unwrap_or_default()
}
