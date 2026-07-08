// XAUUSD trading module
// Full pipeline for Gold trading on Exness

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
    log::info!("Starting XAUUSD pipeline...");
    let mut el = EventLoop::new("XAUUSDm");
    el.run().await;
}
