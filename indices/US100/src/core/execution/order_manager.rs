use crate::core::execution::limit_order::{LimitOrder, LimitOrderParams};
use crate::core::execution::market_entry::{MarketEntry, MarketOrderParams};
use crate::core::execution::risk_manager::{RiskLimitViolation, RiskManager};
use crate::core::execution::stop_order::{StopOrder, StopOrderParams};
use anyhow::Result;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum OrderType {
    Stop,
    Limit,
    Market,
}

#[derive(Debug, Clone)]
pub enum OrderResult {
    Success(String),
    Failed(String),
    PhantomDetected(String),
}

const MAX_RETRIES: u32 = 1;

pub struct OrderManager;

impl OrderManager {
    pub fn new() -> Self {
        Self
    }

    pub async fn place_stop(params: &StopOrderParams) -> OrderResult {
        for attempt in 1..=MAX_RETRIES {
            match StopOrder::place(&StopOrder, params).await {
                Ok(()) => return OrderResult::Success(format!("stop_{}_{}", params.side, attempt)),
                Err(e) => {
                    log::warn!("[OrderMgr] Stop attempt {}/{} failed: {}", attempt, MAX_RETRIES, e);
                    if attempt < MAX_RETRIES {
                        tokio::time::sleep(std::time::Duration::from_secs(1)).await;
                    }
                }
            }
        }
        OrderResult::Failed("Stop order failed after retry".into())
    }

    pub async fn place_limit(params: &LimitOrderParams) -> OrderResult {
        for attempt in 1..=MAX_RETRIES {
            match LimitOrder::place(&LimitOrder, params).await {
                Ok(()) => return OrderResult::Success(format!("limit_{}_{}", params.side, attempt)),
                Err(e) => {
                    log::warn!("[OrderMgr] Limit attempt {}/{} failed: {}", attempt, MAX_RETRIES, e);
                    if attempt < MAX_RETRIES {
                        tokio::time::sleep(std::time::Duration::from_secs(1)).await;
                    }
                }
            }
        }
        OrderResult::Failed("Limit order failed after retry".into())
    }

    pub async fn place_market(params: &MarketOrderParams) -> OrderResult {
        for attempt in 1..=MAX_RETRIES {
            match MarketEntry::execute(&MarketEntry, params).await {
                Ok(()) => return OrderResult::Success(format!("market_{}_{}", params.side, attempt)),
                Err(e) => {
                    log::warn!("[OrderMgr] Market attempt {}/{} failed: {}", attempt, MAX_RETRIES, e);
                    if attempt < MAX_RETRIES {
                        tokio::time::sleep(std::time::Duration::from_secs(1)).await;
                    }
                }
            }
        }
        OrderResult::Failed("Market order failed after retry".into())
    }

    pub fn should_skip_recalibration(result: &OrderResult) -> bool {
        matches!(result, OrderResult::Failed(_)) || matches!(result, OrderResult::PhantomDetected(_))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_place_stop_order() {
        let params = StopOrderParams { side: "buy".into(), entry: 102.0, sl: 99.0, tp: 106.0, lot: 1.0 };
        let result = OrderManager::place_stop(&params).await;
        assert!(matches!(result, OrderResult::Success(_)));
    }

    #[tokio::test]
    async fn test_place_limit_order() {
        let params = LimitOrderParams { side: "sell".into(), limit_price: 101.0, sl: 103.0, tp: 99.0, lot: 1.0 };
        let result = OrderManager::place_limit(&params).await;
        assert!(matches!(result, OrderResult::Success(_)));
    }

    #[tokio::test]
    async fn test_place_market_order() {
        let params = MarketOrderParams { side: "buy".into(), sl: 99.0, tp: 105.0, lot: 0.5 };
        let result = OrderManager::place_market(&params).await;
        assert!(matches!(result, OrderResult::Success(_)));
    }

    #[test]
    fn test_should_skip_recalibration() {
        assert!(OrderManager::should_skip_recalibration(&OrderResult::Failed("err".into())));
        assert!(OrderManager::should_skip_recalibration(&OrderResult::PhantomDetected("phantom".into())));
        assert!(!OrderManager::should_skip_recalibration(&OrderResult::Success("ok".into())));
    }
}
