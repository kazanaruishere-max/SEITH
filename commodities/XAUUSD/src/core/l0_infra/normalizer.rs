// L0 - Normalizer
// Convert gold price to 3 digits (multiplier 0.010)
// XAUUSDm memiliki 5 digit desimal, sistem kerja 3 digit

const XAUUSD_MULTIPLIER: f64 = 0.010;

pub fn normalize_price(raw_price: f64) -> f64 {
    (raw_price * XAUUSD_MULTIPLIER * 100.0).round() / 100.0
}

pub fn denormalize_price(normalized_price: f64) -> f64 {
    normalized_price / XAUUSD_MULTIPLIER
}

pub fn normalize_spread(spread_pips: f64) -> f64 {
    spread_pips * XAUUSD_MULTIPLIER
}

pub fn normalize_pips(pips: f64) -> f64 {
    pips * XAUUSD_MULTIPLIER
}

pub fn pips_to_points(pips: f64) -> f64 {
    pips * 10.0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_normalize_xauusd() {
        let raw = 2350.12345;
        let norm = normalize_price(raw);
        assert!((norm - 23.50).abs() < 0.01);
    }

    #[test]
    fn test_roundtrip() {
        let raw = 2350.12345;
        let norm = normalize_price(raw);
        let back = denormalize_price(norm);
        assert!((back - 2350.0).abs() < 0.1);
    }

    #[test]
    fn test_normalize_spread() {
        let spread = 3.5;
        let norm = normalize_spread(spread);
        assert!((norm - 0.035).abs() < 0.001);
    }
}
