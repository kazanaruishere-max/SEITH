// Signal Types — Enum: BuySignal, SellSignal, NoSignal

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Signal {
    Buy,
    Sell,
    NoSignal,
}

impl Signal {
    pub fn is_valid(&self) -> bool {
        !matches!(self, Signal::NoSignal)
    }
}
