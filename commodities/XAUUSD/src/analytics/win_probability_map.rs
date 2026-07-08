// Win Probability Map — Bayesian Win Rate per Setup Type

use std::collections::HashMap;

#[derive(Debug, Clone, Default)]
pub struct SetupStats {
    pub wins: u32,
    pub losses: u32,
    pub total: u32,
}

impl SetupStats {
    pub fn new() -> Self {
        Self {
            wins: 0,
            losses: 0,
            total: 0,
        }
    }

    pub fn win_rate(&self) -> f64 {
        if self.total == 0 {
            return 0.0;
        }
        self.wins as f64 / self.total as f64
    }
}

#[derive(Debug, Clone, Default)]
pub struct WinProbabilityMap {
    setups: HashMap<String, SetupStats>,
}

impl WinProbabilityMap {
    pub fn new() -> Self {
        Self {
            setups: HashMap::new(),
        }
    }

    pub fn record_result(&mut self, setup: &str, won: bool) {
        let stats = self.setups.entry(setup.to_string()).or_default();
        stats.total += 1;
        if won {
            stats.wins += 1
        } else {
            stats.losses += 1
        };
    }

    pub fn get_stats(&self, setup: &str) -> Option<&SetupStats> {
        self.setups.get(setup)
    }

    pub fn all_setups(&self) -> &HashMap<String, SetupStats> {
        &self.setups
    }

    pub fn best_setup(&self) -> Option<(&str, f64)> {
        self.setups
            .iter()
            .map(|(k, v)| (k.as_str(), v.win_rate()))
            .max_by(|a, b| a.1.partial_cmp(&b.1).unwrap())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_map_empty() {
        let m = WinProbabilityMap::new();
        assert!(m.all_setups().is_empty());
    }

    #[test]
    fn test_record_win() {
        let mut m = WinProbabilityMap::new();
        m.record_result("TIER_1_BUY", true);
        let s = m.get_stats("TIER_1_BUY").unwrap();
        assert_eq!(s.wins, 1);
        assert_eq!(s.win_rate(), 1.0);
    }

    #[test]
    fn test_mixed_results() {
        let mut m = WinProbabilityMap::new();
        m.record_result("TIER_2_SELL", true);
        m.record_result("TIER_2_SELL", false);
        let s = m.get_stats("TIER_2_SELL").unwrap();
        assert_eq!(s.win_rate(), 0.5);
    }

    #[test]
    fn test_best_setup() {
        let mut m = WinProbabilityMap::new();
        m.record_result("A", true);
        m.record_result("B", false);
        let (best, rate) = m.best_setup().unwrap();
        assert_eq!(best, "A");
        assert!((rate - 1.0).abs() < 0.01);
    }
}
