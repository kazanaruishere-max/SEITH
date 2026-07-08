// S_Delta — Delta Score
// Buy aggressor volume - Sell aggressor volume

pub fn calculate_delta(buy_volume: f64, sell_volume: f64) -> f64 {
    buy_volume - sell_volume
}

pub fn delta_score(delta: f64, threshold: f64) -> i32 {
    if delta.abs() < threshold {
        return 0;
    }
    if delta > 0.0 {
        1
    } else {
        -1
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_buying_pressure() {
        assert!(calculate_delta(100.0, 50.0) > 0.0);
    }

    #[test]
    fn test_delta_score_positive() {
        assert_eq!(delta_score(10.0, 5.0), 1);
    }

    #[test]
    fn test_delta_score_zero() {
        assert_eq!(delta_score(2.0, 5.0), 0);
    }
}
