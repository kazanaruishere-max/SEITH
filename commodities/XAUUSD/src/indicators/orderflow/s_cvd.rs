// S_CVD — Cumulative Volume Delta Divergence

#[derive(Debug, Clone)]
pub struct CvdResult {
    pub cvd: f64,
    pub divergence: f64,
    pub has_divergence: bool,
}

pub fn calculate_cvd(deltas: &[f64]) -> CvdResult {
    let cvd: f64 = deltas.iter().sum();
    let n = deltas.len();
    if n < 2 {
        return CvdResult {
            cvd,
            divergence: 0.0,
            has_divergence: false,
        };
    }
    let first_half: f64 = deltas[..n / 2].iter().sum();
    let second_half: f64 = deltas[n / 2..].iter().sum();
    let divergence = second_half - first_half;
    CvdResult {
        cvd,
        divergence,
        has_divergence: divergence.abs() > cvd.abs() * 0.3,
    }
}

pub fn cvd_score(cvd: &CvdResult) -> i32 {
    if !cvd.has_divergence {
        return 0;
    }
    if cvd.divergence > 0.0 {
        1
    } else {
        -1
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cvd_accumulation() {
        let r = calculate_cvd(&[10.0, -5.0, 8.0]);
        assert!((r.cvd - 13.0).abs() < 0.01);
    }

    #[test]
    fn test_divergence_detected() {
        let r = calculate_cvd(&[1.0, 1.0, 10.0, 10.0]);
        assert!(r.has_divergence);
    }
}
