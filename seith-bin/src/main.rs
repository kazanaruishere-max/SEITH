use std::env;

#[tokio::main]
async fn main() {
    // Set PYTHONPATH so PyO3 can find seith_bridge module
    let python_path = "C:/Users/Lenovo/PROJECT/AI SEITH/python/python";
    if let Ok(current) = env::var("PYTHONPATH") {
        env::set_var("PYTHONPATH", format!("{};{}", python_path, current));
    } else {
        env::set_var("PYTHONPATH", python_path);
    }

    // Load .env (harus sebelum env_logger)
    let env_path = std::path::Path::new("C:/Users/Lenovo/PROJECT/AI SEITH/.env");
    if env_path.exists() {
        if let Err(e) = dotenvy::from_path(env_path) {
            eprintln!("[SEITH] .env load warning: {}", e);
        }
    } else {
        eprintln!("[SEITH] .env not found, using system env vars");
    }

    // Pastikan RUST_LOG ter-set
    if env::var("RUST_LOG").is_err() {
        env::set_var("RUST_LOG", "debug");
    }
    env_logger::init();

    eprintln!("========================================");
    eprintln!("  AI SEITH v1.0 - XAUUSD Trading Bot");
    eprintln!("========================================");
    eprintln!("  Mode: Live Dry-Run + Auto-Execute");
    eprintln!("  Trades: 24h (skip weekend + rollover)");
    eprintln!("  Strategy: Contrarian reversal, HV>0.5");
    eprintln!("  SL=$3.00 TP=$4.50 | Lot: S-Curve 65-80%");
    eprintln!("  Telegram: @aiseith");
    eprintln!("========================================");

    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        eprintln!("Usage: seith <INSTRUMENT>");
        eprintln!("Example: seith XAUUSD");
        std::process::exit(1);
    }

    match args[1].as_str() {
        "XAUUSD" => xauusd::run().await,
        "EURUSD" => log::info!("EURUSD placeholder"),
        "BTCUSD" => log::info!("BTCUSD placeholder"),
        other => {
            eprintln!("Unknown instrument: {}", other);
            std::process::exit(1);
        }
    }
}
