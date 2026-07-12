pub struct SCvd;

impl SCvd {
    pub fn new() -> Self {
        Self
    }

    /// Hitung cumulative delta dari array per-bar delta
    pub fn compute_cumulative(deltas: &[f64]) -> Vec<f64> {
        let mut cum = Vec::with_capacity(deltas.len());
        let mut sum = 0.0;
        for d in deltas {
            sum += d;
            cum.push(sum);
        }
        cum
    }

    /// Deteksi divergensi antara price dan cumulative delta
    /// Return S_CVD score -1..+1:
    ///   +1 = bullish divergence (price turun, CVD naik)
    ///   -1 = bearish divergence (price naik, CVD turun)
    ///    0 = no divergence
    pub fn detect_divergence(prices: &[f64], cumulative_delta: &[f64], lookback: usize) -> f64 {
        let n = prices.len().min(cumulative_delta.len());
        if n < lookback + 1 {
            return 0.0;
        }
        let start = n - lookback;
        let price_start = prices[start];
        let price_end = prices[n - 1];
        let cvd_start = cumulative_delta[start];
        let cvd_end = cumulative_delta[n - 1];
        let price_dir = (price_end - price_start).signum();
        let cvd_dir = (cvd_end - cvd_start).signum();

        if price_dir > 0.0 && cvd_dir < 0.0 {
            -1.0 // bearish divergence
        } else if price_dir < 0.0 && cvd_dir > 0.0 {
            1.0 // bullish divergence
        } else {
            0.0
        }
    }

    /// S_CVD: normalized CVD score -1..+1
    /// Menggunakan z-score dari cumulative delta terbaru
    pub fn compute_score(cumulative_delta: &[f64]) -> f64 {
        if cumulative_delta.len() < 2 {
            return 0.0;
        }
        let n = cumulative_delta.len() as f64;
        let mean = cumulative_delta.iter().sum::<f64>() / n;
        let variance = cumulative_delta.iter().map(|v| (v - mean).powi(2)).sum::<f64>() / (n - 1.0);
        if variance < 1e-10 {
            return 0.0;
        }
        let last = *cumulative_delta.last().unwrap();
        let z = (last - mean) / variance.sqrt();
        z.clamp(-1.0, 1.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cumulative_empty() {
        let r = SCvd::compute_cumulative(&[]);
        assert!(r.is_empty());
    }

    #[test]
    fn test_cumulative_single() {
        let r = SCvd::compute_cumulative(&[0.5]);
        assert!((r[0] - 0.5).abs() < 1e-6);
    }

    #[test]
    fn test_cumulative_accumulates() {
        let r = SCvd::compute_cumulative(&[0.3, -0.1, 0.2]);
        assert!((r[0] - 0.3).abs() < 1e-6);
        assert!((r[1] - 0.2).abs() < 1e-6);
        assert!((r[2] - 0.4).abs() < 1e-6);
    }

    #[test]
    fn test_divergence_bullish() {
        let prices = vec![100.0, 99.0, 98.0, 97.0];
        let cvd = vec![0.0, 0.5, 1.0, 1.5];
        let d = SCvd::detect_divergence(&prices, &cvd, 3);
        assert!((d - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_divergence_bearish() {
        let prices = vec![100.0, 101.0, 102.0, 103.0];
        let cvd = vec![0.0, -0.5, -1.0, -1.5];
        let d = SCvd::detect_divergence(&prices, &cvd, 3);
        assert!((d + 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_no_divergence() {
        let prices = vec![100.0, 101.0, 102.0];
        let cvd = vec![0.0, 0.5, 1.0];
        let d = SCvd::detect_divergence(&prices, &cvd, 2);
        assert!((d - 0.0).abs() < 1e-6);
    }

    #[test]
    fn test_divergence_short_window() {
        let d = SCvd::detect_divergence(&[100.0], &[0.0], 5);
        assert!((d - 0.0).abs() < 1e-6);
    }

    #[test]
    fn test_score_zero_for_constant() {
        let s = SCvd::compute_score(&[1.0, 1.0, 1.0]);
        assert!((s - 0.0).abs() < 1e-6);
    }

    #[test]
    fn test_score_positive_for_increasing() {
        let s = SCvd::compute_score(&[0.0, 0.5, 1.0, 1.5, 2.0]);
        assert!(s > 0.0);
    }

    #[test]
    fn test_score_negative_for_decreasing() {
        let s = SCvd::compute_score(&[0.0, -0.5, -1.0, -1.5, -2.0]);
        assert!(s < 0.0);
    }
}
