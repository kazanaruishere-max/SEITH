// L2 - Net_Dev Calculator
// Net Deviation score: (Actual - Forecast) + (Revised_Previous - Original_Previous)
// Threshold: |Net_Dev| >= 2.0 = valid signal

use crate::core::l2_news::data_extractor::ExtractedData;
use crate::core::l2_news::revision_handler::RevisionInfo;

#[derive(Debug, Clone)]
pub struct NetDevResult {
    pub net_dev: f64,
    pub is_valid: bool,
    pub is_extreme: bool,
}

impl NetDevResult {
    pub fn calculate(data: &ExtractedData, revision: &RevisionInfo) -> Self {
        let news_dev = data.actual - data.forecast;
        let revision_dev = revision.revision_delta;
        let net_dev = news_dev + revision_dev;
        let abs_net = net_dev.abs();
        Self {
            net_dev,
            is_valid: abs_net >= 2.0,
            is_extreme: abs_net >= 5.0,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_data(
        actual: f64,
        forecast: f64,
        orig_prev: f64,
        rev_prev: f64,
    ) -> (ExtractedData, RevisionInfo) {
        let data = ExtractedData {
            actual,
            forecast,
            original_previous: orig_prev,
            revised_previous: rev_prev,
        };
        let revision = RevisionInfo::new(orig_prev, rev_prev);
        (data, revision)
    }

    #[test]
    fn test_valid_signal() {
        let (d, r) = make_data(5.0, 1.0, 0.5, 0.5);
        let result = NetDevResult::calculate(&d, &r);
        assert!(result.is_valid);
        assert!((result.net_dev - 4.0).abs() < 0.01);
    }

    #[test]
    fn test_weak_signal() {
        let (d, r) = make_data(1.5, 1.0, 0.8, 0.8);
        let result = NetDevResult::calculate(&d, &r);
        assert!(!result.is_valid);
    }

    #[test]
    fn test_revision_amplifies() {
        let (d, r) = make_data(1.5, 1.0, 1.0, 2.5);
        let result = NetDevResult::calculate(&d, &r);
        assert!(result.is_valid);
        assert!((result.net_dev - 2.0).abs() < 0.01);
    }

    #[test]
    fn test_extreme_net_dev() {
        let (d, r) = make_data(10.0, 1.0, 0.5, 0.5);
        let result = NetDevResult::calculate(&d, &r);
        assert!(result.is_extreme);
    }
}
