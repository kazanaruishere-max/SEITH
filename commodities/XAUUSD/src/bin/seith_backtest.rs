use std::env;
/// AI SEITH — Backtest Runner with threshold sweep support.
/// cargo run -p xauusd --bin seith-backtest [--ofs=2] [--spread=3.5] [--tier1=75] [--tier2=60]
use std::path::Path;

#[tokio::main]
async fn main() {
    dotenvy::dotenv().ok();
    env_logger::init();

    let args: Vec<String> = env::args().collect();
    let mut _ofs_min = 2i32;
    let mut _spread_tol = 3.5f64;
    let mut _tier1_thr = 0.75f64;
    let mut _tier2_thr = 0.60f64;
    let mut _gvz_thr = 1.0f64;
    let mut _frama_dev = 0.5f64;

    for arg in &args[1..] {
        if let Some(v) = arg.strip_prefix("--ofs=") {
            env::set_var("BT_OFS_MIN", v);
            _ofs_min = v.parse().unwrap_or(2);
        }
        if let Some(v) = arg.strip_prefix("--spread=") {
            env::set_var("BT_SPREAD_TOL", v);
            _spread_tol = v.parse().unwrap_or(3.5);
        }
        if let Some(v) = arg.strip_prefix("--tier1=") {
            let val: f64 = v.parse().unwrap_or(75.0);
            let dec = val / 100.0;
            env::set_var("BT_TIER1_THR", dec.to_string());
            _tier1_thr = dec;
        }
        if let Some(v) = arg.strip_prefix("--tier2=") {
            let val: f64 = v.parse().unwrap_or(60.0);
            let dec = val / 100.0;
            env::set_var("BT_TIER2_THR", dec.to_string());
            _tier2_thr = dec;
        }
        if let Some(v) = arg.strip_prefix("--gvz=") {
            env::set_var("BT_GVZ_THR", v);
            _gvz_thr = v.parse().unwrap_or(1.0);
        }
        if let Some(v) = arg.strip_prefix("--frama=") {
            env::set_var("BT_FRAMA_DEV", v);
            _frama_dev = v.parse().unwrap_or(0.5);
        }
    }

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
    println!(
        "  Params: ofs={} spread={} t1={:.0}% t2={:.0}% gvz={} frama={}",
        _ofs_min,
        _spread_tol,
        _tier1_thr * 100.0,
        _tier2_thr * 100.0,
        _gvz_thr,
        _frama_dev
    );
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
