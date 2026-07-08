// L3 - Anti-Paralysis Adaptation
// Threshold relaxation when skip_strike_count >= 3

#[derive(Debug, Clone)]
pub struct RelaxationParams {
    pub spread_tolerance_reduction: f64,
    pub ofs_threshold_reduction: i32,
    pub order_distance_reduction: f64,
}

impl RelaxationParams {
    pub fn normal() -> Self {
        Self {
            spread_tolerance_reduction: 1.0,
            ofs_threshold_reduction: 0,
            order_distance_reduction: 1.0,
        }
    }

    pub fn crisis() -> Self {
        Self {
            spread_tolerance_reduction: 0.85,
            ofs_threshold_reduction: 1,
            order_distance_reduction: 0.5,
        }
    }
}

#[derive(Debug, Clone)]
pub struct AntiParalysis {
    skip_strike_count: u32,
    max_skip_before_crisis: u32,
}

impl Default for AntiParalysis {
    fn default() -> Self {
        Self::new()
    }
}

impl AntiParalysis {
    pub fn new() -> Self {
        Self {
            skip_strike_count: 0,
            max_skip_before_crisis: 3,
        }
    }

    pub fn skip_count(&self) -> u32 {
        self.skip_strike_count
    }

    pub fn increment(&mut self) {
        self.skip_strike_count += 1;
        log::warn!("Anti-Paralysis: skip strike {}", self.skip_strike_count);
    }

    pub fn reset(&mut self) {
        if self.skip_strike_count > 0 {
            log::info!("Anti-Paralysis: reset from {} to 0", self.skip_strike_count);
        }
        self.skip_strike_count = 0;
    }

    pub fn is_crisis(&self) -> bool {
        self.skip_strike_count >= self.max_skip_before_crisis
    }

    pub fn should_force_entry(&self) -> bool {
        self.skip_strike_count >= self.max_skip_before_crisis
    }

    pub fn params(&self) -> RelaxationParams {
        if self.is_crisis() {
            RelaxationParams::crisis()
        } else {
            RelaxationParams::normal()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_initial_state() {
        let ap = AntiParalysis::new();
        assert!(!ap.is_crisis());
        assert_eq!(ap.skip_count(), 0);
    }

    #[test]
    fn test_increment() {
        let mut ap = AntiParalysis::new();
        ap.increment();
        ap.increment();
        assert_eq!(ap.skip_count(), 2);
        assert!(!ap.is_crisis());
    }

    #[test]
    fn test_crisis_at_three() {
        let mut ap = AntiParalysis::new();
        ap.increment();
        ap.increment();
        ap.increment();
        assert!(ap.is_crisis());
        assert!(ap.should_force_entry());
    }

    #[test]
    fn test_reset() {
        let mut ap = AntiParalysis::new();
        ap.increment();
        ap.increment();
        ap.increment();
        assert!(ap.is_crisis());
        ap.reset();
        assert_eq!(ap.skip_count(), 0);
        assert!(!ap.is_crisis());
    }

    #[test]
    fn test_normal_params() {
        let ap = AntiParalysis::new();
        let p = ap.params();
        assert!((p.spread_tolerance_reduction - 1.0).abs() < 0.01);
    }

    #[test]
    fn test_crisis_params() {
        let mut ap = AntiParalysis::new();
        ap.increment();
        ap.increment();
        ap.increment();
        let p = ap.params();
        assert!((p.spread_tolerance_reduction - 0.85).abs() < 0.01);
        assert_eq!(p.ofs_threshold_reduction, 1);
    }
}
