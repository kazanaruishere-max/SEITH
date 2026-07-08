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
