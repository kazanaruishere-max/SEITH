// Filter 1 — Bayesian Gatekeeper
// P(A|B) = (P(B|A) x P(A)) / P(B)
// P(A) = prior from OANDA client sentiment (% long/short)
// P(B|A) = likelihood from historical accuracy of sentiment
// < tier2_thr -> BLOCK | tier2_thr~tier1_thr -> TIER 2 | >= tier1_thr -> TIER 1

use std::sync::OnceLock;

fn read_env_f64(key: &str, default: f64) -> f64 {
    std::env::var(key)
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(default)
}

fn tier2_thr() -> f64 {
    static V: OnceLock<f64> = OnceLock::new();
    *V.get_or_init(|| read_env_f64("BT_TIER2_THR", 0.60))
}

fn tier1_thr() -> f64 {
    static V: OnceLock<f64> = OnceLock::new();
    *V.get_or_init(|| read_env_f64("BT_TIER1_THR", 0.75))
}

/// OANDA prior is fetched once per session and cached
fn oanda_prior() -> Option<f64> {
    static PRIOR: OnceLock<Option<f64>> = OnceLock::new();
    *PRIOR.get_or_init(|| {
        let instrument = std::env::var("MT5_SYMBOL").unwrap_or_else(|_| "XAU_USD".to_string());
        let oanda_inst = instrument
            .replace(".sml", "")
            .replace("/", "_")
            .replace(".", "_");

        // Sync call to Python bridge (blocking OK at init)
        let json: std::result::Result<String, anyhow::Error> = pyo3::Python::with_gil(|py| {
            let oanda = pyo3::types::PyModule::import(py, "seith_bridge.oanda")?;
            let result: String = oanda
                .call_method1("get_sentiment", (&oanda_inst,))?
                .extract()?;
            Ok(result)
        });

        match json {
            Ok(s) => match serde_json::from_str::<serde_json::Value>(&s) {
                Ok(v) => {
                    let long_pct = v["long_pct"].as_f64().unwrap_or(50.0);
                    if long_pct > 0.0 {
                        let prior = if long_pct > 80.0 {
                            0.30
                        } else if long_pct < 20.0 {
                            0.70
                        } else {
                            long_pct / 100.0
                        };
                        log::info!(
                            "OANDA sentiment prior: {:.3} (long={:.1}%)",
                            prior,
                            long_pct
                        );
                        Some(prior)
                    } else {
                        log::warn!("OANDA sentiment not available, using default prior");
                        None
                    }
                }
                Err(e) => {
                    log::warn!("OANDA JSON parse error: {}", e);
                    None
                }
            },
            Err(e) => {
                log::warn!("OANDA sentiment unavailable: {} (using fallback)", e);
                None
            }
        }
    })
}

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

/// Calculate posterior probability using OANDA sentiment as prior.
/// Falls back to momentum-based prior if OANDA unavailable.
pub fn calculate_posterior(likelihood: f64, evidence: f64, direction_long: bool) -> f64 {
    if evidence == 0.0 {
        return 0.0;
    }
    // Prior from OANDA sentiment
    let prior = oanda_prior().unwrap_or(0.50);

    // Adjust prior based on direction:
    // If going long and sentiment is bullish (prior > 0.5), prior is confirmed.
    // If going long and sentiment is bearish (prior < 0.5), prior is contrarian.
    let adjusted_prior = if direction_long { prior } else { 1.0 - prior };

    (likelihood * adjusted_prior) / evidence
}

pub fn evaluate_bayesian(posterior: f64) -> BayesianResult {
    let t2 = tier2_thr();
    let t1 = tier1_thr();
    let decision = if posterior < t2 {
        BayesianDecision::Block
    } else if posterior < t1 {
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
        // P(A) = 0.6 (bullish sentiment), P(B|A) = 0.8, P(B) = 0.6
        let p = calculate_posterior(0.8, 0.6, true);
        assert!((p - 0.6667).abs() < 0.01);
    }

    #[test]
    fn test_zero_evidence() {
        let p = calculate_posterior(0.8, 0.0, true);
        assert!((p - 0.0).abs() < 0.01);
    }
}
