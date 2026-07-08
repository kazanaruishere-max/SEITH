// Signal Enricher — Add metadata to signal

use super::signal_types::Signal;

#[derive(Debug, Clone)]
pub struct EnrichedSignal {
    pub signal: Signal,
    pub confidence: f64,
    pub tier: String,
}

pub fn enrich(signal: Signal, confidence: f64, tier: &str) -> EnrichedSignal {
    EnrichedSignal {
        signal,
        confidence,
        tier: tier.to_string(),
    }
}
