/// AI SEITH — Tick-Level Backtest Runner
/// cargo run -p xauusd --bin seith-ticktest
use std::path::Path;

fn main() {
    let tick_csv =
        "C:/Users/Lenovo/PROJECT/AI SEITH/jupyter/backtest_analysis/xauusd_ticks_dukascopy.csv";

    if !Path::new(tick_csv).exists() {
        eprintln!("Tick data not found: {}", tick_csv);
        eprintln!("Run: python jupyter/download_ticks_dukascopy.py");
        eprintln!("Or use: python jupyter/generate_synthetic_ticks.py");
        std::process::exit(1);
    }

    let sep = "=".repeat(60);
    println!("{}", sep);
    println!("  AI SEITH Tick-Level Backtest (Real Dukascopy Data)");
    println!("  Data: {}", tick_csv);
    println!("{}", sep);

    // Load tick stream
    let mut stream =
        match xauusd::core::backtest::tick_data::TickStream::from_csv(tick_csv, Some(1_000_000)) {
            Ok(s) => s,
            Err(e) => {
                eprintln!("Failed to load ticks: {}", e);
                std::process::exit(1);
            }
        };
    println!("  Loaded: {} ticks", stream.len());

    // Run tick engine
    let mut engine = xauusd::core::backtest::tick_engine::TickEngine::new();
    let report = engine.run(&mut stream);

    println!();
    println!("{}", sep);
    println!("  TICK BACKTEST RESULTS");
    println!("{}", sep);
    println!("  Trades:      {}", report.total_trades);
    println!("  Win Rate:    {:.1}%", report.win_rate);
    println!("  Net Pips:    {:.1}", report.net_pips);
    println!("  Profit Fac:  {:.2}", report.profit_factor);
    println!(
        "  Max CW/CL:   {}/{}",
        report.max_consecutive_wins, report.max_consecutive_losses
    );
    println!("{}", sep);
}
