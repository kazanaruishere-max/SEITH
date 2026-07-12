pub mod s_cvd;
pub mod s_delta;
pub mod s_dom;

use crate::config::thresholds;

/// OFS gabungan: S_Delta + S_CVD + S_DOM
/// Range teoritis -3..+3 (masing-masing kontributor -1..+1)
pub fn compute_ofs(delta: f64, cvd: f64, dom: f64) -> f64 {
    (delta + cvd + dom).clamp(-3.0, 3.0)
}

pub enum OfsVerdict {
    Pass,
    Block,
}

pub fn evaluate_ofs(ofs: f64, ofs_threshold: f64) -> OfsVerdict {
    if ofs.abs() >= ofs_threshold {
        OfsVerdict::Pass
    } else {
        OfsVerdict::Block
    }
}

pub fn is_retail_noise(ofs: f64) -> bool {
    ofs.abs() <= thresholds::OFS_NOISE_MAX
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ofs_sum() {
        let ofs = compute_ofs(0.8, 0.6, 0.4);
        assert!((ofs - 1.8).abs() < 1e-6);
    }

    #[test]
    fn test_ofs_negative() {
        let ofs = compute_ofs(-0.5, -0.3, -0.7);
        assert!((ofs + 1.5).abs() < 1e-6);
    }

    #[test]
    fn test_ofs_clamped() {
        let ofs = compute_ofs(2.0, 2.0, 2.0);
        assert!((ofs - 3.0).abs() < 1e-6);
    }

    #[test]
    fn test_evaluate_pass() {
        assert!(matches!(evaluate_ofs(3.0, 3.0), OfsVerdict::Pass));
        assert!(matches!(evaluate_ofs(-3.0, 3.0), OfsVerdict::Pass));
        assert!(matches!(evaluate_ofs(2.0, 2.0), OfsVerdict::Pass));
    }

    #[test]
    fn test_evaluate_block() {
        assert!(matches!(evaluate_ofs(2.0, 3.0), OfsVerdict::Block));
        assert!(matches!(evaluate_ofs(1.5, 2.0), OfsVerdict::Block));
    }

    #[test]
    fn test_retail_noise() {
        assert!(is_retail_noise(0.5));
        assert!(is_retail_noise(1.0));
        assert!(!is_retail_noise(1.5));
    }

    #[test]
    fn test_signed_threshold() {
        // Both positive and negative should pass at same threshold
        assert!(matches!(evaluate_ofs(-3.0, 3.0), OfsVerdict::Pass));
        assert!(matches!(evaluate_ofs(3.0, 3.0), OfsVerdict::Pass));
    }
}

