// Telegram Bot API wrapper — PyO3 bridge
// Stub only — no implementation yet

#![allow(non_local_definitions)]

use pyo3::prelude::*;

#[pyclass]
pub struct TelegramWrapper {}

#[pymethods]
impl TelegramWrapper {
    #[new]
    pub fn new() -> Self {
        Self {}
    }

    fn send_message(&self, _chat_id: &str, _text: &str) -> PyResult<bool> {
        todo!("Implement Telegram send via python-telegram-bot")
    }

    fn send_photo(&self, _chat_id: &str, _photo_path: &str, _caption: &str) -> PyResult<bool> {
        todo!("Implement Telegram photo send")
    }
}
