// XAUUSD trading module
// Full pipeline for Gold trading on OANDA MT5

pub mod analytics;
pub mod config;
pub mod core;
pub mod data;
pub mod external;
pub mod indicators;
pub mod signals;

use core::l3_engine::event_loop::EventLoop;

pub async fn run() {
    dotenvy::dotenv().ok();
    let symbol = std::env::var("MT5_SYMBOL").unwrap_or_else(|_| "XAUUSD.sml".to_string());
    log::info!("Starting {} pipeline...", symbol);
    let mut el = EventLoop::new(&symbol);
    el.run().await;
}
