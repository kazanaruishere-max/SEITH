// Order Manager — Order Lifecycle & Validation
// Routes to limit/stop execution (HARAM market order/Instant Entry)

use crate::core::execution::limit_order::{calculate_limit_order, LimitOrderParams};
use crate::core::execution::stop_order::{calculate_stop_order, StopOrderParams};
use crate::core::l1_structure::signal_classifier::SignalTier;

#[derive(Debug, Clone)]
pub enum ExecutionPlan {
    Limit(LimitOrderParams),
    Stop(StopOrderParams),
    None,
}

pub fn plan_execution(
    tier: &SignalTier,
    direction: &str,
    current_price: f64,
    consolidate: (f64, f64),
    spread_pips: f64,
    body_ratio_val: f64,
    velocity: f64,
) -> ExecutionPlan {
    match tier {
        SignalTier::NoSignal => ExecutionPlan::None,
        SignalTier::Tier1Institutional | SignalTier::Tier2Tactical => {
            let (high, low) = consolidate;
            let range_pips = (high - low).abs();
            if crate::core::execution::instant_entry::can_instant_entry(body_ratio_val, velocity) {
                ExecutionPlan::None // HARAM market order — skip demi PF 4.0
            } else if range_pips > spread_pips * 3.0 {
                ExecutionPlan::Limit(calculate_limit_order(
                    current_price,
                    direction,
                    tier,
                    spread_pips,
                ))
            } else {
                ExecutionPlan::Stop(calculate_stop_order(high, low, direction, tier, range_pips))
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn exec(tier: &SignalTier, high: f64, low: f64, br: f64, vel: f64) -> ExecutionPlan {
        plan_execution(tier, "BUY", 100.0, (high, low), 0.5, br, vel)
    }

    #[test]
    fn test_no_signal() {
        let r = exec(&SignalTier::NoSignal, 101.0, 99.0, 0.5, 100.0);
        assert!(matches!(r, ExecutionPlan::None));
    }

    #[test]
    fn test_instant_now_none() {
        // Instant Entry sekarang HARAM → harus None
        let r = exec(&SignalTier::Tier2Tactical, 101.0, 99.0, 0.15, 250.0);
        assert!(matches!(r, ExecutionPlan::None));
    }

    #[test]
    fn test_limit_on_wide_range() {
        let r = exec(&SignalTier::Tier2Tactical, 105.0, 95.0, 0.5, 100.0);
        assert!(matches!(r, ExecutionPlan::Limit(_)));
    }

    #[test]
    fn test_stop_on_narrow_range() {
        let r = exec(&SignalTier::Tier2Tactical, 100.5, 99.5, 0.5, 100.0);
        assert!(matches!(r, ExecutionPlan::Stop(_)));
    }
}
