// Filter 3 — Market Compass
// GVZ Z-Score > +1.0 → HIGH VOL: FRAMA Trend Rider
// GVZ Z-Score ≤ +1.0 → LOW VOL: AMT Volume Profile + VWAP

use std::sync::OnceLock;

fn gvz_zscore_threshold() -> f64 {
    static V: OnceLock<f64> = OnceLock::new();
    *V.get_or_init(|| {
        std::env::var("BT_GVZ_THR")
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or(1.0)
    })
}

fn frama_deviation_max() -> f64 {
    static V: OnceLock<f64> = OnceLock::new();
    *V.get_or_init(|| {
        std::env::var("BT_FRAMA_DEV")
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or(0.5)
    })
}

pub const POC_MAGNET_PIPS: f64 = 5.0;
pub const VWAP_BAND_EXTREME: f64 = 2.5;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VolatilityRegime {
    HighVol,
    LowVol,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CompassDecision {
    Pass,
    BlockFomo,
    BlockMagnet,
    BlockOverextended,
}

#[derive(Debug, Clone)]
pub struct CompassResult {
    pub gvz_zscore: f64,
    pub regime: VolatilityRegime,
    pub decision: CompassDecision,
}

pub fn determine_regime(gvz_zscore: f64) -> VolatilityRegime {
    if gvz_zscore > gvz_zscore_threshold() {
        VolatilityRegime::HighVol
    } else {
        VolatilityRegime::LowVol
    }
}

pub fn evaluate_frama(frama_deviation: f64) -> CompassDecision {
    if frama_deviation.abs() <= frama_deviation_max() {
        CompassDecision::Pass
    } else {
        CompassDecision::BlockFomo
    }
}

pub fn evaluate_amt_vwap(poc_distance: f64, vwap_deviation: f64) -> CompassDecision {
    if poc_distance.abs() <= POC_MAGNET_PIPS {
        return CompassDecision::BlockMagnet;
    }
    if vwap_deviation.abs() > VWAP_BAND_EXTREME {
        return CompassDecision::BlockOverextended;
    }
    CompassDecision::Pass
}

pub fn evaluate_compass(
    gvz_zscore: f64,
    frama_dev: f64,
    poc_dist: f64,
    vwap_dev: f64,
) -> CompassResult {
    let regime = determine_regime(gvz_zscore);
    let decision = match regime {
        VolatilityRegime::HighVol => evaluate_frama(frama_dev),
        VolatilityRegime::LowVol => evaluate_amt_vwap(poc_dist, vwap_dev),
    };
    CompassResult {
        gvz_zscore,
        regime,
        decision,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_high_vol_regime() {
        assert!(matches!(determine_regime(1.5), VolatilityRegime::HighVol));
    }

    #[test]
    fn test_low_vol_regime() {
        assert!(matches!(determine_regime(0.5), VolatilityRegime::LowVol));
    }

    #[test]
    fn test_frama_pass() {
        assert!(matches!(evaluate_frama(0.3), CompassDecision::Pass));
    }

    #[test]
    fn test_frama_block() {
        assert!(matches!(evaluate_frama(0.7), CompassDecision::BlockFomo));
    }

    #[test]
    fn test_amt_magnet_block() {
        assert!(matches!(
            evaluate_amt_vwap(2.0, 1.0),
            CompassDecision::BlockMagnet
        ));
    }

    #[test]
    fn test_vwap_overextended_block() {
        assert!(matches!(
            evaluate_amt_vwap(10.0, 3.0),
            CompassDecision::BlockOverextended
        ));
    }

    #[test]
    fn test_amt_fair_area_pass() {
        assert!(matches!(
            evaluate_amt_vwap(10.0, 1.0),
            CompassDecision::Pass
        ));
    }
}
