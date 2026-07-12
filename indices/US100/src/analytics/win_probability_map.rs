use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct WinProbabilityMap {
    pub mode_map: HashMap<String, WinStats>,
    pub session_map: HashMap<String, WinStats>,
}

#[derive(Debug, Clone)]
pub struct WinStats {
    pub wins: u32,
    pub losses: u32,
    pub total_pl: f64,
}

impl WinStats {
    pub fn new() -> Self {
        Self { wins: 0, losses: 0, total_pl: 0.0 }
    }

    pub fn win_rate(&self) -> f64 {
        let total = self.wins + self.losses;
        if total == 0 { 0.0 } else { self.wins as f64 / total as f64 * 100.0 }
    }

    pub fn record_trade(&mut self, is_win: bool, pl: f64) {
        if is_win { self.wins += 1 } else { self.losses += 1 }
        self.total_pl += pl;
    }
}

impl WinProbabilityMap {
    pub fn new() -> Self {
        Self { mode_map: HashMap::new(), session_map: HashMap::new() }
    }

    pub fn record(&mut self, mode: &str, session_phase: &str, is_win: bool, pl: f64) {
        self.mode_map.entry(mode.to_string()).or_insert(WinStats::new()).record_trade(is_win, pl);
        self.session_map.entry(session_phase.to_string()).or_insert(WinStats::new()).record_trade(is_win, pl);
    }

    pub fn get_mode_stats(&self, mode: &str) -> Option<&WinStats> {
        self.mode_map.get(mode)
    }

    pub fn get_session_stats(&self, phase: &str) -> Option<&WinStats> {
        self.session_map.get(phase)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty_map() {
        let map = WinProbabilityMap::new();
        assert!(map.get_mode_stats("SNIPER").is_none());
    }

    #[test]
    fn test_record_win() {
        let mut map = WinProbabilityMap::new();
        map.record("SNIPER", "NORMAL", true, 3.0);
        let stats = map.get_mode_stats("SNIPER").unwrap();
        assert_eq!(stats.wins, 1);
        assert!(stats.win_rate() - 100.0 < 1e-6);
    }

    #[test]
    fn test_record_loss() {
        let mut map = WinProbabilityMap::new();
        map.record("SNIPER", "NORMAL", false, -1.0);
        let stats = map.get_mode_stats("SNIPER").unwrap();
        assert_eq!(stats.losses, 1);
        assert!(stats.win_rate() - 0.0 < 1e-6);
    }

    #[test]
    fn test_multiple_trades() {
        let mut map = WinProbabilityMap::new();
        map.record("SNIPER", "NORMAL", true, 3.0);
        map.record("SNIPER", "NORMAL", true, 2.0);
        map.record("SNIPER", "NORMAL", false, -1.0);
        let stats = map.get_mode_stats("SNIPER").unwrap();
        assert_eq!(stats.wins, 2);
        assert_eq!(stats.losses, 1);
        assert!((stats.win_rate() - (2.0/3.0*100.0)).abs() < 1e-6);
        assert!((stats.total_pl - 4.0).abs() < 1e-6);
    }

    #[test]
    fn test_session_map_separate() {
        let mut map = WinProbabilityMap::new();
        map.record("SNIPER", "NORMAL", true, 3.0);
        map.record("SCALP", "POWER_HOUR", true, 1.0);
        assert_eq!(map.get_mode_stats("SNIPER").unwrap().wins, 1);
        assert_eq!(map.get_mode_stats("SCALP").unwrap().wins, 1);
    }

    #[test]
    fn test_win_stats_empty() {
        let s = WinStats::new();
        assert!((s.win_rate() - 0.0).abs() < 1e-6);
    }
}
