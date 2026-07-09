// FRAMA — Flexible Moving Average
// Trend Rider M15: fractal dimension-based adaptive MA

const FRAMA_PERIOD: usize = 16;

#[derive(Debug, Clone)]
pub struct Frama {
    pub value: f64,
    pub deviation: f64,
}

pub fn calculate_frama(prices: &[f64]) -> Option<Frama> {
    if prices.len() <= FRAMA_PERIOD * 2 {
        return None;
    }
    let half = FRAMA_PERIOD;
    let n3 = half / 2;

    let h1 = range_sum(&prices[0..half], n3);
    let h2 = range_sum(&prices[half..half * 2], n3);
    let h3 = range_sum(prices, half * 2);

    let d = (h1 + h2).ln() - h3.ln();
    let alpha = (-4.0 * d).exp();
    let alpha = alpha.clamp(0.01, 1.0);

    let mut frama = prices[0];
    for &p in &prices[1..=half * 2] {
        frama = alpha * p + (1.0 - alpha) * frama;
    }

    let deviation = prices[half * 2] - frama;
    Some(Frama {
        value: frama,
        deviation,
    })
}

pub fn z_score_frama(price: f64, frama: &Frama) -> f64 {
    if frama.value == 0.0 {
        return 0.0;
    }
    (price - frama.value) / frama.value.abs().max(0.001) * 10.0
}

fn range_sum(prices: &[f64], period: usize) -> f64 {
    if prices.len() < period {
        return 0.0;
    }
    let mut sum = 0.0;
    for i in 1..period {
        sum += (prices[i] - prices[i - 1]).abs().ln().max(-10.0);
    }
    sum / period as f64
}

#[cfg(test)]
mod tests {
    use super::*;

    fn gen_prices(n: usize) -> Vec<f64> {
        (0..n)
            .map(|i| 2300.0 + (i as f64 * 0.5).sin() * 10.0)
            .collect()
    }

    #[test]
    fn test_frama_short_input() {
        assert!(calculate_frama(&[1.0; 10]).is_none());
    }

    #[test]
    fn test_frama_returns_value() {
        let r = calculate_frama(&gen_prices(40));
        assert!(r.is_some());
        assert!(r.unwrap().value > 0.0);
    }

    #[test]
    fn test_z_score_zero() {
        let f = Frama {
            value: 0.0,
            deviation: 0.0,
        };
        assert!((z_score_frama(100.0, &f) - 0.0).abs() < 0.01);
    }
}
