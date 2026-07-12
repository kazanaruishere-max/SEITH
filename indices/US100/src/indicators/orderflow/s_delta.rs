pub struct SDelta;

impl SDelta {
    pub fn new() -> Self {
        Self
    }

    /// Hitung delta: normalisasi (buy_vol - sell_vol) / max_vol ke range -1..+1
    pub fn compute(buy_volume: f64, sell_volume: f64) -> f64 {
        let diff = buy_volume - sell_volume;
        let max = buy_volume.max(sell_volume);
        if max < 1e-10 {
            return 0.0;
        }
        (diff / max).clamp(-1.0, 1.0)
    }

    /// Delta dari array trade: [0]=buy, [1]=sell
    pub fn compute_from_trades(trades: &[(f64, f64)]) -> Vec<f64> {
        trades.iter().map(|(b, s)| Self::compute(*b, *s)).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_delta_buy_dominant() {
        let d = SDelta::compute(100.0, 30.0);
        assert!(d > 0.0);
        assert!(d <= 1.0);
    }

    #[test]
    fn test_delta_sell_dominant() {
        let d = SDelta::compute(20.0, 80.0);
        assert!(d < 0.0);
        assert!(d >= -1.0);
    }

    #[test]
    fn test_delta_equal() {
        let d = SDelta::compute(50.0, 50.0);
        assert!((d - 0.0).abs() < 1e-6);
    }

    #[test]
    fn test_delta_zero_volume() {
        let d = SDelta::compute(0.0, 0.0);
        assert!((d - 0.0).abs() < 1e-6);
    }

    #[test]
    fn test_delta_max_buy() {
        let d = SDelta::compute(100.0, 0.0);
        assert!((d - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_delta_max_sell() {
        let d = SDelta::compute(0.0, 100.0);
        assert!((d + 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_delta_from_trades() {
        let trades = vec![(100.0, 30.0), (20.0, 80.0), (50.0, 50.0)];
        let result = SDelta::compute_from_trades(&trades);
        assert_eq!(result.len(), 3);
        assert!(result[0] > 0.0);
        assert!(result[1] < 0.0);
        assert!((result[2] - 0.0).abs() < 1e-6);
    }
}
