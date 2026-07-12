use crate::core::l1_pipeline::pipeline_router::PipelineResult;
use crate::indicators::r#yield::YieldVerdict;
use crate::signals::signal_types::SignalMetadata;

pub struct SignalEnricher;

impl SignalEnricher {
    pub fn new() -> Self {
        Self
    }

    pub fn enrich(
        result: &PipelineResult,
        hv_z: f64,
        hv_regime: &str,
        yield_verdict: YieldVerdict,
        ofs: f64,
    ) -> SignalMetadata {
        SignalMetadata {
            hv_z,
            hv_regime: hv_regime.to_string(),
            yield_verdict: format!("{:?}", yield_verdict),
            ofs,
            gates_passed: result.gates_passed,
            reduce_lot: result.reduce_lot,
        }
    }

    pub fn confidence_from_enriched(metadata: &SignalMetadata) -> f64 {
        let mut conf = 1.0;
        if metadata.hv_z > 1.5 && metadata.hv_z <= 2.0 {
            conf *= 0.75;
        }
        if metadata.reduce_lot {
            conf *= 0.5;
        }
        conf
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::l1_pipeline::macro_gate::GateResult;
    use crate::core::l1_pipeline::pipeline_router::PipelineResult;

    fn sample_result() -> PipelineResult {
        PipelineResult {
            gate0: Some(GateResult::Pass), gate1: true, gate2: true,
            gate3: true, gate4: true, gates_passed: 5, reduce_lot: false,
        }
    }

    #[test]
    fn test_enrich_normal() {
        let r = sample_result();
        let m = SignalEnricher::enrich(&r, 0.5, "normal", YieldVerdict::Neutral, 3.5);
        assert!((m.hv_z - 0.5).abs() < 1e-6);
        assert_eq!(m.hv_regime, "normal");
        assert_eq!(m.yield_verdict, "Neutral");
        assert!((m.ofs - 3.5).abs() < 1e-6);
        assert_eq!(m.gates_passed, 5);
    }

    #[test]
    fn test_enrich_with_verdict() {
        let r = sample_result();
        let m = SignalEnricher::enrich(&r, 1.8, "elevated", YieldVerdict::Bearish, -3.0);
        assert_eq!(m.hv_regime, "elevated");
        assert_eq!(m.yield_verdict, "Bearish");
    }

    #[test]
    fn test_confidence_normal() {
        let m = SignalMetadata {
            hv_z: 0.5, hv_regime: "normal".into(), yield_verdict: "Neutral".into(),
            ofs: 3.5, gates_passed: 5, reduce_lot: false,
        };
        assert!((SignalEnricher::confidence_from_enriched(&m) - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_confidence_hv_elevated() {
        let m = SignalMetadata {
            hv_z: 1.8, hv_regime: "elevated".into(), yield_verdict: "Neutral".into(),
            ofs: 3.5, gates_passed: 5, reduce_lot: false,
        };
        assert!((SignalEnricher::confidence_from_enriched(&m) - 0.75).abs() < 1e-6);
    }

    #[test]
    fn test_confidence_reduce_lot() {
        let m = SignalMetadata {
            hv_z: 0.5, hv_regime: "normal".into(), yield_verdict: "Neutral".into(),
            ofs: 3.5, gates_passed: 5, reduce_lot: true,
        };
        assert!((SignalEnricher::confidence_from_enriched(&m) - 0.5).abs() < 1e-6);
    }
}
