/// AI SEITH — Backtest Runner
/// cargo run -p xauusd --bin seith-backtest
use std::path::Path;

#[tokio::main]
async fn main() {
    dotenvy::dotenv().ok();
    env_logger::init();

    eprintln!("Starting backtest...");
    let csv_path = "C:/Users/Lenovo/PROJECT/AI SEITH/jupyter/backtest_analysis/xauusd_m1_3m.csv";
    let out_dir = "C:/Users/Lenovo/PROJECT/AI SEITH/jupyter/backtest_analysis";

    if !Path::new(csv_path).exists() {
        eprintln!("Data file not found: {}", csv_path);
        eprintln!("Run: python jupyter/download_m1_data.py");
        std::process::exit(1);
    }

    let sep = "=".repeat(60);
    println!("{}", sep);
    println!("  AI SEITH Backtest Engine");
    println!("  Data: {}", csv_path);
    println!("{}", sep);

    match xauusd::core::backtest::run_backtest(csv_path, out_dir).await {
        Ok(report) => {
            println!();
            println!("{}", sep);
            println!("  BACKTEST RESULTS");
            println!("{}", sep);
            println!("  Trades:      {}", report.total_trades);
            println!("  Win Rate:    {:.1}%", report.win_rate);
            println!("  Net Pips:    {:.1}", report.net_pips);
            println!("  Profit Fac:  {:.2}", report.profit_factor);
            println!("  Max DD:      {:.2}%", report.max_drawdown_pct);
            println!(
                "  Max CW/CL:   {}/{}",
                report.max_consecutive_wins, report.max_consecutive_losses
            );
            println!("  Avg Spread:  {:.2} pips", report.avg_spread);
            println!("  P&L:         ${:.2}", report.pnl);
            println!("{}", sep);
        }
        Err(e) => {
            eprintln!("Backtest failed: {}", e);
            std::process::exit(1);
        }
    }
}
