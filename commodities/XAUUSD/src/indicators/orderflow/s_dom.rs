// S_DOM — Depth of Market Heatmap Z-Score
// Updated with real DOM cache support for live pipeline

use crate::core::l0_infra::DomSnapshot;

#[derive(Debug, Clone)]
pub struct DomResult {
    pub imbalance: f64,
    pub z_score: f64,
    pub heatmap_score: i32,
}

/// Calculate S_DOM from live DomSnapshot (real OANDA DOM data).
/// Uses limit order book depth asymmetry between bids and asks.
pub fn calculate_s_dom_from_snapshot(snapshot: &DomSnapshot) -> DomResult {
    let bid_vol: f64 = snapshot.bids.iter().map(|l| l.volume as f64).sum();
    let ask_vol: f64 = snapshot.asks.iter().map(|l| l.volume as f64).sum();
    let total = bid_vol + ask_vol;

    if total == 0.0 {
        return DomResult {
            imbalance: 0.0,
            z_score: 0.0,
            heatmap_score: 0,
        };
    }

    let imbalance = (bid_vol - ask_vol) / total;
    let z_score = imbalance * 5.0;
    let heatmap_score = if z_score > 1.0 {
        1
    } else if z_score < -1.0 {
        -1
    } else {
        0
    };
    DomResult {
        imbalance,
        z_score,
        heatmap_score,
    }
}

/// Calculate from raw volume slices (backward compat, used in backtest).
pub fn calculate_dom(bid_volume: &[f64], ask_volume: &[f64]) -> DomResult {
    let bid_sum: f64 = bid_volume.iter().sum();
    let ask_sum: f64 = ask_volume.iter().sum();
    let total = bid_sum + ask_sum;
    if total == 0.0 {
        return DomResult {
            imbalance: 0.0,
            z_score: 0.0,
            heatmap_score: 0,
        };
    }
    let imbalance = (bid_sum - ask_sum) / total;
    let z_score = imbalance * 5.0;
    let heatmap_score = if z_score > 1.0 {
        1
    } else if z_score < -1.0 {
        -1
    } else {
        0
    };
    DomResult {
        imbalance,
        z_score,
        heatmap_score,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::l0_infra::{DomLevel, DomSnapshot};

    fn sample_snapshot(bid_vol: u64, ask_vol: u64) -> DomSnapshot {
        DomSnapshot::new(
            "XAUUSD.sml",
            vec![DomLevel {
                price: 4100.0,
                volume: bid_vol,
            }],
            vec![DomLevel {
                price: 4101.0,
                volume: ask_vol,
            }],
            3.5,
        )
    }

    #[test]
    fn test_bid_dominated() {
        let snap = sample_snapshot(200, 50);
        let r = calculate_s_dom_from_snapshot(&snap);
        assert!(r.imbalance > 0.0);
        assert_eq!(r.heatmap_score, 1);
    }

    #[test]
    fn test_ask_dominated() {
        let snap = sample_snapshot(10, 200);
        let r = calculate_s_dom_from_snapshot(&snap);
        assert!(r.imbalance < 0.0);
        assert_eq!(r.heatmap_score, -1);
    }

    #[test]
    fn test_zero_volume() {
        let snap = sample_snapshot(0, 0);
        let r = calculate_s_dom_from_snapshot(&snap);
        assert_eq!(r.heatmap_score, 0);
    }

    #[test]
    fn test_legacy_backward_compat() {
        let r = calculate_dom(&[100.0, 200.0], &[50.0, 30.0]);
        assert!(r.imbalance > 0.0);
    }
}
