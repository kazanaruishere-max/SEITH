// GVZ Z-Score Calculator
// Market Regime Detector: (GVZ_Current - μ20) / σ20

const GVZ_PERIOD: usize = 20;

#[derive(Debug, Clone)]
pub struct GvzResult {
    pub current: f64,
    pub mean: f64,
    pub std_dev: f64,
    pub z_score: f64,
}

pub fn calculate_gvz_zscore(gvz_values: &[f64]) -> Option<GvzResult> {
    if gvz_values.len() < GVZ_PERIOD {
        return None;
    }
    let recent: Vec<f64> = gvz_values.iter().rev().take(GVZ_PERIOD).cloned().collect();
    let current = recent[0];
    let mean = recent.iter().sum::<f64>() / GVZ_PERIOD as f64;
    let variance = recent.iter().map(|v| (v - mean).powi(2)).sum::<f64>() / GVZ_PERIOD as f64;
    let std_dev = variance.sqrt();
    if std_dev == 0.0 {
        return Some(GvzResult {
            current,
            mean,
            std_dev: 0.0,
            z_score: 0.0,
        });
    }
    let z_score = (current - mean) / std_dev;
    Some(GvzResult {
        current,
        mean,
        std_dev,
        z_score,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_short_input() {
        assert!(calculate_gvz_zscore(&[1.0; 10]).is_none());
    }

    #[test]
    fn test_constant_values() {
        let r = calculate_gvz_zscore(&[15.0; 25]).unwrap();
        assert!((r.z_score - 0.0).abs() < 0.01);
    }

    #[test]
    fn test_spike_detected() {
        let mut vals = vec![15.0; 20];
        vals.push(25.0);
        let r = calculate_gvz_zscore(&vals).unwrap();
        assert!(r.z_score > 1.0);
    }
}
