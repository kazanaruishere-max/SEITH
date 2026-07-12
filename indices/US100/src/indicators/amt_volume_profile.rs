#[derive(Debug, Clone)]
pub struct AmtVolumeProfile {
    pub poc: f64,
    pub vah: f64,
    pub val: f64,
    pub magnet_zone_low: f64,
    pub magnet_zone_high: f64,
}

#[derive(Debug, Clone, Copy)]
pub struct VolumeBar {
    pub price: f64,
    pub volume: f64,
}

impl AmtVolumeProfile {
    pub fn new() -> Self {
        Self {
            poc: 0.0,
            vah: 0.0,
            val: 0.0,
            magnet_zone_low: 0.0,
            magnet_zone_high: 0.0,
        }
    }

    pub fn compute(&mut self, bars: &[VolumeBar]) {
        if bars.is_empty() {
            return;
        }
        let total_volume: f64 = bars.iter().map(|b| b.volume).sum();
        if total_volume < 1e-10 {
            return;
        }
        let mut sorted = bars.to_vec();
        sorted.sort_by(|a, b| a.price.partial_cmp(&b.price).unwrap());
        let poc_idx = sorted.iter()
            .enumerate()
            .max_by(|(_, a), (_, b)| a.volume.partial_cmp(&b.volume).unwrap())
            .map(|(i, _)| i)
            .unwrap_or(0);
        self.poc = sorted[poc_idx].price;

        let target_vol = total_volume * 0.70;
        let mut cum_vol = sorted[poc_idx].volume;
        let mut vah = self.poc;
        let mut val = self.poc;
        let mut up = poc_idx + 1;
        let mut down = poc_idx.wrapping_sub(1);

        while cum_vol < target_vol {
            let up_remaining = up < sorted.len();
            let down_remaining = down < sorted.len();
            if !up_remaining && !down_remaining {
                break;
            }
            if up_remaining && (!down_remaining || sorted[up].volume >= sorted[down].volume) {
                cum_vol += sorted[up].volume;
                vah = sorted[up].price;
                up += 1;
            } else if down_remaining {
                cum_vol += sorted[down].volume;
                val = sorted[down].price;
                down = down.wrapping_sub(1);
            }
        }

        self.vah = vah;
        self.val = val;
        let magnet_range = (self.vah - self.val).abs() * 0.1;
        self.magnet_zone_low = self.poc - magnet_range;
        self.magnet_zone_high = self.poc + magnet_range;
    }

    pub fn is_in_magnet_zone(&self, price: f64) -> bool {
        price >= self.magnet_zone_low && price <= self.magnet_zone_high
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_bars() -> Vec<VolumeBar> {
        vec![
            VolumeBar { price: 100.0, volume: 50.0 },
            VolumeBar { price: 101.0, volume: 150.0 },
            VolumeBar { price: 102.0, volume: 400.0 },
            VolumeBar { price: 103.0, volume: 300.0 },
            VolumeBar { price: 104.0, volume: 100.0 },
        ]
    }

    #[test]
    fn test_compute_empty() {
        let mut amt = AmtVolumeProfile::new();
        amt.compute(&[]);
        assert!((amt.poc - 0.0).abs() < 1e-6);
    }

    #[test]
    fn test_poc_at_highest_volume() {
        let mut amt = AmtVolumeProfile::new();
        amt.compute(&sample_bars());
        assert!((amt.poc - 102.0).abs() < 1e-6);
    }

    #[test]
    fn test_vah_val_non_zero() {
        let mut amt = AmtVolumeProfile::new();
        amt.compute(&sample_bars());
        assert!(amt.vah >= amt.poc);
        assert!(amt.val <= amt.poc);
    }

    #[test]
    fn test_magnet_zone_contains_poc() {
        let mut amt = AmtVolumeProfile::new();
        amt.compute(&sample_bars());
        assert!(amt.is_in_magnet_zone(amt.poc));
    }

    #[test]
    fn test_magnet_zone_range() {
        let mut amt = AmtVolumeProfile::new();
        amt.compute(&sample_bars());
        assert!(amt.magnet_zone_low < amt.poc);
        assert!(amt.magnet_zone_high > amt.poc);
    }

    #[test]
    fn test_price_far_from_poc_not_in_zone() {
        let mut amt = AmtVolumeProfile::new();
        amt.compute(&sample_bars());
        assert!(!amt.is_in_magnet_zone(999.0));
    }
}
