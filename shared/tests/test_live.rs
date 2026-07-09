// Integration test for MT5 live connection and Telegram dispatch

use anyhow::Result;
use shared::external::{mt5_bridge::Mt5Api, telegram_bridge};

fn setup() {
    // Manually load env vars since dotenvy has path issues in test context
    let content = std::fs::read_to_string("C:/Users/Lenovo/PROJECT/AI SEITH/.env")
        .expect("Failed to read .env file");
    for line in content.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        if let Some(eq_pos) = line.find('=') {
            let key = line[..eq_pos].trim();
            let value = line[eq_pos + 1..].trim().trim_matches('"');
            std::env::set_var(key, value);
        }
    }
    // Debug: verify
    assert!(
        std::env::var("TELEGRAM_BOT_TOKEN").is_ok(),
        "TELEGRAM_BOT_TOKEN not set"
    );

    // Clean chat ID if it's a URL
    if let Ok(mut chat_id) = std::env::var("TELEGRAM_CHAT_ID") {
        if chat_id.starts_with("https://t.me/") {
            chat_id = chat_id.replace("https://t.me/", "@");
        }
        std::env::set_var("TELEGRAM_CHAT_ID", chat_id);
    }

    // Setup PYTHONPATH for PyO3
    pyo3::Python::with_gil(|py| {
        let sys = pyo3::types::PyModule::import(py, "sys").unwrap();
        let path: &pyo3::types::PyList = sys.getattr("path").unwrap().downcast().unwrap();
        path.insert(0, "C:/Users/Lenovo/PROJECT/AI SEITH/python/python")
            .unwrap();
        path.insert(
            0,
            "C:/Users/Lenovo/AppData/Local/Programs/Python/Python311/Lib/site-packages",
        )
        .unwrap();
    });
}

#[tokio::test]
#[ignore]
async fn test_mt5_live_connection() -> Result<()> {
    setup();
    let symbol = std::env::var("MT5_SYMBOL").unwrap_or_else(|_| "XAUUSD.sml".to_string());
    let api = Mt5Api::new(&symbol);
    let connect_res = api.connect().await;
    assert!(
        connect_res.is_ok(),
        "MT5 live connection failed: {:?}",
        connect_res.err()
    );
    let price = api.get_price().await?;
    println!("Live {} Price: {}", symbol, price);
    assert!(price > 0.0);
    Ok(())
}

#[tokio::test]
#[ignore]
async fn test_telegram_dispatch() -> Result<()> {
    setup();
    telegram_bridge::init_telegram().await?;
    let send_res = telegram_bridge::send_message(
        "<b>[AI SEITH]</b> Live Integration Test: Telegram dispatch is operational. 🚀",
    )
    .await;
    assert!(
        send_res.is_ok(),
        "Telegram message send failed: {:?}",
        send_res.err()
    );
    Ok(())
}
