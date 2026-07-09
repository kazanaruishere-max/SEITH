// Filter 4 — Institutional Tracker (OFS)
// OFS = S_Delta + S_CVD + S_DOM
// -1, 0, +1 → BLOCK (retail noise)
// ≥ +2 atau ≤ -2 → PASS (institutional valid)

use std::sync::OnceLock;

fn ofs_min_valid() -> i32 {
    static V: OnceLock<i32> = OnceLock::new();
    *V.get_or_init(|| {
        std::env::var("BT_OFS_MIN")
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or(2)
    })
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OfsDecision {
    BlockRetailNoise,
    PassInstitutional,
}

#[derive(Debug, Clone)]
pub struct OfsResult {
    pub s_delta: i32,
    pub s_cvd: i32,
    pub s_dom: i32,
    pub ofs_total: i32,
    pub decision: OfsDecision,
}

pub fn calculate_ofs(s_delta: i32, s_cvd: i32, s_dom: i32) -> OfsResult {
    let total = s_delta + s_cvd + s_dom;
    let decision = if total.abs() >= ofs_min_valid() {
        OfsDecision::PassInstitutional
    } else {
        OfsDecision::BlockRetailNoise
    };
    OfsResult {
        s_delta,
        s_cvd,
        s_dom,
        ofs_total: total,
        decision,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pass_positive() {
        let r = calculate_ofs(1, 1, 1);
        assert!(matches!(r.decision, OfsDecision::PassInstitutional));
        assert_eq!(r.ofs_total, 3);
    }

    #[test]
    fn test_pass_negative() {
        let r = calculate_ofs(-1, -1, 0);
        assert!(matches!(r.decision, OfsDecision::PassInstitutional));
        assert_eq!(r.ofs_total, -2);
    }

    #[test]
    fn test_block_noise_positive() {
        let r = calculate_ofs(1, 0, 0);
        assert!(matches!(r.decision, OfsDecision::BlockRetailNoise));
    }

    #[test]
    fn test_block_noise_zero() {
        let r = calculate_ofs(0, 0, 0);
        assert!(matches!(r.decision, OfsDecision::BlockRetailNoise));
    }
}
