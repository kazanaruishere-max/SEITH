// L2 - Data Extractor
// Parse Actual, Forecast, Previous values from scraped news

use super::red_folder_detector::NewsEvent;

#[derive(Debug, Clone)]
pub struct ExtractedData {
    pub actual: f64,
    pub forecast: f64,
    pub original_previous: f64,
    pub revised_previous: f64,
}

pub fn extract_values(event: &NewsEvent) -> Option<ExtractedData> {
    let actual = parse_value(event.actual.as_deref()?)?;
    let forecast = parse_value(event.forecast.as_deref()?)?;
    let previous = parse_value(event.previous.as_deref()?)?;

    Some(ExtractedData {
        actual,
        forecast,
        original_previous: previous,
        revised_previous: previous,
    })
}

pub fn extract_with_revision(event: &NewsEvent, revised: Option<&str>) -> Option<ExtractedData> {
    let mut data = extract_values(event)?;
    if let Some(rev) = revised.and_then(parse_value) {
        data.revised_previous = rev;
    }
    Some(data)
}

fn parse_value(s: &str) -> Option<f64> {
    let cleaned: String = s
        .chars()
        .filter(|c| c.is_ascii_digit() || *c == '.' || *c == '-')
        .collect();
    if cleaned.is_empty() || cleaned == "-" {
        return None;
    }
    cleaned.parse::<f64>().ok()
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;

    fn make_event(actual: &str, forecast: &str, previous: &str) -> NewsEvent {
        NewsEvent {
            time: Utc::now(),
            currency: "USD".to_string(),
            impact: "High".to_string(),
            title: "Test".to_string(),
            actual: Some(actual.to_string()),
            forecast: Some(forecast.to_string()),
            previous: Some(previous.to_string()),
        }
    }

    #[test]
    fn test_extract_normal() {
        let e = make_event("1.2", "1.0", "0.8");
        let data = extract_values(&e).unwrap();
        assert!((data.actual - 1.2).abs() < 0.01);
    }

    #[test]
    fn test_extract_missing_previous() {
        let e = make_event("1.2", "1.0", "N/A");
        assert!(extract_values(&e).is_none());
    }

    #[test]
    fn test_with_revision() {
        let e = make_event("1.2", "1.0", "0.8");
        let data = extract_with_revision(&e, Some("0.5")).unwrap();
        assert!((data.revised_previous - 0.5).abs() < 0.01);
    }

    #[test]
    fn test_parse_k_format() {
        let v = parse_value("125K").unwrap();
        assert!((v - 125.0).abs() < 0.01);
    }
}
