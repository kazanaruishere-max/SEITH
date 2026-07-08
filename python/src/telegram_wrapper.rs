// Telegram Bot API wrapper — PyO3 bridge
// Wraps python/seith_bridge/telegram.py functions

#![allow(non_local_definitions)]

use pyo3::prelude::*;

#[pyclass]
pub struct TelegramWrapper;

#[pymethods]
impl TelegramWrapper {
    #[new]
    fn new() -> Self {
        Self
    }

    fn init(&self, token: &str) -> PyResult<bool> {
        Python::with_gil(|py| {
            let tg = PyModule::import(py, "seith_bridge.telegram")?;
            tg.call_method1("init_telegram", (token,))?
                .extract::<bool>()
        })
    }

    fn send_message(&self, chat_id: &str, text: &str) -> PyResult<bool> {
        Python::with_gil(|py| {
            let tg = PyModule::import(py, "seith_bridge.telegram")?;
            tg.call_method1("send_message", (chat_id, text))?
                .extract::<bool>()
        })
    }

    fn send_photo(&self, chat_id: &str, photo_path: &str, caption: &str) -> PyResult<bool> {
        Python::with_gil(|py| {
            let tg = PyModule::import(py, "seith_bridge.telegram")?;
            tg.call_method1("send_photo", (chat_id, photo_path, caption))?
                .extract::<bool>()
        })
    }
}
