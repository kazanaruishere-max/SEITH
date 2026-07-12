pub mod analytics;
pub mod config;
pub mod core;
pub mod data;
pub mod external;
pub mod indicators;
pub mod signals;

pub async fn run() {
    log::info!("US100 pipeline starting...");
    log::info!("US100.cash | OANDA v20 | 5-Gate Pipeline");
    log::info!("Session: 14:30-21:00 UTC | Mode: Sniper + Scalp Hybrid");
}
