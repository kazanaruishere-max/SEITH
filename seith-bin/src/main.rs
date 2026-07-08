use std::env;

#[tokio::main]
async fn main() {
    dotenvy::dotenv().ok();
    env_logger::init();

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
