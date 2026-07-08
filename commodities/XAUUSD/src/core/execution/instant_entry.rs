// Instant Entry — Manual Mode
// Market order on rejection confirm: Body Ratio < 0.25 & Velocity >= 200

#[derive(Debug, Clone)]
pub struct InstantEntryParams {
    pub direction: &'static str,
    pub entry_price: f64,
    pub stop_loss: f64,
    pub take_profit: f64,
}

pub fn can_instant_entry(body_ratio: f64, velocity: f64) -> bool {
    body_ratio < 0.25 && velocity >= 200.0
}

pub fn calculate_instant_entry(
    current_price: f64,
    direction: &str,
    spread_pips: f64,
) -> InstantEntryParams {
    let sl_buffer = spread_pips * 5.0;
    match direction {
        "BUY" => {
            let entry = current_price;
            let sl = entry - sl_buffer;
            let tp = entry + sl_buffer * 2.0;
            InstantEntryParams {
                direction: "BUY",
                entry_price: entry,
                stop_loss: sl,
                take_profit: tp,
            }
        }
        "SELL" => {
            let entry = current_price;
            let sl = entry + sl_buffer;
            let tp = entry - sl_buffer * 2.0;
            InstantEntryParams {
                direction: "SELL",
                entry_price: entry,
                stop_loss: sl,
                take_profit: tp,
            }
        }
        _ => panic!("Invalid direction"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rejection_entry_valid() {
        assert!(can_instant_entry(0.15, 250.0));
    }

    #[test]
    fn test_rejection_blocked_low_velocity() {
        assert!(!can_instant_entry(0.15, 50.0));
    }

    #[test]
    fn test_rejection_blocked_large_body() {
        assert!(!can_instant_entry(0.50, 250.0));
    }

    #[test]
    fn test_buy_entry_at_market() {
        let r = calculate_instant_entry(100.0, "BUY", 0.5);
        assert!((r.entry_price - 100.0).abs() < 0.01);
        assert!(r.take_profit > r.entry_price);
    }
}
