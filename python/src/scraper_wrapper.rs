// Web scraper wrapper — PyO3 bridge
// Wraps python/seith_bridge/scraper.py functions

#![allow(non_local_definitions)]

use pyo3::prelude::*;

#[pyclass]
pub struct ScraperWrapper;

#[pymethods]
impl ScraperWrapper {
    #[new]
    fn new() -> Self {
        Self
    }

    fn fetch_forex_factory(&self, url: &str) -> PyResult<Option<String>> {
        Python::with_gil(|py| {
            let sc = PyModule::import(py, "seith_bridge.scraper")?;
            let result = sc.call_method1("fetch_forex_factory", (url,))?;
            let data: Option<String> = result.extract()?;
            Ok(data)
        })
    }

    fn fetch_investing_com(&self, url: &str) -> PyResult<Option<String>> {
        Python::with_gil(|py| {
            let sc = PyModule::import(py, "seith_bridge.scraper")?;
            let result = sc.call_method1("fetch_investing_com", (url,))?;
            let data: Option<String> = result.extract()?;
            Ok(data)
        })
    }
}
