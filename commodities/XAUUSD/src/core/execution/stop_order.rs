// Stop Order — Momentum Rider Mode
// BUY_STOP / SELL_STOP outside M1 consolidation range

use crate::core::l1_structure::signal_classifier::SignalTier;

#[derive(Debug, Clone)]
pub struct StopOrderParams {
    pub entry_price: f64,
    pub stop_loss: f64,
    pub take_profit: f64,
    pub order_type: &'static str,
    pub breakout_pips: f64,
}

pub fn calculate_stop_order(
    consolidation_high: f64,
    consolidation_low: f64,
    direction: &str,
    tier: &SignalTier,
    range_pips: f64,
) -> StopOrderParams {
    let breakout_buffer = range_pips * 1.5;
    let sl_distance = match tier {
        SignalTier::Tier1Institutional => range_pips * 3.0,
        SignalTier::Tier2Tactical => range_pips * 2.0,
        SignalTier::NoSignal => range_pips,
    };

    match direction {
        "BUY" => {
            let entry = consolidation_high + breakout_buffer;
            let sl = consolidation_low - sl_distance;
            let tp = entry + sl_distance * 2.0;
            StopOrderParams {
                entry_price: entry,
                stop_loss: sl,
                take_profit: tp,
                order_type: "BUY_STOP",
                breakout_pips: breakout_buffer,
            }
        }
        "SELL" => {
            let entry = consolidation_low - breakout_buffer;
            let sl = consolidation_high + sl_distance;
            let tp = entry - sl_distance * 2.0;
            StopOrderParams {
                entry_price: entry,
                stop_loss: sl,
                take_profit: tp,
                order_type: "SELL_STOP",
                breakout_pips: breakout_buffer,
            }
        }
        _ => panic!("Invalid direction"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::l1_structure::signal_classifier::SignalTier;

    #[test]
    fn test_buy_stop_above_range() {
        let r = calculate_stop_order(101.0, 99.0, "BUY", &SignalTier::Tier2Tactical, 2.0);
        assert!(r.entry_price > 101.0);
        assert_eq!(r.order_type, "BUY_STOP");
    }

    #[test]
    fn test_sell_stop_below_range() {
        let r = calculate_stop_order(101.0, 99.0, "SELL", &SignalTier::Tier2Tactical, 2.0);
        assert!(r.entry_price < 99.0);
    }
}
