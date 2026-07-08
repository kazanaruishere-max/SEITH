// L2 - Red Folder Detector
// Identify USD high-impact news events within T-30 to T-60 minute window

use chrono::{DateTime, Utc};

#[derive(Debug, Clone)]
pub struct NewsEvent {
    pub time: DateTime<Utc>,
    pub currency: String,
    pub impact: String,
    pub title: String,
    pub actual: Option<String>,
    pub forecast: Option<String>,
    pub previous: Option<String>,
}

const RED_CURRENCIES: &[&str] = &["USD"];
const RED_IMPACTS: &[&str] = &["High", "RED", "Red Folder", "High Impact"];

pub fn is_red_folder(event: &NewsEvent) -> bool {
    let currency_match = RED_CURRENCIES
        .iter()
        .any(|c| event.currency.to_uppercase().contains(c));
    let impact_match = RED_IMPACTS
        .iter()
        .any(|i| event.impact.to_lowercase().contains(&i.to_lowercase()));
    currency_match && impact_match
}

pub fn find_red_folder_events(events: &[NewsEvent], now: &DateTime<Utc>) -> Vec<NewsEvent> {
    let window_start = *now + chrono::Duration::minutes(30);
    let window_end = *now + chrono::Duration::minutes(60);

    events
        .iter()
        .filter(|e| is_red_folder(e) && e.time >= window_start && e.time <= window_end)
        .cloned()
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;

    fn make_event(time: DateTime<Utc>, currency: &str, impact: &str) -> NewsEvent {
        NewsEvent {
            time,
            currency: currency.to_string(),
            impact: impact.to_string(),
            title: format!("{} {} Event", currency, impact),
            actual: None,
            forecast: None,
            previous: None,
        }
    }

    #[test]
    fn test_red_folder_detected() {
        let e = make_event(Utc::now(), "USD", "High Impact");
        assert!(is_red_folder(&e));
    }

    #[test]
    fn test_non_red_folder() {
        let e = make_event(Utc::now(), "EUR", "Low");
        assert!(!is_red_folder(&e));
    }

    #[test]
    fn test_find_in_window() {
        let now = Utc.with_ymd_and_hms(2026, 7, 8, 12, 0, 0).unwrap();
        let in_window = now + chrono::Duration::minutes(45);
        let events = vec![make_event(in_window, "USD", "High")];
        let found = find_red_folder_events(&events, &now);
        assert_eq!(found.len(), 1);
    }

    #[test]
    fn test_outside_window() {
        let now = Utc.with_ymd_and_hms(2026, 7, 8, 12, 0, 0).unwrap();
        let too_soon = now + chrono::Duration::minutes(15);
        let events = vec![make_event(too_soon, "USD", "High")];
        let found = find_red_folder_events(&events, &now);
        assert_eq!(found.len(), 0);
    }
}
