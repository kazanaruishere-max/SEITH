#[derive(Debug, Clone)]
pub struct SignalMetadata {
    pub hv_z: f64,
    pub hv_regime: String,
    pub yield_verdict: String,
    pub ofs: f64,
    pub gates_passed: u32,
    pub reduce_lot: bool,
}

#[derive(Debug, Clone)]
pub enum Signal {
    Buy { confidence: f64, entry: f64, sl: f64, tp: f64, lot: f64, metadata: SignalMetadata },
    Sell { confidence: f64, entry: f64, sl: f64, tp: f64, lot: f64, metadata: SignalMetadata },
    NoSignal { reason: String },
}

impl Signal {
    pub fn confidence(&self) -> f64 {
        match self {
            Signal::Buy { confidence, .. } => *confidence,
            Signal::Sell { confidence, .. } => *confidence,
            Signal::NoSignal { .. } => 0.0,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn dummy_metadata() -> SignalMetadata {
        SignalMetadata {
            hv_z: 0.5, hv_regime: "normal".into(), yield_verdict: "neutral".into(),
            ofs: 3.5, gates_passed: 5, reduce_lot: false,
        }
    }

    #[test]
    fn test_buy_signal() {
        let s = Signal::Buy { confidence: 1.0, entry: 100.0, sl: 99.0, tp: 103.0, lot: 1.0, metadata: dummy_metadata() };
        assert!((s.confidence() - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_sell_signal() {
        let s = Signal::Sell { confidence: 0.8, entry: 100.0, sl: 101.0, tp: 97.0, lot: 0.5, metadata: dummy_metadata() };
        assert!((s.confidence() - 0.8).abs() < 1e-6);
    }

    #[test]
    fn test_no_signal() {
        let s = Signal::NoSignal { reason: "macro red".into() };
        assert!((s.confidence() - 0.0).abs() < 1e-6);
    }

    #[test]
    fn test_metadata() {
        let m = dummy_metadata();
        assert!((m.ofs - 3.5).abs() < 1e-6);
        assert_eq!(m.gates_passed, 5);
    }
}
