use crate::config::thresholds;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum HvVerdict {
    SkipLow,
    Pass,
    PassConfidenceDown,
    SkipHigh,
}

pub struct HvCompass;

impl HvCompass {
    pub fn new() -> Self {
        Self
    }

    pub fn evaluate(&self, hv_z: f64) -> HvVerdict {
        if hv_z < thresholds::HV_SKIP_LOW { HvVerdict::SkipLow }
        else if hv_z <= thresholds::HV_SWEET_SPOT_MAX { HvVerdict::Pass }
        else if hv_z <= thresholds::HV_ELEVATED_MAX { HvVerdict::PassConfidenceDown }
        else { HvVerdict::SkipHigh }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_skip_low() {
        let c = HvCompass::new();
        assert!(matches!(c.evaluate(-1.5), HvVerdict::SkipLow));
    }

    #[test]
    fn test_pass() {
        let c = HvCompass::new();
        assert!(matches!(c.evaluate(0.0), HvVerdict::Pass));
    }

    #[test]
    fn test_pass_confidence_down() {
        let c = HvCompass::new();
        assert!(matches!(c.evaluate(1.8), HvVerdict::PassConfidenceDown));
    }

    #[test]
    fn test_skip_high() {
        let c = HvCompass::new();
        assert!(matches!(c.evaluate(2.5), HvVerdict::SkipHigh));
    }

    #[test]
    fn test_boundary_15() {
        let c = HvCompass::new();
        assert!(matches!(c.evaluate(1.5), HvVerdict::Pass));
    }

    #[test]
    fn test_boundary_20() {
        let c = HvCompass::new();
        assert!(matches!(c.evaluate(2.0), HvVerdict::PassConfidenceDown));
    }
}
