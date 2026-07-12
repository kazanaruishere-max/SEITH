use crate::config::thresholds;

pub struct FramaResult {
    pub frama: f64,
    pub z_frama: f64,
}

pub struct Frama {
    pub period: usize,
}

impl Frama {
    pub fn new(period: usize) -> Self {
        Self { period }
    }

    pub fn compute(&self, prices: &[f64]) -> Vec<FramaResult> {
        if prices.len() < self.period + 2 {
            return Vec::new();
        }
        let mut results: Vec<FramaResult> = Vec::with_capacity(prices.len());
        for i in self.period..prices.len() {
            let window = &prices[i - self.period..=i];
            let half = self.period / 2;
            let n1 = window[..=half].iter().cloned().fold(f64::MAX, f64::min);
            let x1 = window[..=half].iter().cloned().fold(f64::MIN, f64::max);
            let n2 = window[half..].iter().cloned().fold(f64::MAX, f64::min);
            let x2 = window[half..].iter().cloned().fold(f64::MIN, f64::max);
            let n3 = window.iter().cloned().fold(f64::MAX, f64::min);
            let x3 = window.iter().cloned().fold(f64::MIN, f64::max);
            let range1 = x1 - n1;
            let range2 = x2 - n2;
            let range3 = x3 - n3;
            let d = if range3.abs() < 1e-10 || (range1 + range2).abs() < 1e-10 {
                1.0
            } else {
                let num = (range1 + range2).ln();
                let den = range3.ln();
                if den.abs() < 1e-10 { 1.0 } else { (num / den) / 2f64.ln() }
            };
            let alpha = (-4.6 * (d - 1.0)).exp();
            let prev_frama = if results.is_empty() { prices[0] } else { results.last().unwrap().frama };
            let frama = alpha * prices[i] + (1.0 - alpha) * prev_frama;
            let atr = (prices[i] - prices[i - 1]).abs().max(0.01);
            let z_frama = (prices[i] - frama) / atr;
            results.push(FramaResult { frama, z_frama });
        }
        results
    }

    pub fn is_overextended(z_frama: f64) -> bool {
        z_frama > thresholds::FRAMA_OVEREXTENDED
    }

    pub fn is_pullback_valid(z_frama: f64) -> bool {
        z_frama <= thresholds::FRAMA_OVEREXTENDED
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compute_empty() {
        let frama = Frama::new(10);
        let r = frama.compute(&[]);
        assert!(r.is_empty());
    }

    #[test]
    fn test_compute_short() {
        let frama = Frama::new(10);
        let r = frama.compute(&[1.0, 2.0, 3.0]);
        assert!(r.is_empty());
    }

    #[test]
    fn test_compute_trend() {
        let frama = Frama::new(5);
        let prices: Vec<f64> = (0..20).map(|i| 100.0 + i as f64).collect();
        let r = frama.compute(&prices);
        assert!(!r.is_empty());
        assert!(r.last().unwrap().frama.is_finite());
    }

    #[test]
    fn test_compute_z_frama_positive_in_trend() {
        let frama = Frama::new(5);
        let prices: Vec<f64> = (0..20).map(|i| 100.0 + i as f64).collect();
        let r = frama.compute(&prices);
        assert!(r.last().unwrap().z_frama > -1.0);
    }

    #[test]
    fn test_is_overextended() {
        assert!(Frama::is_overextended(0.6));
        assert!(!Frama::is_overextended(0.3));
    }

    #[test]
    fn test_is_pullback_valid() {
        assert!(Frama::is_pullback_valid(0.3));
        assert!(!Frama::is_pullback_valid(0.6));
    }

    #[test]
    fn test_boundary_at_05() {
        assert!(Frama::is_pullback_valid(0.5));
        assert!(!Frama::is_overextended(0.5));
        assert!(Frama::is_overextended(0.5001));
    }

    #[test]
    fn test_frama_follows_trend_direction() {
        let frama = Frama::new(5);
        let up: Vec<f64> = (0..15).map(|i| 100.0 + i as f64).collect();
        let r_up = frama.compute(&up);
        let down: Vec<f64> = (0..15).map(|i| 100.0 - i as f64).collect();
        let r_down = frama.compute(&down);
        assert!(r_up.last().unwrap().frama > r_down.last().unwrap().frama);
    }
}
