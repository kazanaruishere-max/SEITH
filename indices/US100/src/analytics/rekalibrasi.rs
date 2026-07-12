use crate::config::settings;
use crate::config::thresholds;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum RecalibrationMode {
    Immediate,
    Batch(u32),
}

pub struct Rekalibrasi {
    pub scalp_trade_count: u32,
    pub buffer: Vec<f64>,
}

impl Rekalibrasi {
    pub fn new() -> Self {
        Self { scalp_trade_count: 0, buffer: Vec::new() }
    }

    pub fn determine_mode(slippage_pct: f64, mode: &str, scalp_count: u32) -> RecalibrationMode {
        if slippage_pct > settings::SLIPPAGE_FORCE_IMMEDIATE {
            log::warn!("[Rekalibrasi] Slippage {:.1}% > {}% — force immediate", slippage_pct, settings::SLIPPAGE_FORCE_IMMEDIATE);
            return RecalibrationMode::Immediate;
        }
        match mode {
            "SNIPER" => RecalibrationMode::Immediate,
            "SCALP" => {
                if scalp_count == 0 {
                    RecalibrationMode::Immediate
                } else if scalp_count % thresholds::SCALP_BATCH_SIZE == 0 {
                    RecalibrationMode::Immediate
                } else {
                    let remaining = thresholds::SCALP_BATCH_SIZE - (scalp_count % thresholds::SCALP_BATCH_SIZE);
                    RecalibrationMode::Batch(remaining)
                }
            }
            _ => RecalibrationMode::Immediate,
        }
    }

    pub fn record_scalp_trade(&mut self, slippage_pct: f64) -> RecalibrationMode {
        self.scalp_trade_count += 1;
        Self::determine_mode(slippage_pct, "SCALP", self.scalp_trade_count)
    }

    pub fn add_to_buffer(&mut self, value: f64) {
        self.buffer.push(value);
    }

    pub fn flush_buffer(&mut self) -> Vec<f64> {
        std::mem::take(&mut self.buffer)
    }

    pub fn reset_scalp_count(&mut self) {
        self.scalp_trade_count = 0;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sniper_immediate() {
        let mode = Rekalibrasi::determine_mode(0.1, "SNIPER", 0);
        assert_eq!(mode, RecalibrationMode::Immediate);
    }

    #[test]
    fn test_scalp_trade1_immediate() {
        let mode = Rekalibrasi::determine_mode(0.1, "SCALP", 0);
        assert_eq!(mode, RecalibrationMode::Immediate);
    }

    #[test]
    fn test_scalp_trade2_batch() {
        let mode = Rekalibrasi::determine_mode(0.1, "SCALP", 1);
        assert_eq!(mode, RecalibrationMode::Batch(1));
    }

    #[test]
    fn test_scalp_trade3_immediate_after_batch() {
        let mode = Rekalibrasi::determine_mode(0.1, "SCALP", 2);
        assert_eq!(mode, RecalibrationMode::Immediate);
    }

    #[test]
    fn test_slippage_darurat_force_immediate() {
        let mode = Rekalibrasi::determine_mode(0.6, "SCALP", 1);
        assert_eq!(mode, RecalibrationMode::Immediate);
    }

    #[test]
    fn test_record_scalp_count() {
        let mut r = Rekalibrasi::new();
        r.record_scalp_trade(0.1);
        assert_eq!(r.scalp_trade_count, 1);
        r.record_scalp_trade(0.1);
        assert_eq!(r.scalp_trade_count, 2);
    }

    #[test]
    fn test_flush_buffer() {
        let mut r = Rekalibrasi::new();
        r.add_to_buffer(1.0);
        r.add_to_buffer(2.0);
        let buf = r.flush_buffer();
        assert_eq!(buf.len(), 2);
        assert!(r.buffer.is_empty());
    }

    #[test]
    fn test_reset_scalp_count() {
        let mut r = Rekalibrasi::new();
        r.scalp_trade_count = 5;
        r.reset_scalp_count();
        assert_eq!(r.scalp_trade_count, 0);
    }
}
