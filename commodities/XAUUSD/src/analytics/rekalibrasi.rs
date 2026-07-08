// Rekalibrasi — Post-Trade Cognitive Recalibration
// Optimasi spacing buffer, reset skip counter, update win probability map

use super::win_probability_map::WinProbabilityMap;

#[derive(Debug, Clone)]
pub struct RekalibrasiResult {
    pub new_spacing_buffer: f64,
    pub avg_slippage: f64,
    pub reset_skip_counter: bool,
}

pub fn rekalibrasi_spacing(avg_slippage: f64, current_spacing: f64) -> f64 {
    let optimal = current_spacing + avg_slippage * 0.5;
    (optimal * 100.0).round() / 100.0
}

pub fn rekalibrasi_win_map(win_map: &mut WinProbabilityMap, setup: &str, won: bool) {
    win_map.record_result(setup, won);
}

pub fn should_reset_skip_counter(trade_result: &str) -> bool {
    matches!(trade_result, "WIN" | "LOSS")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_spacing_optimization() {
        let new = rekalibrasi_spacing(0.3, 1.0);
        assert!(new > 1.0);
    }

    #[test]
    fn test_reset_on_result() {
        assert!(should_reset_skip_counter("WIN"));
        assert!(should_reset_skip_counter("LOSS"));
    }
}
