// L0 - Jam Hantu (Ghost Hour Protection)
// Force close at 20:45 broker time (risk of spread lock rollover)

use chrono::{DateTime, Timelike, Utc};

const JAM_HANTU_HOUR: u32 = 20;
const JAM_HANTU_MINUTE: u32 = 45;

pub fn is_jam_hantu_now(now: &DateTime<Utc>) -> bool {
    now.hour() == JAM_HANTU_HOUR && now.minute() == JAM_HANTU_MINUTE
}

pub fn is_jam_hantu_window(now: &DateTime<Utc>) -> bool {
    let total_min = now.hour() * 60 + now.minute();
    let target = JAM_HANTU_HOUR * 60 + JAM_HANTU_MINUTE;
    total_min >= target && total_min < target + 5
}

pub async fn force_close_all() {
    log::warn!("Jam Hantu: force closing all Tier 2 positions");
    todo!("Force close via MT5 API")
}

pub fn minutes_to_jam_hantu(now: &DateTime<Utc>) -> i64 {
    let target = JAM_HANTU_HOUR * 60 + JAM_HANTU_MINUTE;
    let current = now.hour() as i64 * 60 + now.minute() as i64;
    target as i64 - current
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;

    #[test]
    fn test_exact_jam_hantu() {
        let t = Utc.with_ymd_and_hms(2026, 7, 8, 20, 45, 0).unwrap();
        assert!(is_jam_hantu_now(&t));
    }

    #[test]
    fn test_not_jam_hantu() {
        let t = Utc.with_ymd_and_hms(2026, 7, 8, 14, 30, 0).unwrap();
        assert!(!is_jam_hantu_now(&t));
    }

    #[test]
    fn test_jam_hantu_window() {
        let t = Utc.with_ymd_and_hms(2026, 7, 8, 20, 47, 0).unwrap();
        assert!(is_jam_hantu_window(&t));
    }

    #[test]
    fn test_outside_window() {
        let t = Utc.with_ymd_and_hms(2026, 7, 8, 20, 55, 0).unwrap();
        assert!(!is_jam_hantu_window(&t));
    }
}
