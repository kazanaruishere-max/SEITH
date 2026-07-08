// Telegram Bot API bridge via PyO3

use anyhow::Result;
use pyo3::prelude::*;

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
    let settings = crate::config::settings::Settings::from_env()?;
    let token = settings.telegram.bot_token.clone();
    let chat_id = settings.telegram.chat_id.clone();
    let text = text.to_string();

    tokio::task::spawn_blocking(move || {
        pyo3::Python::with_gil(|py| {
            let tg = py.import("seith_bridge.telegram")?;
            let token_py = token.into_py(py);
            let chat_id_py = chat_id.into_py(py);
            let text_py = text.into_py(py);

            let init_ok: bool = tg.call_method1("init_telegram", (token_py,))?.extract()?;
            if !init_ok {
                anyhow::bail!("Failed to re-initialize telegram bot");
            }
            let success: bool = tg
                .call_method1("send_message", (chat_id_py, text_py))?
                .extract()?;
            if !success {
                anyhow::bail!("send_message returned false");
            }
            Ok::<(), anyhow::Error>(())
        })
    })
    .await??;

    Ok(())
}

/// Send photo with caption to Telegram
pub async fn send_photo(photo_path: &str, caption: &str) -> Result<()> {
    log::debug!("Sending Telegram photo: {}", photo_path);
    let settings = crate::config::settings::Settings::from_env()?;
    let token = settings.telegram.bot_token.clone();
    let chat_id = settings.telegram.chat_id.clone();
    let photo_path = photo_path.to_string();
    let caption = caption.to_string();

    tokio::task::spawn_blocking(move || {
        pyo3::Python::with_gil(|py| {
            let tg = py.import("seith_bridge.telegram")?;
            let token_py = token.into_py(py);
            let chat_id_py = chat_id.into_py(py);
            let photo_path_py = photo_path.into_py(py);
            let caption_py = caption.into_py(py);

            // Re-init
            let init_ok: bool = tg.call_method1("init_telegram", (token_py,))?.extract()?;
            if !init_ok {
                anyhow::bail!("Failed to re-initialize telegram bot");
            }
            let success: bool = tg
                .call_method1("send_photo", (chat_id_py, photo_path_py, caption_py))?
                .extract()?;
            if !success {
                anyhow::bail!("send_photo returned false");
            }
            Ok::<(), anyhow::Error>(())
        })
    })
    .await??;

    Ok(())
}
