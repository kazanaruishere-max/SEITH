#[derive(Debug, Clone)]
pub struct VwapResult {
    pub vwap: f64,
    pub upper_band: f64,
    pub lower_band: f64,
    pub band_width: f64,
}

pub struct VwapBands;

impl VwapBands {
    pub fn compute(prices: &[f64], volumes: &[f64], band_width: f64) -> Option<VwapResult> {
        if prices.is_empty() || prices.len() != volumes.len() {
            return None;
        }
        let total_vol: f64 = volumes.iter().sum();
        if total_vol < 1e-10 {
            return None;
        }
        let vwap: f64 = prices.iter()
            .zip(volumes.iter())
            .map(|(p, v)| p * v)
            .sum::<f64>() / total_vol;
        let variance: f64 = prices.iter()
            .map(|p| (p - vwap).powi(2))
            .sum::<f64>() / prices.len() as f64;
        let std_dev = variance.sqrt();
        Some(VwapResult {
            vwap,
            upper_band: vwap + band_width * std_dev,
            lower_band: vwap - band_width * std_dev,
            band_width,
        })
    }

    pub fn is_within_bands(price: f64, bands: &VwapResult) -> bool {
        price >= bands.lower_band && price <= bands.upper_band
    }

    pub fn is_overextended(price: f64, bands: &VwapResult) -> bool {
        price > bands.upper_band || price < bands.lower_band
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compute_empty() {
        assert!(VwapBands::compute(&[], &[], 2.5).is_none());
    }

    #[test]
    fn test_compute_mismatched_lengths() {
        assert!(VwapBands::compute(&[100.0], &[100.0, 200.0], 2.5).is_none());
    }

    #[test]
    fn test_compute_single_bar() {
        let r = VwapBands::compute(&[100.0], &[1000.0], 2.5).unwrap();
        assert!((r.vwap - 100.0).abs() < 1e-6);
    }

    #[test]
    fn test_compute_vwap() {
        let prices = vec![100.0, 102.0, 101.0];
        let volumes = vec![1000.0, 500.0, 800.0];
        let r = VwapBands::compute(&prices, &volumes, 2.5).unwrap();
        let expected_vwap = (100.0*1000.0 + 102.0*500.0 + 101.0*800.0) / 2300.0;
        assert!((r.vwap - expected_vwap).abs() < 1e-6);
    }

    #[test]
    fn test_bands_ordered() {
        let r = VwapBands::compute(&[100.0, 101.0, 99.0], &[100.0, 200.0, 150.0], 2.5).unwrap();
        assert!(r.lower_band < r.vwap);
        assert!(r.upper_band > r.vwap);
    }

    #[test]
    fn test_is_within_bands() {
        let r = VwapBands::compute(&[100.0, 101.0, 99.0, 100.5], &[100.0, 200.0, 150.0, 180.0], 2.5).unwrap();
        assert!(VwapBands::is_within_bands(r.vwap, &r));
        assert!(!VwapBands::is_overextended(r.vwap, &r));
    }

    #[test]
    fn test_is_overextended() {
        let r = VwapBands::compute(&[100.0, 101.0], &[100.0, 100.0], 1.0).unwrap();
        let far_price = r.vwap + r.upper_band * 2.0;
        assert!(VwapBands::is_overextended(far_price, &r));
    }

    #[test]
    fn test_band_width_parameter() {
        let prices = vec![100.0, 102.0, 98.0];
        let volumes = vec![100.0, 100.0, 100.0];
        let tight = VwapBands::compute(&prices, &volumes, 1.0).unwrap();
        let loose = VwapBands::compute(&prices, &volumes, 3.0).unwrap();
        assert!(loose.upper_band > tight.upper_band);
    }
}
