use crate::config::thresholds;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum AbsorptionSignal {
    None,
    BuyAbsorption,
    SellAbsorption,
}

pub struct SDom;

impl SDom {
    pub fn new() -> Self {
        Self
    }

    /// Hitung DOM imbalance Z-Score dari bid/ask volume
    /// Return nilai -1..+1:
    ///   +1 = bid dominance (buy pressure)
    ///   -1 = ask dominance (sell pressure)
    pub fn compute_imbalance(bid_volume: f64, ask_volume: f64) -> f64 {
        let total = bid_volume + ask_volume;
        if total < 1e-10 {
            return 0.0;
        }
        let ratio = (bid_volume - ask_volume) / total;
        ratio.clamp(-1.0, 1.0)
    }

    /// Deteksi absorpsi: limit order besar termakan tanpa pergerakan harga signifikan
    /// Parameter:
    ///   - price_change: perubahan harga dalam periode (points)
    ///   - bid_size: size bid limit order
    ///   - ask_size: size ask limit order
    ///   - threshold: minimal size untuk dianggap "besar" (default 2x rata-rata)
    pub fn detect_absorption(
        price_change: f64,
        bid_size: f64,
        ask_size: f64,
        avg_size: f64,
    ) -> AbsorptionSignal {
        let threshold = avg_size * 2.0;
        let price_moved = price_change.abs() > thresholds::SCALP_SL_MIN * 0.3;

        if bid_size > threshold && !price_moved {
            AbsorptionSignal::BuyAbsorption
        } else if ask_size > threshold && !price_moved {
            AbsorptionSignal::SellAbsorption
        } else {
            AbsorptionSignal::None
        }
    }

    /// S_DOM score: gabungan imbalance + absorption
    /// Range -1..+1 (0 jika no signal)
    pub fn compute_score(
        bid_volume: f64,
        ask_volume: f64,
        price_change: f64,
        bid_size: f64,
        ask_size: f64,
        avg_size: f64,
    ) -> f64 {
        let imbalance = Self::compute_imbalance(bid_volume, ask_volume);
        let absorption = Self::detect_absorption(price_change, bid_size, ask_size, avg_size);
        let abs_score = match absorption {
            AbsorptionSignal::BuyAbsorption => 0.5,
            AbsorptionSignal::SellAbsorption => -0.5,
            AbsorptionSignal::None => 0.0,
        };
        (imbalance + abs_score).clamp(-1.0, 1.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_imbalance_bid_dominant() {
        let s = SDom::compute_imbalance(100.0, 30.0);
        assert!(s > 0.0);
    }

    #[test]
    fn test_imbalance_ask_dominant() {
        let s = SDom::compute_imbalance(20.0, 80.0);
        assert!(s < 0.0);
    }

    #[test]
    fn test_imbalance_equal() {
        let s = SDom::compute_imbalance(50.0, 50.0);
        assert!((s - 0.0).abs() < 1e-6);
    }

    #[test]
    fn test_imbalance_zero() {
        let s = SDom::compute_imbalance(0.0, 0.0);
        assert!((s - 0.0).abs() < 1e-6);
    }

    #[test]
    fn test_absorption_buy_detected() {
        let a = SDom::detect_absorption(0.1, 1000.0, 100.0, 100.0);
        assert_eq!(a, AbsorptionSignal::BuyAbsorption);
    }

    #[test]
    fn test_absorption_sell_detected() {
        let a = SDom::detect_absorption(0.1, 100.0, 1000.0, 100.0);
        assert_eq!(a, AbsorptionSignal::SellAbsorption);
    }

    #[test]
    fn test_absorption_none_when_price_moves() {
        let a = SDom::detect_absorption(5.0, 1000.0, 100.0, 100.0);
        assert_eq!(a, AbsorptionSignal::None);
    }

    #[test]
    fn test_absorption_none_when_small() {
        let a = SDom::detect_absorption(0.1, 150.0, 100.0, 100.0);
        assert_eq!(a, AbsorptionSignal::None);
    }

    #[test]
    fn test_compute_score_buy() {
        let s = SDom::compute_score(100.0, 30.0, 0.1, 1000.0, 50.0, 100.0);
        assert!(s > 0.0);
    }

    #[test]
    fn test_compute_score_sell() {
        let s = SDom::compute_score(20.0, 80.0, 0.1, 50.0, 1000.0, 100.0);
        assert!(s < 0.0);
    }

    #[test]
    fn test_compute_score_no_signal() {
        let s = SDom::compute_score(50.0, 50.0, 0.0, 0.0, 0.0, 100.0);
        assert!((s - 0.0).abs() < 1e-6);
    }
}
