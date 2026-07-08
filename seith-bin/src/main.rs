use std::env;

#[tokio::main]
async fn main() {
    env_logger::init();

    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        eprintln!("Usage: seith <INSTRUMENT>");
        eprintln!("Example: seith XAUUSD");
        std::process::exit(1);
    }

    let instrument = &args[1];

    match instrument.as_str() {
        "XAUUSD" => {
            log::info!("Starting AI SEITH for XAUUSD...");
            println!("✓ AI SEITH XAUUSD initialized (framework only)");
        }
        "EURUSD" => {
            log::info!("Starting AI SEITH for EURUSD...");
            println!("✓ AI SEITH EURUSD initialized (placeholder)");
        }
        "BTCUSD" => {
            log::info!("Starting AI SEITH for BTCUSD...");
            println!("✓ AI SEITH BTCUSD initialized (placeholder)");
        }
        _ => {
            eprintln!("Error: Unknown instrument '{}'", instrument);
            eprintln!("Supported: XAUUSD, EURUSD, BTCUSD");
            std::process::exit(1);
        }
    }

    log::info!("AI SEITH graceful shutdown");
}
