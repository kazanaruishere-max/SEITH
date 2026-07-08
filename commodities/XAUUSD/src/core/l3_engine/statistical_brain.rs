// L3 - Statistical Brain
// Historical analysis: volatility spike, slippage, spread max

#[derive(Debug, Clone, Default)]
pub struct StatsResult {
    pub avg_spread: f64,
    pub max_spread: f64,
    pub avg_slippage: f64,
    pub max_slippage: f64,
    pub avg_volatility_spike: f64,
    pub sample_count: usize,
}

impl StatsResult {
    pub fn spread_buffer_pips(&self) -> f64 {
        (self.avg_spread * 1.5).max(self.max_spread * 0.8)
    }
}

#[derive(Debug, Clone)]
pub struct StatisticalBrain {
    _samples: usize,
    max_samples: usize,
}

impl Default for StatisticalBrain {
    fn default() -> Self {
        Self::new()
    }
}

impl StatisticalBrain {
    pub fn new() -> Self {
        Self {
            _samples: 0,
            max_samples: 500,
        }
    }

    pub fn analyze_spreads(&self, spreads: &[f64]) -> StatsResult {
        let n = spreads.len().min(self.max_samples);
        if n == 0 {
            return StatsResult::default();
        }
        let avg = spreads.iter().sum::<f64>() / n as f64;
        let max = spreads.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
        StatsResult {
            avg_spread: avg,
            max_spread: max,
            sample_count: n,
            ..Default::default()
        }
    }

    pub fn analyze_slippages(&self, slippages: &[f64]) -> StatsResult {
        let n = slippages.len().min(self.max_samples);
        if n == 0 {
            return StatsResult::default();
        }
        let avg = slippages.iter().sum::<f64>() / n as f64;
        let max = slippages.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
        StatsResult {
            avg_slippage: avg,
            max_slippage: max,
            sample_count: n,
            ..Default::default()
        }
    }

    pub fn analyze_volatility(&self, candles: &[(f64, f64)]) -> StatsResult {
        let n = candles.len().min(self.max_samples);
        if n == 0 {
            return StatsResult::default();
        }
        let spikes: Vec<f64> = candles
            .iter()
            .map(|(high, low)| (high - low).abs())
            .collect();
        let avg = spikes.iter().sum::<f64>() / spikes.len() as f64;
        StatsResult {
            avg_volatility_spike: avg,
            sample_count: n,
            ..Default::default()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty_spread() {
        let brain = StatisticalBrain::new();
        let r = brain.analyze_spreads(&[]);
        assert_eq!(r.sample_count, 0);
    }

    #[test]
    fn test_spread_avg_max() {
        let brain = StatisticalBrain::new();
        let r = brain.analyze_spreads(&[1.0, 2.0, 3.0, 4.0, 5.0]);
        assert!((r.avg_spread - 3.0).abs() < 0.01);
        assert!((r.max_spread - 5.0).abs() < 0.01);
    }

    #[test]
    fn test_volatility() {
        let brain = StatisticalBrain::new();
        let candles = vec![(10.0, 8.0), (12.0, 9.0)];
        let r = brain.analyze_volatility(&candles);
        assert!((r.avg_volatility_spike - 2.5).abs() < 0.01);
    }

    #[test]
    fn test_buffer_pips() {
        let r = StatsResult {
            avg_spread: 2.0,
            max_spread: 3.5,
            sample_count: 100,
            ..Default::default()
        };
        let buf = r.spread_buffer_pips();
        assert!(buf > 0.0);
    }
}
