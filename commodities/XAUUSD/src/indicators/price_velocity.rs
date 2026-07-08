// Price Velocity — Points Per Second Momentum

pub fn calculate_price_velocity(price_change: f64, time_secs: f64) -> f64 {
    if time_secs <= 0.0 {
        return 0.0;
    }
    (price_change / time_secs).abs()
}

pub fn is_high_velocity(velocity: f64) -> bool {
    velocity >= 250.0
}

pub fn is_low_velocity(velocity: f64) -> bool {
    velocity < 50.0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_normal_velocity() {
        let v = calculate_price_velocity(5.0, 0.02);
        assert!((v - 250.0).abs() < 0.01);
    }

    #[test]
    fn test_high_velocity_detected() {
        assert!(is_high_velocity(300.0));
        assert!(!is_high_velocity(100.0));
    }

    #[test]
    fn test_low_velocity_detected() {
        assert!(is_low_velocity(30.0));
        assert!(!is_low_velocity(100.0));
    }

    #[test]
    fn test_zero_time() {
        assert!((calculate_price_velocity(10.0, 0.0) - 0.0).abs() < 0.01);
    }
}
