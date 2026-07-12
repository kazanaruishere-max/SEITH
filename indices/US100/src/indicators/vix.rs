use crate::config::thresholds;

pub enum HvVerdict {
    SkipLow,
    Pass,
    PassConfidenceDown,
    SkipHigh,
}

pub struct Vix {
    pub hv_z_history: Vec<f64>,
    pub threshold_bias: f64,
}

impl Vix {
    pub fn new() -> Self {
        Self {
            hv_z_history: Vec::with_capacity(100),
            threshold_bias: thresholds::VIX_DEFAULT_BASELINE,
        }
    }

    pub fn compute_hv_zscore(&self, prices: &[f64], window: usize) -> f64 {
        if prices.len() < window + 1 {
            return 0.0;
        }
        let returns: Vec<f64> = prices.windows(2)
            .map(|w| (w[1] - w[0]) / w[0])
            .collect();
        let recent = &returns[returns.len().saturating_sub(window)..];
        let n = recent.len() as f64;
        if n < 2.0 {
            return 0.0;
        }
        let mean = recent.iter().sum::<f64>() / n;
        let variance = recent.iter().map(|r| (r - mean).powi(2)).sum::<f64>() / (n - 1.0);
        let std_dev = variance.sqrt();
        if std_dev < 1e-10 {
            return 0.0;
        }
        let last_return = *recent.last().unwrap_or(&0.0);
        (last_return - mean) / std_dev
    }

    pub fn evaluate_hv(&self, hv_z: f64) -> HvVerdict {
        if hv_z < thresholds::HV_SKIP_LOW { HvVerdict::SkipLow }
        else if hv_z <= thresholds::HV_SWEET_SPOT_MAX { HvVerdict::Pass }
        else if hv_z <= thresholds::HV_ELEVATED_MAX { HvVerdict::PassConfidenceDown }
        else { HvVerdict::SkipHigh }
    }

    pub fn calibrate_threshold(&mut self, vix_value: f64) {
        self.threshold_bias = vix_value;
    }

    pub fn adjusted_hv_zscore(&self, raw_hv_z: f64) -> f64 {
        let adjustment = (self.threshold_bias - thresholds::VIX_DEFAULT_BASELINE) / 10.0;
        raw_hv_z + adjustment
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hv_zscore_empty() {
        let vix = Vix::new();
        assert!((vix.compute_hv_zscore(&[], 10) - 0.0).abs() < 1e-6);
    }

    #[test]
    fn test_hv_zscore_constant() {
        let vix = Vix::new();
        let prices = vec![100.0; 20];
        assert!((vix.compute_hv_zscore(&prices, 10) - 0.0).abs() < 1e-6);
    }

    #[test]
    fn test_hv_zscore_trending() {
        let vix = Vix::new();
        let prices: Vec<f64> = (0..20).map(|i| 100.0 + i as f64).collect();
        let z = vix.compute_hv_zscore(&prices, 10);
        assert!(z.is_finite());
    }

    #[test]
    fn test_evaluate_hv_skip_low() {
        let vix = Vix::new();
        assert!(matches!(vix.evaluate_hv(-1.5), HvVerdict::SkipLow));
    }

    #[test]
    fn test_evaluate_hv_pass() {
        let vix = Vix::new();
        assert!(matches!(vix.evaluate_hv(0.0), HvVerdict::Pass));
    }

    #[test]
    fn test_evaluate_hv_pass_confidence_down() {
        let vix = Vix::new();
        assert!(matches!(vix.evaluate_hv(1.8), HvVerdict::PassConfidenceDown));
    }

    #[test]
    fn test_evaluate_hv_skip_high() {
        let vix = Vix::new();
        assert!(matches!(vix.evaluate_hv(2.5), HvVerdict::SkipHigh));
    }

    #[test]
    fn test_evaluate_hv_boundary_15() {
        let vix = Vix::new();
        assert!(matches!(vix.evaluate_hv(1.5), HvVerdict::Pass));
    }

    #[test]
    fn test_evaluate_hv_boundary_20() {
        let vix = Vix::new();
        assert!(matches!(vix.evaluate_hv(2.0), HvVerdict::PassConfidenceDown));
    }

    #[test]
    fn test_calibrate_threshold() {
        let mut vix = Vix::new();
        vix.calibrate_threshold(25.0);
        assert!((vix.threshold_bias - 25.0).abs() < 1e-6);
    }

    #[test]
    fn test_adjusted_hv_zscore_higher_vix() {
        let mut vix = Vix::new();
        vix.calibrate_threshold(28.0);
        let adjusted = vix.adjusted_hv_zscore(1.0);
        assert!(adjusted > 1.0);
    }

    #[test]
    fn test_adjusted_hv_zscore_default_vix() {
        let vix = Vix::new();
        let adjusted = vix.adjusted_hv_zscore(1.0);
        assert!((adjusted - 1.0).abs() < 1e-6);
    }
}
