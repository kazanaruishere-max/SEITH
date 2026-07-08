// L0 - Data Feed
// Raw price streaming from MT5 + history buffer

use chrono::{DateTime, Utc};
use std::collections::VecDeque;

#[derive(Debug, Clone)]
pub struct PriceTick {
    pub symbol: String,
    pub bid: f64,
    pub ask: f64,
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Clone)]
pub struct Ohlcv {
    pub time: DateTime<Utc>,
    pub open: f64,
    pub high: f64,
    pub low: f64,
    pub close: f64,
    pub volume: f64,
}

const HISTORY_CAPACITY: usize = 500;

pub struct DataFeed {
    symbol: String,
    history: VecDeque<Ohlcv>,
    last_tick: Option<PriceTick>,
}

impl DataFeed {
    pub fn new(symbol: &str) -> Self {
        Self {
            symbol: symbol.to_string(),
            history: VecDeque::with_capacity(HISTORY_CAPACITY),
            last_tick: None,
        }
    }

    pub fn symbol(&self) -> &str {
        &self.symbol
    }

    pub fn last_tick(&self) -> Option<&PriceTick> {
        self.last_tick.as_ref()
    }

    pub fn history(&self) -> &VecDeque<Ohlcv> {
        &self.history
    }

    pub fn update_tick(&mut self, tick: PriceTick) {
        self.last_tick = Some(tick);
    }

    pub fn push_ohlcv(&mut self, candle: Ohlcv) {
        if self.history.len() >= HISTORY_CAPACITY {
            self.history.pop_front();
        }
        self.history.push_back(candle);
    }

    pub fn latest_candle(&self) -> Option<&Ohlcv> {
        self.history.back()
    }
}
