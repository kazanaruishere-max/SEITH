use crate::config::settings;
use chrono::Utc;

pub struct AutoKill;

impl AutoKill {
    pub fn new() -> Self {
        Self
    }

    pub fn should_auto_kill(current_time_utc: chrono::NaiveTime) -> bool {
        let kill_time = settings::SESSION_CLOSE + chrono::Duration::minutes(settings::AUTO_KILL_DELAY_MINUTES);
        current_time_utc >= kill_time
    }

    pub fn time_until_auto_kill(current_time_utc: chrono::NaiveTime) -> chrono::Duration {
        let kill_time = settings::SESSION_CLOSE + chrono::Duration::minutes(settings::AUTO_KILL_DELAY_MINUTES);
        if current_time_utc >= kill_time {
            chrono::Duration::zero()
        } else {
            kill_time - current_time_utc
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::NaiveTime;

    #[test]
    fn test_should_auto_kill_after_close() {
        let t = NaiveTime::from_hms_opt(21, 10, 0).unwrap();
        assert!(AutoKill::should_auto_kill(t));
    }

    #[test]
    fn test_should_auto_kill_before_close() {
        let t = NaiveTime::from_hms_opt(20, 0, 0).unwrap();
        assert!(!AutoKill::should_auto_kill(t));
    }

    #[test]
    fn test_should_auto_kill_exact_boundary() {
        let t = NaiveTime::from_hms_opt(21, 5, 0).unwrap();
        assert!(AutoKill::should_auto_kill(t));
    }

    #[test]
    fn test_time_until_auto_kill() {
        let t = NaiveTime::from_hms_opt(20, 0, 0).unwrap();
        let remaining = AutoKill::time_until_auto_kill(t);
        assert!(remaining > chrono::Duration::zero());
    }

    #[test]
    fn test_time_until_auto_kill_zero_after() {
        let t = NaiveTime::from_hms_opt(22, 0, 0).unwrap();
        let remaining = AutoKill::time_until_auto_kill(t);
        assert_eq!(remaining, chrono::Duration::zero());
    }
}
