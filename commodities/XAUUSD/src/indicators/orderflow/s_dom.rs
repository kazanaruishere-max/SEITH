// S_DOM — Depth of Market Heatmap Z-Score

#[derive(Debug, Clone)]
pub struct DomResult {
    pub imbalance: f64,
    pub z_score: f64,
    pub heatmap_score: i32,
}

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
    let z_score = imbalance * 5.0; // normalized to z-score scale
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

    #[test]
    fn test_bid_dominated() {
        let r = calculate_dom(&[100.0, 200.0], &[50.0, 30.0]);
        assert!(r.imbalance > 0.0);
    }

    #[test]
    fn test_ask_dominated() {
        let r = calculate_dom(&[10.0, 20.0], &[100.0, 200.0]);
        assert!(r.imbalance < 0.0);
    }

    #[test]
    fn test_zero_volume() {
        let r = calculate_dom(&[], &[]);
        assert_eq!(r.heatmap_score, 0);
    }
}
