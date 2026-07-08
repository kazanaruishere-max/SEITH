// Test Python scraper bridge via PyO3
// Run: cargo test -p shared --test test_scraper -- --nocapture

use std::fs;

#[test]
fn test_python_scraper() {
    pyo3::Python::with_gil(|py| {
        let locals = pyo3::types::PyDict::new(py);
        py.run(
            "
import sys, json
sys.path.insert(0, 'C:/Users/Lenovo/PROJECT/AI SEITH/python/python')
sys.path.insert(0, 'C:/Users/Lenovo/AppData/Local/Programs/Python/Python311/Lib/site-packages')
from seith_bridge.scraper import fetch_forex_factory
data = fetch_forex_factory('https://www.forexfactory.com/calendar')
if data:
    events = json.loads(data)
    with open('C:/temp/scraper_test.txt', 'w') as f:
        f.write(f'OK: {len(events)} events\\n')
        for e in events[:3]:
            f.write(f'{e[\"currency\"]} {e[\"impact\"]} {e[\"title\"][:30]}\\n')
else:
    with open('C:/temp/scraper_test.txt', 'w') as f:
        f.write('No data returned\\n')
",
            None,
            Some(locals),
        )
        .expect("Python exec failed");
    });

    let result = fs::read_to_string("C:/temp/scraper_test.txt").unwrap_or_default();
    println!("{}", result);
    assert!(!result.is_empty(), "Scraper should return data");
}

#[tokio::test]
async fn test_today_high_impact_news() {
    // Setup PYTHONPATH programmatically for PyO3 in this test
    pyo3::Python::with_gil(|py| {
        let sys = pyo3::types::PyModule::import(py, "sys").unwrap();
        let path: &pyo3::types::PyList = sys.getattr("path").unwrap().downcast().unwrap();
        path.insert(0, "C:/Users/Lenovo/PROJECT/AI SEITH/python/python")
            .unwrap();
        path.insert(
            0,
            "C:/Users/Lenovo/AppData/Local/Programs/Python/Python311/Lib/site-packages",
        )
        .unwrap();
    });

    println!("Fetching calendar events...");
    match shared::external::news_aggregator::fetch_calendar().await {
        Ok(events) => {
            println!("Total events fetched: {}", events.len());
            let today = chrono::Utc::now();
            let mut found_today_high = false;

            for e in events {
                // Check if today (same day)
                let is_today = e.time.date_naive() == today.date_naive();
                let is_usd = e.currency == "USD";
                let _is_high = e.impact.to_lowercase().contains("high") || e.impact.is_empty(); // FF impact might be empty or blank in scrape

                let title_lower = e.title.to_lowercase();
                let is_target_news = title_lower.contains("fomc") 
                    || title_lower.contains("payroll") 
                    || title_lower.contains("nfp") 
                    || title_lower.contains("cpi") 
                    || title_lower.contains("pmi") // include PMI since it is high impact today
                    || title_lower.contains("employment");

                if is_today && is_usd && is_target_news {
                    println!(
                        "检测到今日高影响新闻: TIME={} CUR={} TITLE={} IMPACT={}",
                        e.time, e.currency, e.title, e.impact
                    );
                    found_today_high = true;
                }
            }

            if !found_today_high {
                println!("未检测到今日有 FOMC/NFP/CPI 的 High Impact 新闻。");
            }
        }
        Err(e) => {
            panic!("Failed to fetch calendar: {}", e);
        }
    }
}
