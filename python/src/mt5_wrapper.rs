// MT5 API wrapper — PyO3 bridge
// Wraps python/seith_bridge/mt5.py functions

#![allow(non_local_definitions)]

use pyo3::prelude::*;

#[pyclass]
pub struct Mt5Wrapper;

#[pymethods]
impl Mt5Wrapper {
    #[new]
    fn new() -> Self {
        Self
    }

    fn init(&self, path: &str) -> PyResult<bool> {
        Python::with_gil(|py| {
            let mt5 = PyModule::import(py, "seith_bridge.mt5")?;
            mt5.call_method1("init_mt5", (path,))?.extract::<bool>()
        })
    }

    fn login(&self, account: i64, password: &str, server: &str) -> PyResult<bool> {
        Python::with_gil(|py| {
            let mt5 = PyModule::import(py, "seith_bridge.mt5")?;
            mt5.call_method1("login", (account, password, server))?
                .extract::<bool>()
        })
    }

    fn get_price(&self, symbol: &str) -> PyResult<Option<f64>> {
        Python::with_gil(|py| {
            let mt5 = PyModule::import(py, "seith_bridge.mt5")?;
            mt5.call_method1("get_price", (symbol,))?
                .extract::<Option<f64>>()
        })
    }

    fn place_order(
        &self,
        symbol: &str,
        order_type: i32,
        volume: f64,
        price: f64,
        sl: f64,
        tp: f64,
    ) -> PyResult<Option<i64>> {
        Python::with_gil(|py| {
            let mt5 = PyModule::import(py, "seith_bridge.mt5")?;
            mt5.call_method1(
                "place_order",
                (symbol, order_type, volume, price, sl, tp, ""),
            )?
            .extract::<Option<i64>>()
        })
    }

    fn shutdown(&self) -> PyResult<()> {
        Python::with_gil(|py| {
            let mt5 = PyModule::import(py, "seith_bridge.mt5")?;
            mt5.call_method0("shutdown")?;
            Ok(())
        })
    }
}
