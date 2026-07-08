// Signal Classifier
// Tier 1 — Institutional Ride (≥75% Bayesian, RR 1:2.0-2.5)
// Tier 2 — Tactical Scalp (60-74% Bayesian, RR 1:1.0-1.2)

use crate::core::l1_structure::filter1_bayesian::BayesianDecision;
use crate::core::l1_structure::filter2_cvar::CvarResult;
use crate::core::l1_structure::filter3_market_compass::CompassResult;
use crate::core::l1_structure::filter4_orderflow::OfsResult;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SignalTier {
    Tier1Institutional,
    Tier2Tactical,
    NoSignal,
}

#[derive(Debug, Clone)]
pub struct ClassifiedSignal {
    pub tier: SignalTier,
    pub risk_percent: f64,
    pub rr_min: f64,
    pub rr_max: f64,
    pub sl_location: &'static str,
}

pub fn classify_signal(
    bayesian: &BayesianDecision,
    cvar: &CvarResult,
    compass: &CompassResult,
    ofs: &OfsResult,
) -> ClassifiedSignal {
    if matches!(bayesian, BayesianDecision::Block) {
        return no_signal();
    }
    if !cvar.passed {
        return no_signal();
    }
    if !matches!(
        compass.decision,
        crate::core::l1_structure::filter3_market_compass::CompassDecision::Pass
    ) {
        return no_signal();
    }
    if matches!(
        ofs.decision,
        crate::core::l1_structure::filter4_orderflow::OfsDecision::BlockRetailNoise
    ) {
        return no_signal();
    }

    match bayesian {
        BayesianDecision::Tier1Institutional => ClassifiedSignal {
            tier: SignalTier::Tier1Institutional,
            risk_percent: 1.0,
            rr_min: 2.0,
            rr_max: 2.5,
            sl_location: "Outer Liquidity Pool",
        },
        BayesianDecision::Tier2Tactical => ClassifiedSignal {
            tier: SignalTier::Tier2Tactical,
            risk_percent: 0.5,
            rr_min: 1.0,
            rr_max: 1.2,
            sl_location: "FRAMA",
        },
        BayesianDecision::Block => no_signal(),
    }
}

fn no_signal() -> ClassifiedSignal {
    ClassifiedSignal {
        tier: SignalTier::NoSignal,
        risk_percent: 0.0,
        rr_min: 0.0,
        rr_max: 0.0,
        sl_location: "",
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::l1_structure::filter1_bayesian::BayesianDecision;
    use crate::core::l1_structure::filter2_cvar::*;
    use crate::core::l1_structure::filter3_market_compass::*;
    use crate::core::l1_structure::filter4_orderflow::*;

    fn cvar_ok() -> CvarResult {
        evaluate_cvar(100.0, &BayesianDecision::Tier2Tactical)
    }
    fn compass_pass() -> CompassResult {
        evaluate_compass(0.5, 0.3, 10.0, 1.0)
    }
    fn ofs_pass() -> OfsResult {
        calculate_ofs(1, 1, 1)
    }

    #[test]
    fn test_tier1_signal() {
        let s = classify_signal(
            &BayesianDecision::Tier1Institutional,
            &cvar_ok(),
            &compass_pass(),
            &ofs_pass(),
        );
        assert!(matches!(s.tier, SignalTier::Tier1Institutional));
    }

    #[test]
    fn test_tier2_signal() {
        let s = classify_signal(
            &BayesianDecision::Tier2Tactical,
            &cvar_ok(),
            &compass_pass(),
            &ofs_pass(),
        );
        assert!(matches!(s.tier, SignalTier::Tier2Tactical));
    }

    #[test]
    fn test_blocked_bayesian() {
        let s = classify_signal(
            &BayesianDecision::Block,
            &cvar_ok(),
            &compass_pass(),
            &ofs_pass(),
        );
        assert!(matches!(s.tier, SignalTier::NoSignal));
    }
}
