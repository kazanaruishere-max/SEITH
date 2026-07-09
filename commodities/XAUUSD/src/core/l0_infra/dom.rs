// L0 - Depth of Market (DOM) snapshot parser
// Parses raw OANDA MT5 market book into structured bids/asks
// Feeds Orderflow Score (OFS) S_DOM calculator

use chrono::{DateTime, Utc};

/// One price level in the order book
#[derive(Debug, Clone, Copy)]
pub struct DomLevel {
    pub price: f64,
    pub volume: u64,
}

/// Full DOM snapshot with validation metadata
#[derive(Debug, Clone)]
pub struct DomSnapshot {
    pub symbol: String,
    pub timestamp: DateTime<Utc>,
    pub bids: Vec<DomLevel>,
    pub asks: Vec<DomLevel>,
    pub best_bid: f64,
    pub best_ask: f64,
    pub spread_pips: f64,
    pub validity: DomValidity,
}

/// Spread validation result
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum DomValidity {
    /// Spread within safe range, can trade
    Valid,
    /// Spread > tolerance, pause entries
    SpreadTooWide { spread_pips: f64, max_allowed: f64 },
    /// Missing bid or ask side — book incomplete
    Incomplete,
}

impl DomSnapshot {
    /// Parse from Python bridge JSON fields.
    /// `bids` must be sorted HIGHEST→LOWEST (best bid at index 0).
    /// `asks` must be sorted LOWEST→HIGHEST (best ask at index 0).
    pub fn new(
        symbol: &str,
        bids: Vec<DomLevel>,
        asks: Vec<DomLevel>,
        spread_tolerance_pips: f64,
    ) -> Self {
        let best_bid = bids.first().map(|l| l.price).unwrap_or(0.0);
        let best_ask = asks.first().map(|l| l.price).unwrap_or(0.0);
        let spread_pips = if best_bid > 0.0 && best_ask > 0.0 {
            (best_ask - best_bid).max(0.0)
        } else {
            0.0
        };

        let validity =
            Self::validate_spread(spread_pips, best_bid, best_ask, spread_tolerance_pips);

        Self {
            symbol: symbol.to_string(),
            timestamp: Utc::now(),
            bids,
            asks,
            best_bid,
            best_ask,
            spread_pips,
            validity,
        }
    }

    fn validate_spread(
        spread_pips: f64,
        best_bid: f64,
        best_ask: f64,
        tolerance: f64,
    ) -> DomValidity {
        if best_bid <= 0.0 || best_ask <= 0.0 {
            return DomValidity::Incomplete;
        }
        if spread_pips <= 0.0 {
            return DomValidity::Incomplete;
        }
        if spread_pips > tolerance {
            return DomValidity::SpreadTooWide {
                spread_pips,
                max_allowed: tolerance,
            };
        }
        DomValidity::Valid
    }

    /// Total bid volume across all levels
    pub fn total_bid_volume(&self) -> u64 {
        self.bids.iter().map(|l| l.volume).sum()
    }

    /// Total ask volume across all levels
    pub fn total_ask_volume(&self) -> u64 {
        self.asks.iter().map(|l| l.volume).sum()
    }

    /// Bid/Ask imbalance ratio: (bid_vol - ask_vol) / (bid_vol + ask_vol)
    /// Positive = bid-dominated (bullish), Negative = ask-dominated (bearish)
    pub fn imbalance(&self) -> f64 {
        let bid_v = self.total_bid_volume() as f64;
        let ask_v = self.total_ask_volume() as f64;
        let total = bid_v + ask_v;
        if total == 0.0 {
            return 0.0;
        }
        (bid_v - ask_v) / total
    }

    /// True if DOM is healthy enough to allow entries
    pub fn can_trade(&self) -> bool {
        matches!(self.validity, DomValidity::Valid)
    }
}

