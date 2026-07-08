// AMT Volume Profile — Auction Market Theory
// POC (Point of Control), VAH (Value Area High), VAL (Value Area Low)

#[derive(Debug, Clone)]
pub struct VolumeProfile {
    pub poc: f64,
    pub vah: f64,
    pub val: f64,
    pub value_area_volume: f64,
    pub total_volume: f64,
}

const VALUE_AREA_PERCENT: f64 = 0.70;

pub fn calculate_profile(prices: &[f64], volumes: &[f64]) -> Option<VolumeProfile> {
    if prices.is_empty() || prices.len() != volumes.len() {
        return None;
    }

    let total_volume: f64 = volumes.iter().sum();
    if total_volume == 0.0 {
        return None;
    }

    let tick_size = 0.1;
    let mut seen: std::collections::HashMap<i64, f64> = std::collections::HashMap::new();
    for (&p, &v) in prices.iter().zip(volumes) {
        let key = (p / tick_size).round() as i64;
        *seen.entry(key).or_insert(0.0) += v;
    }
    let mut sorted: Vec<_> = seen.into_iter().collect();
    sorted.sort_by_key(|(k, _)| *k);

    let poc_key = sorted
        .iter()
        .max_by(|a, b| a.1.partial_cmp(&b.1).unwrap())
        .map(|(k, _)| *k)?;
    let poc = poc_key as f64 * tick_size;

    sorted.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap());
    let target_vol = total_volume * VALUE_AREA_PERCENT;
    let mut cum_vol = 0.0;
    let mut vah = poc;
    let mut val = poc;

    for &(k, v) in sorted.iter().rev() {
        cum_vol += v;
        let price = k as f64 * tick_size;
        if price > vah {
            vah = price;
        }
        if price < val {
            val = price;
        }
        if cum_vol >= target_vol {
            break;
        }
    }

    Some(VolumeProfile {
        poc,
        vah,
        val,
        value_area_volume: cum_vol,
        total_volume,
    })
}

pub fn is_in_magnet_zone(price: f64, profile: &VolumeProfile, pips: f64) -> bool {
    (price - profile.poc).abs() <= pips
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_profile() {
        let prices = vec![10.0, 10.1, 10.2, 10.3, 10.4];
        let volumes = vec![100.0, 200.0, 500.0, 200.0, 100.0];
        let r = calculate_profile(&prices, &volumes).unwrap();
        assert!((r.poc - 10.2).abs() < 0.05);
    }

    #[test]
    fn test_magnet_zone() {
        let p = VolumeProfile {
            poc: 10.0,
            vah: 10.5,
            val: 9.5,
            value_area_volume: 700.0,
            total_volume: 1000.0,
        };
        assert!(is_in_magnet_zone(10.0, &p, 0.5));
        assert!(!is_in_magnet_zone(11.0, &p, 0.5));
    }

    #[test]
    fn test_empty_input() {
        assert!(calculate_profile(&[], &[]).is_none());
    }
}
