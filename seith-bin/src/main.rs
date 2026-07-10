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

    // Load .env
    let env_path = std::path::Path::new("C:/Users/Lenovo/PROJECT/AI SEITH/.env");
    if env_path.exists() {
        let _ = dotenvy::from_path(env_path);
    }
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
