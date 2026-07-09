// L0 - Data Feed
// Raw price streaming from MT5 + history buffer + DOM

use chrono::{DateTime, Utc};
use std::collections::VecDeque;

#[derive(Debug, Clone)]
pub struct PriceTick {
    pub symbol: String,
    pub bid: f64,
    pub ask: f64,
    pub spread: f64,
    pub timestamp: DateTime<Utc>,
}

impl PriceTick {
    pub fn from_tick_data(symbol: &str, bid: f64, ask: f64) -> Self {
        Self {
            symbol: symbol.to_string(),
            bid,
            ask,
            spread: (ask - bid).max(0.0),
            timestamp: Utc::now(),
        }
    }
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

#[derive(Debug, Clone)]
pub struct DomLevel {
    pub price: f64,
    pub volume: u64,
}

/// In-memory copy of latest DOM snapshot for fast access
#[derive(Debug, Clone)]
pub struct DomSnapshotCache {
    pub bids: Vec<DomLevel>,
    pub asks: Vec<DomLevel>,
    pub best_bid: f64,
    pub best_ask: f64,
    pub spread_pips: f64,
    pub timestamp: DateTime<Utc>,
}

const HISTORY_CAPACITY: usize = 500;

pub struct DataFeed {
    symbol: String,
    history: VecDeque<Ohlcv>,
    last_tick: Option<PriceTick>,
    dom_cache: Option<DomSnapshotCache>,
}

impl DataFeed {
    pub fn new(symbol: &str) -> Self {
        Self {
            symbol: symbol.to_string(),
            history: VecDeque::with_capacity(HISTORY_CAPACITY),
            last_tick: None,
            dom_cache: None,
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
    pub fn dom(&self) -> Option<&DomSnapshotCache> {
        self.dom_cache.as_ref()
    }

    pub fn update_tick(&mut self, tick: PriceTick) {
        self.last_tick = Some(tick);
    }

    /// Update DOM cache from raw levels.
    /// levels: Vec<(price, volume, mt5_type)> where 1=ASK, 2=BID.
    pub fn update_dom(&mut self, levels: &[(f64, u64, i32)]) {
        let mut asks: Vec<DomLevel> = levels
            .iter()
            .filter(|l| l.2 == 1) // type=1 = ASK
            .map(|l| DomLevel {
                price: l.0,
                volume: l.1,
            })
            .collect();
        let mut bids: Vec<DomLevel> = levels
            .iter()
            .filter(|l| l.2 == 2) // type=2 = BID
            .map(|l| DomLevel {
                price: l.0,
                volume: l.1,
            })
            .collect();

        // Sort: asks LOWEST→HIGHEST, bids HIGHEST→LOWEST
        asks.sort_by(|a, b| a.price.partial_cmp(&b.price).unwrap());
        bids.sort_by(|a, b| b.price.partial_cmp(&a.price).unwrap());

        let best_bid = bids.first().map(|l| l.price).unwrap_or(0.0);
        let best_ask = asks.first().map(|l| l.price).unwrap_or(0.0);

        self.dom_cache = Some(DomSnapshotCache {
            bids,
            asks,
            best_bid,
            best_ask,
            spread_pips: (best_ask - best_bid).max(0.0),
            timestamp: Utc::now(),
        });

        // Also update last_tick spread if available
        if let Some(ref mut tick) = self.last_tick {
            tick.spread = (best_ask - best_bid).max(0.0);
        }
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

    pub fn spread(&self) -> f64 {
        self.last_tick.as_ref().map(|t| t.spread).unwrap_or(0.0)
    }

    pub fn last_price(&self) -> f64 {
        self.last_tick.as_ref().map(|t| t.bid).unwrap_or(0.0)
    }
}
