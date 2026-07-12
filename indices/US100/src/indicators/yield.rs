use crate::config::thresholds;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum YieldVerdict {
    Bearish,
    Bullish,
    Neutral,
}

pub struct Yield {
    pub us10y_history: Vec<f64>,
    pub us02y_history: Vec<f64>,
}

impl Yield {
    pub fn new() -> Self {
        Self {
            us10y_history: Vec::with_capacity(100),
            us02y_history: Vec::with_capacity(100),
        }
    }

    pub fn compute_zscore(values: &[f64]) -> Option<f64> {
        if values.len() < 2 {
            return None;
        }
        let n = values.len() as f64;
        let mean = values.iter().sum::<f64>() / n;
        let variance = values.iter().map(|v| (v - mean).powi(2)).sum::<f64>() / (n - 1.0);
        if variance < 1e-10 {
            return Some(0.0);
        }
        let std_dev = variance.sqrt();
        let last = *values.last().unwrap();
        Some((last - mean) / std_dev)
    }

    pub fn compute_curve_spread(us10y: f64, us02y: f64) -> f64 {
        us10y - us02y
    }

    pub fn evaluate_yield(us10y_z: Option<f64>, _curve_spread: f64) -> YieldVerdict {
        let z = match us10y_z {
            Some(z) => z,
            None => return YieldVerdict::Neutral,
        };
        if z > thresholds::YIELD_BEARISH {
            YieldVerdict::Bearish
        } else if z < thresholds::YIELD_BULLISH {
            YieldVerdict::Bullish
        } else {
            YieldVerdict::Neutral
        }
    }

    pub fn evaluate_pair(us10y_z: Option<f64>) -> YieldVerdict {
        Self::evaluate_yield(us10y_z, 0.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_zscore_empty() {
        assert!(Yield::compute_zscore(&[]).is_none());
    }

    #[test]
    fn test_zscore_single() {
        assert!(Yield::compute_zscore(&[3.5]).is_none());
    }

    #[test]
    fn test_zscore_constant() {
        let z = Yield::compute_zscore(&[4.0, 4.0, 4.0, 4.0]).unwrap();
        assert!((z - 0.0).abs() < 1e-6);
    }

    #[test]
    fn test_zscore_trend() {
        let z = Yield::compute_zscore(&[3.0, 3.5, 4.0, 4.5]).unwrap();
        assert!(z > 0.0);
    }

    #[test]
    fn test_curve_spread_positive() {
        let spread = Yield::compute_curve_spread(4.5, 3.0);
        assert!((spread - 1.5).abs() < 1e-6);
    }

    #[test]
    fn test_curve_spread_inverted() {
        let spread = Yield::compute_curve_spread(3.0, 4.5);
        assert!((spread - (-1.5)).abs() < 1e-6);
    }

    #[test]
    fn test_evaluate_yield_bearish() {
        assert_eq!(Yield::evaluate_yield(Some(2.0), 0.5), YieldVerdict::Bearish);
    }

    #[test]
    fn test_evaluate_yield_bullish() {
        assert_eq!(Yield::evaluate_yield(Some(-2.0), -0.5), YieldVerdict::Bullish);
    }

    #[test]
    fn test_evaluate_yield_neutral() {
        assert_eq!(Yield::evaluate_yield(Some(0.0), 0.0), YieldVerdict::Neutral);
    }

    #[test]
    fn test_evaluate_yield_none_data() {
        assert_eq!(Yield::evaluate_yield(None, 0.0), YieldVerdict::Neutral);
    }

    #[test]
    fn test_curve_inverted_gives_negative() {
        assert!(Yield::compute_curve_spread(3.2, 4.8) < 0.0);
    }
}