/// Calculate slippage after order execution.
/// Returns absolute slippage in raw price pips (positive = worse for trader).
pub fn calculate_slippage(requested_price: f64, executed_price: f64, is_buy: bool) -> f64 {
    if is_buy {
        (executed_price - requested_price).max(0.0)
    } else {
        (requested_price - executed_price).max(0.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn healthy_dom() -> DomSnapshot {
        DomSnapshot::new(
            "XAUUSD.sml",
            vec![
                DomLevel {
                    price: 4088.445,
                    volume: 10,
                },
                DomLevel {
                    price: 4088.370,
                    volume: 300,
                },
                DomLevel {
                    price: 4088.090,
                    volume: 500,
                },
            ],
            vec![
                DomLevel {
                    price: 4088.875,
                    volume: 10,
                },
                DomLevel {
                    price: 4088.950,
                    volume: 100,
                },
                DomLevel {
                    price: 4089.130,
                    volume: 200,
                },
            ],
            3.5,
        )
    }

    #[test]
    fn test_parse_valid_dom() {
        let dom = healthy_dom();
        assert_eq!(dom.best_bid, 4088.445);
        assert_eq!(dom.best_ask, 4088.875);
        assert!((dom.spread_pips - 0.430).abs() < 0.001);
        assert_eq!(dom.validity, DomValidity::Valid);
    }

    #[test]
    fn test_bids_sorted_highest_first() {
        let dom = healthy_dom();
        let prices: Vec<f64> = dom.bids.iter().map(|l| l.price).collect();
        assert!(prices.windows(2).all(|w| w[0] >= w[1]));
    }

    #[test]
    fn test_asks_sorted_lowest_first() {
        let dom = healthy_dom();
        let prices: Vec<f64> = dom.asks.iter().map(|l| l.price).collect();
        assert!(prices.windows(2).all(|w| w[0] <= w[1]));
    }

    #[test]
    fn test_spread_too_wide() {
        let dom = DomSnapshot::new(
            "XAUUSD.sml",
            vec![DomLevel {
                price: 4088.0,
                volume: 10,
            }],
            vec![DomLevel {
                price: 4095.0,
                volume: 10,
            }],
            3.5,
        );
        assert_eq!(
            dom.validity,
            DomValidity::SpreadTooWide {
                spread_pips: 7.0,
                max_allowed: 3.5
            }
        );
        assert!(!dom.can_trade());
    }

    #[test]
    fn test_incomplete_empty_side() {
        let dom = DomSnapshot::new(
            "XAUUSD.sml",
            vec![],
            vec![DomLevel {
                price: 4090.0,
                volume: 10,
            }],
            3.5,
        );
        assert_eq!(dom.validity, DomValidity::Incomplete);
        assert!(!dom.can_trade());
    }

    #[test]
    fn test_imbalance_bullish() {
        let dom = DomSnapshot::new(
            "XAUUSD.sml",
            vec![DomLevel {
                price: 4088.0,
                volume: 1000,
            }],
            vec![DomLevel {
                price: 4089.0,
                volume: 100,
            }],
            3.5,
        );
        assert!(dom.imbalance() > 0.0);
    }

    #[test]
    fn test_imbalance_bearish() {
        let dom = DomSnapshot::new(
            "XAUUSD.sml",
            vec![DomLevel {
                price: 4088.0,
                volume: 100,
            }],
            vec![DomLevel {
                price: 4089.0,
                volume: 1000,
            }],
            3.5,
        );
        assert!(dom.imbalance() < 0.0);
    }

    #[test]
    fn test_calculate_slippage_buy_worse() {
        let s = calculate_slippage(4088.875, 4089.200, true);
        assert!((s - 0.325).abs() < 0.001);
    }

    #[test]
    fn test_calculate_slippage_sell_worse() {
        let s = calculate_slippage(4088.875, 4088.500, false);
        assert!((s - 0.375).abs() < 0.001);
    }

    #[test]
    fn test_calculate_slippage_no_slippage() {
        let s = calculate_slippage(4088.875, 4088.875, true);
        assert!((s - 0.0).abs() < 0.001);
    }
}
