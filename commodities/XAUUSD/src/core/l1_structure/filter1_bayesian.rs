// Filter 1 — Bayesian Gatekeeper
// P(A|B) = (P(B|A) × P(A)) / P(B)
// < 60% → BLOCK | 60-74% → TIER 2 | ≥ 75% → TIER 1

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BayesianDecision {
    Block,
    Tier2Tactical,
    Tier1Institutional,
}

#[derive(Debug, Clone)]
pub struct BayesianResult {
    pub posterior_probability: f64,
    pub decision: BayesianDecision,
}

pub fn calculate_posterior(prior: f64, likelihood: f64, evidence: f64) -> f64 {
    if evidence == 0.0 {
        return 0.0;
    }
    (likelihood * prior) / evidence
}

pub fn evaluate_bayesian(posterior: f64) -> BayesianResult {
    let decision = if posterior < 0.60 {
        BayesianDecision::Block
    } else if posterior < 0.75 {
        BayesianDecision::Tier2Tactical
    } else {
        BayesianDecision::Tier1Institutional
    };
    BayesianResult {
        posterior_probability: posterior,
        decision,
    }
}

pub fn tier2_rr() -> (f64, f64) {
    (1.0, 1.2)
}
pub fn tier2_sl_location() -> &'static str {
    "FRAMA"
}
pub fn tier1_rr() -> (f64, f64) {
    (2.0, 2.5)
}
pub fn tier1_sl_location() -> &'static str {
    "Outer Liquidity Pool"
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_block_below_60() {
        let r = evaluate_bayesian(0.45);
        assert!(matches!(r.decision, BayesianDecision::Block));
    }

    #[test]
    fn test_tier2_at_60() {
        let r = evaluate_bayesian(0.60);
        assert!(matches!(r.decision, BayesianDecision::Tier2Tactical));
    }

    #[test]
    fn test_tier1_at_75() {
        let r = evaluate_bayesian(0.75);
        assert!(matches!(r.decision, BayesianDecision::Tier1Institutional));
    }

    #[test]
    fn test_posterior_calculation() {
        let p = calculate_posterior(0.5, 0.8, 0.6);
        assert!((p - 0.6667).abs() < 0.01);
    }

    #[test]
    fn test_zero_evidence() {
        let p = calculate_posterior(0.5, 0.8, 0.0);
        assert!((p - 0.0).abs() < 0.01);
    }
}
