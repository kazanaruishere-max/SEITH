// Telegram Bot API bridge via PyO3

use anyhow::Result;

/// Initialize Telegram bot
pub async fn init_telegram() -> Result<()> {
    log::info!("Telegram bridge initializing");
    let settings = crate::config::settings::Settings::from_env()?;

    // Call PyO3 init
    pyo3::Python::with_gil(|py| {
        let tg_module = py
            .import("seith_bridge.telegram")
            .map_err(|e| anyhow::anyhow!("PyO3 import telegram: {}", e))?;

        let token = settings.telegram.bot_token;
        let success: bool = tg_module
            .call_method1("init_telegram", (token,))?
            .extract()?;

        if !success {
            anyhow::bail!("Failed to initialize telegram bot (check token)");
        }
        Ok::<_, anyhow::Error>(())
    })?;

    log::info!("Telegram bridge initialized successfully");
    Ok(())
}

/// Send message to Telegram
pub async fn send_message(text: &str) -> Result<()> {
    log::debug!("Sending Telegram message: {}", text);
    let settings = match crate::config::settings::Settings::from_env() {
        Ok(s) => s,
        Err(e) => {
            log::warn!("Telegram settings unavailable: {}", e);
            return Ok(());
        }
    };
    let token = settings.telegram.bot_token.clone();
    let chat_id = settings.telegram.chat_id.clone();
    let text = text.to_string();

    // Direct Python::with_gil instead of spawn_blocking to avoid
    // deadlock with tokio runtime + PyO3 auto-initialize
    let success: bool = pyo3::Python::with_gil(|py| {
        let tg = py.import("seith_bridge.telegram")?;
        let init_ok: bool = tg.call_method1("init_telegram", (token,))?.extract()?;
        if !init_ok {
            anyhow::bail!("Failed to re-initialize telegram bot");
        }
        let ok: bool = tg
            .call_method1("send_message", (chat_id, text))?
            .extract()?;
        Ok::<_, anyhow::Error>(ok)
    })?;

    if success {
        log::info!("Telegram message sent");
    } else {
        log::warn!("Telegram send_message returned false");
    }

    Ok(())
}

/// Send formatted trading signal with chart to Telegram
#[allow(clippy::too_many_arguments)]
pub async fn send_signal(
    direction: &str,
    entry_price: f64,
    sl_price: f64,
    tp1_price: f64,
    tp2_price: Option<f64>,
    lot_size: f64,
    confidence: f64,
    reasoning: &str,
    order_type: &str,
    prices: Vec<(i64, f64, f64)>, // (timestamp, bid, ask) for chart
) -> Result<()> {
    let settings = match crate::config::settings::Settings::from_env() {
        Ok(s) => s,
        Err(_) => return Ok(()), // silent fail if no telegram
    };
    let _chat_id = settings.telegram.chat_id.clone();
    let direction_emoji = if direction == "BUY" {
        "\u{1f7e2}"
    } else {
        "\u{1f534}"
    };
    let dir_str = if direction == "BUY" {
        "LONG (BUY)"
    } else {
        "SHORT (SELL)"
    };

    let rr1 = (tp1_price - entry_price).abs() / (entry_price - sl_price).abs().max(0.001);
    let rr2 =
        tp2_price.map(|tp2| (tp2 - entry_price).abs() / (entry_price - sl_price).abs().max(0.001));

    // Build signal text
    let text = format!(
        "{emoji} AUTO-SIGNAL — XAUUSD.sml M15\n\
        \u{2501}\u{2501}\u{2501}\u{2501}\u{2501}\u{2501}\u{2501}\u{2501}\u{2501}\u{2501}\u{2501}\u{2501}\u{2501}\u{2501}\u{2501}\u{2501}\u{2501}\u{2501}\u{2501}\u{2501}\u{2501}\u{2501}\n\
        {dir_emoji} Direction : {dir}\n\
        \u{1f3af} Entry Zone: {entry:.3} \u{00b1} {zone:.3}\n\
        \u{1f6d1} Stop Loss : {sl:.3}\n\
        \u{1f3c6} TP1 : {tp1:.3} (1:{rr1:.1} RR) — {hit1:.0}%\n\
        {tp2_line}\
        \u{1f4e6} Lot Size : {lot}\n\
        \u{1f9e0} Confidence: {conf:.0}%\n\
        \u{2501}\u{2501}\u{2501}\u{2501}\u{2501}\u{2501}\u{2501}\u{2501}\u{2501}\u{2501}\u{2501}\u{2501}\u{2501}\u{2501}\u{2501}\u{2501}\u{2501}\u{2501}\u{2501}\u{2501}\u{2501}\u{2501}\n\
        \u{1f9e0} Reasoning:\n{reasoning}\n\
        \u{2501}\u{2501}\u{2501}\u{2501}\u{2501}\u{2501}\u{2501}\u{2501}\u{2501}\u{2501}\u{2501}\u{2501}\u{2501}\u{2501}\u{2501}\u{2501}\u{2501}\u{2501}\u{2501}\u{2501}\u{2501}\u{2501}\n\
        \u{26a1} Order: {order_type} | Valid: 10 menit",
        emoji = direction_emoji,
        dir_emoji = direction_emoji,
        dir = dir_str,
        entry = entry_price,
        zone = (entry_price * 0.0003).max(0.1),
        sl = sl_price,
        tp1 = tp1_price,
        rr1 = rr1,
        hit1 = (confidence * 0.65).min(100.0),
        tp2_line = if let (Some(tp2), Some(rr2)) = (tp2_price, rr2) {
            format!("\u{26a1} TP2 : {:.3} (1:{:.1} RR) — {:.0}% \u{1f680}\n", tp2, rr2, (confidence * 0.25).min(100.0))
        } else { String::new() },
        lot = lot_size,
        conf = confidence,
        reasoning = reasoning,
        order_type = order_type,
    );

    // Generate chart
    let chart_path = std::env::temp_dir().join(format!(
        "seith_signal_{}.png",
        chrono::Utc::now().timestamp()
    ));
    let chart_str = chart_path.to_str().unwrap_or("seith_signal.png");

    let _ = generate_chart_image(
        &prices,
        entry_price,
        sl_price,
        tp1_price,
        tp2_price,
        direction,
        chart_str,
    )
    .await;

    // Send photo with signal as caption
    let _ = send_photo(chart_str, &text).await;

    // Also send plain text as fallback
    let _ = send_message(&text).await;

    Ok(())
}

