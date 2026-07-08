// Python bridge entry point (PyO3)
// Rust↔Python FFI bridge untuk MT5, Telegram, scraping

use pyo3::prelude::*;

mod mt5_wrapper;
mod scraper_wrapper;
mod telegram_wrapper;

/// Initialize Python bridge
#[pyfunction]
fn init_bridge() -> PyResult<String> {
    log::info!("AI SEITH Python bridge initialized");
    Ok("seith_python bridge ready".to_string())
}

/// Python module definition
#[pymodule]
fn seith_python(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(init_bridge, m)?)?;
    m.add_class::<mt5_wrapper::Mt5Wrapper>()?;
    m.add_class::<telegram_wrapper::TelegramWrapper>()?;
    m.add_class::<scraper_wrapper::ScraperWrapper>()?;
    Ok(())
}
