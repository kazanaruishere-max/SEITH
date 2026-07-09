// Backtest engine — replay historical OHLCV through L1→Execution pipeline
// Output: trade log + performance report for Jupyter analysis

mod data_loader;
mod reporter;
mod simulator;

pub use data_loader::load_ohlcv_csv;
pub use reporter::BacktestReport;
pub use simulator::BacktestEngine;

use anyhow::Result;
use std::path::Path;

/// Run full backtest from CSV → report
pub async fn run_backtest(csv_path: &str, output_dir: &str) -> Result<BacktestReport> {
    let candles = load_ohlcv_csv(csv_path)?;
    log::info!("Loaded {} M1 candles from {}", candles.len(), csv_path);

    let mut engine = BacktestEngine::new();
    engine.run(&candles).await?;

    let report = engine.report();
    let report_path = Path::new(output_dir).join("backtest_report.json");
    let json = serde_json::to_string_pretty(&report)?;
    std::fs::write(&report_path, &json)?;
    log::info!("Report written to {}", report_path.display());

    // Also write trade log CSV for Jupyter
    let csv_path = Path::new(output_dir).join("trades_backtest.csv");
    let mut csv = String::from("time,tier,direction,entry,exit,pips,result,spread,slippage\n");
    for t in &engine.trades {
        csv.push_str(&format!(
            "{},{},{},{},{},{},{},{},{}\n",
            t.time, t.tier, t.direction, t.entry, t.exit, t.pips, t.result, t.spread, t.slippage
        ));
    }
    std::fs::write(&csv_path, csv)?;
    log::info!("Trade log written to {}", csv_path.display());

    Ok(report)
}
