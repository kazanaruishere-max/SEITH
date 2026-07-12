use crate::config::settings::DECIMAL_PLACES;

pub fn normalize_price(price: f64) -> f64 {
    let factor = 10f64.powi(DECIMAL_PLACES as i32);
    (price * factor).round() / factor
}

pub fn normalize_prices(prices: &[f64]) -> Vec<f64> {
    prices.iter().copied().map(normalize_price).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_normalize_price_rounds_to_2_decimals() {
        let result = normalize_price(123.456);
        assert!((result - 123.46).abs() < 1e-10);
    }

    #[test]
    fn test_normalize_price_already_2_decimals() {
        let result = normalize_price(100.25);
        assert!((result - 100.25).abs() < 1e-10);
    }

    #[test]
    fn test_normalize_price_integer() {
        let result = normalize_price(100.0);
        assert!((result - 100.0).abs() < 1e-10);
    }

    #[test]
    fn test_normalize_prices() {
        let input = vec![1.234, 5.678, 9.999];
        let result = normalize_prices(&input);
        assert!((result[0] - 1.23).abs() < 1e-10);
        assert!((result[1] - 5.68).abs() < 1e-10);
        assert!((result[2] - 10.0).abs() < 1e-10);
    }
}