/// Generate chart image via Python chart.py
async fn generate_chart_image(
    prices: &[(i64, f64, f64)],
    entry: f64,
    sl: f64,
    tp1: f64,
    tp2: Option<f64>,
    direction: &str,
    path: &str,
) -> Result<()> {
    let prices = prices.to_vec();
    let direction = direction.to_string();
    let path = path.to_string();

    tokio::task::spawn_blocking(move || {
        pyo3::Python::with_gil(|py| {
            let chart_mod = py
                .import("seith_bridge.chart")
                .map_err(|e| anyhow::anyhow!("PyO3 import chart: {}", e))?;

            let prices_py: Vec<Vec<f64>> = prices
                .iter()
                .map(|(t, b, a)| vec![*t as f64, *b, *a])
                .collect();

            let tp2_py = tp2.unwrap_or(0.0);

            let _result: String = chart_mod
                .call_method1(
                    "generate_chart",
                    (prices_py, entry, sl, tp1, tp2_py, &direction, &path),
                )?
                .extract()?;

            Ok::<_, anyhow::Error>(())
        })
    })
    .await??;

    Ok(())
}

/// Send photo with caption to Telegram
pub async fn send_photo(photo_path: &str, caption: &str) -> Result<()> {
    log::debug!("Sending Telegram photo: {}", photo_path);
    let settings = match crate::config::settings::Settings::from_env() {
        Ok(s) => s,
        Err(_) => return Ok(()),
    };
    let chat_id = settings.telegram.chat_id.clone();
    let token = settings.telegram.bot_token.clone();
    let photo_path = photo_path.to_string();
    let caption = caption.to_string();

    let success: bool = pyo3::Python::with_gil(|py| {
        let tg = py.import("seith_bridge.telegram")?;
        let init_ok: bool = tg.call_method1("init_telegram", (token,))?.extract()?;
        if !init_ok {
            anyhow::bail!("Telegram init failed");
        }
        let ok: bool = tg
            .call_method1("send_photo", (chat_id, photo_path, caption))?
            .extract()?;
        Ok::<_, anyhow::Error>(ok)
    })?;

    if success {
        log::info!("Telegram photo sent");
    }
    Ok(())
}
