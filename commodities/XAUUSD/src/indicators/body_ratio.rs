// Body Ratio — Candlestick Rejection Detection
// Real Body / Total Range < 0.25 → rejection valid

pub fn calculate_body_ratio(open: f64, close: f64, high: f64, low: f64) -> f64 {
    let body = (close - open).abs();
    let range = (high - low).abs();
    if range == 0.0 {
        return 0.0;
    }
    body / range
}

pub fn is_rejection_valid(body_ratio: f64) -> bool {
    body_ratio < 0.25
}

pub fn determine_rejection_direction(open: f64, close: f64) -> &'static str {
    if close > open {
        "BULLISH_REJECTION"
    } else {
        "BEARISH_REJECTION"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_small_body_rejection() {
        let r = calculate_body_ratio(100.0, 100.5, 106.0, 99.0);
        assert!(is_rejection_valid(r));
    }

    #[test]
    fn test_large_body_no_rejection() {
        let r = calculate_body_ratio(100.0, 104.0, 105.0, 99.0);
        assert!(!is_rejection_valid(r));
    }

    #[test]
    fn test_zero_range() {
        let r = calculate_body_ratio(100.0, 100.0, 100.0, 100.0);
        assert!((r - 0.0).abs() < 0.01);
    }

    #[test]
    fn test_bullish_rejection() {
        assert_eq!(
            determine_rejection_direction(100.0, 101.0),
            "BULLISH_REJECTION"
        );
    }
}
