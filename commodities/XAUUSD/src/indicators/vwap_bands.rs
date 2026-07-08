// VWAP Deviation Bands — Session Volume Weighted Average Price ±2.5

#[derive(Debug, Clone)]
pub struct VwapResult {
    pub vwap: f64,
    pub deviation: f64,
    pub upper_band: f64,
    pub lower_band: f64,
}

const BAND_MULTIPLIER: f64 = 2.5;

pub fn calculate_vwap(prices: &[f64], volumes: &[f64]) -> Option<VwapResult> {
    if prices.is_empty() || prices.len() != volumes.len() {
        return None;
    }
    let vol_sum: f64 = volumes.iter().sum();
    if vol_sum == 0.0 {
        return None;
    }
    let vwap: f64 = prices.iter().zip(volumes).map(|(p, v)| p * v).sum::<f64>() / vol_sum;
    let variance: f64 = prices
        .iter()
        .zip(volumes)
        .map(|(p, v)| (p - vwap).powi(2) * v)
        .sum::<f64>()
        / vol_sum;
    let std_dev = variance.sqrt();
    Some(VwapResult {
        vwap,
        deviation: std_dev,
        upper_band: vwap + BAND_MULTIPLIER * std_dev,
        lower_band: vwap - BAND_MULTIPLIER * std_dev,
    })
}

pub fn is_overextended(price: f64, vwap: &VwapResult) -> bool {
    price > vwap.upper_band || price < vwap.lower_band
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_vwap_calculation() {
        let r = calculate_vwap(&[10.0, 11.0, 12.0], &[100.0, 200.0, 100.0]).unwrap();
        assert!((r.vwap - 11.0).abs() < 0.01);
    }

    #[test]
    fn test_no_volume() {
        assert!(calculate_vwap(&[10.0], &[0.0]).is_none());
    }

    #[test]
    fn test_overextended() {
        let r = calculate_vwap(&[10.0, 11.0, 12.0], &[100.0, 200.0, 100.0]).unwrap();
        assert!(is_overextended(100.0, &r));
        assert!(!is_overextended(11.0, &r));
    }
}
