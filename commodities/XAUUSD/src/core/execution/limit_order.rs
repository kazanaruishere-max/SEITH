// Limit Order — Sniper Mode
// BUY_LIMIT / SELL_LIMIT at Outer Liquidity Pool + Spread Buffer

use crate::core::l1_structure::signal_classifier::SignalTier;

#[derive(Debug, Clone)]
pub struct LimitOrderParams {
    pub entry_price: f64,
    pub stop_loss: f64,
    pub take_profit: f64,
    pub order_type: &'static str,
    pub spread_buffer: f64,
}

pub fn calculate_limit_order(
    current_price: f64,
    direction: &str,
    tier: &SignalTier,
    spread_pips: f64,
) -> LimitOrderParams {
    let spread_buffer = spread_pips * 2.0;
    let sl_buffer = match tier {
        SignalTier::Tier1Institutional => spread_pips * 5.0,
        SignalTier::Tier2Tactical => spread_pips * 3.0,
        SignalTier::NoSignal => spread_pips,
    };

    let (entry_price, sl_price, tp_price, order_type) = match direction {
        "BUY" => {
            let entry = current_price - spread_buffer;
            let sl = entry - sl_buffer * 2.0;
            let tp = entry
                + sl_buffer
                    * (if tier == &SignalTier::Tier1Institutional {
                        4.0
                    } else {
                        2.0
                    });
            (entry, sl, tp, "BUY_LIMIT")
        }
        "SELL" => {
            let entry = current_price + spread_buffer;
            let sl = entry + sl_buffer * 2.0;
            let tp = entry
                - sl_buffer
                    * (if tier == &SignalTier::Tier1Institutional {
                        4.0
                    } else {
                        2.0
                    });
            (entry, sl, tp, "SELL_LIMIT")
        }
        _ => panic!("Invalid direction"),
    };

    LimitOrderParams {
        entry_price,
        stop_loss: sl_price,
        take_profit: tp_price,
        order_type,
        spread_buffer,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::l1_structure::signal_classifier::SignalTier;

    #[test]
    fn test_buy_limit_entry_below_market() {
        let r = calculate_limit_order(100.0, "BUY", &SignalTier::Tier2Tactical, 0.5);
        assert!(r.entry_price < 100.0);
        assert_eq!(r.order_type, "BUY_LIMIT");
    }

    #[test]
    fn test_sell_limit_entry_above_market() {
        let r = calculate_limit_order(100.0, "SELL", &SignalTier::Tier2Tactical, 0.5);
        assert!(r.entry_price > 100.0);
    }

    #[test]
    fn test_tier1_wider_sl() {
        let t1 = calculate_limit_order(100.0, "BUY", &SignalTier::Tier1Institutional, 0.5);
        let t2 = calculate_limit_order(100.0, "BUY", &SignalTier::Tier2Tactical, 0.5);
        assert!((t1.stop_loss - 100.0).abs() > (t2.stop_loss - 100.0).abs());
    }
}
