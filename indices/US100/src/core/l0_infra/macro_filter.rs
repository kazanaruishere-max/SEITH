use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum MacroEvent {
    Fomc,
    FomcMinutes,
    Cpi,
    Nfp,
    Gdp,
    Ppi,
    IsmPmi,
    RetailSales,
    EarningsAapl,
    EarningsMsft,
    EarningsNvda,
    EarningsAmzn,
    EarningsGoogl,
    EarningsMeta,
}

impl MacroEvent {
    pub fn is_red(&self) -> bool {
        matches!(self, MacroEvent::Fomc | MacroEvent::FomcMinutes | MacroEvent::Cpi | MacroEvent::Nfp | MacroEvent::Gdp)
    }

    pub fn is_orange(&self) -> bool {
        matches!(self, MacroEvent::Ppi | MacroEvent::IsmPmi | MacroEvent::RetailSales)
    }

    pub fn is_earnings(&self) -> bool {
        matches!(self,
            MacroEvent::EarningsAapl | MacroEvent::EarningsMsft | MacroEvent::EarningsNvda
            | MacroEvent::EarningsAmzn | MacroEvent::EarningsGoogl | MacroEvent::EarningsMeta
        )
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum MacroVerdict {
    Red,
    Orange,
    Warning,
    Green,
}

impl MacroVerdict {
    pub fn lot_multiplier(&self) -> f64 {
        match self {
            MacroVerdict::Red => 0.0,
            MacroVerdict::Orange => 0.5,
            MacroVerdict::Warning => 0.5,
            MacroVerdict::Green => 1.0,
        }
    }
}

pub struct MacroFilter {
    pub today_events: Vec<MacroEvent>,
    pub verdict: MacroVerdict,
    pub no_trade_until: Option<DateTime<Utc>>,
}

impl MacroFilter {
    pub fn new() -> Self {
        Self {
            today_events: Vec::new(),
            verdict: MacroVerdict::Green,
            no_trade_until: None,
        }
    }

    pub fn evaluate(&mut self) -> MacroVerdict {
        if self.today_events.is_empty() {
            self.verdict = MacroVerdict::Green;
            return self.verdict;
        }
        if self.today_events.iter().any(|e| e.is_red()) {
            self.verdict = MacroVerdict::Red;
        } else if self.today_events.iter().any(|e| e.is_orange()) {
            self.verdict = MacroVerdict::Orange;
        } else if self.today_events.iter().any(|e| e.is_earnings()) {
            self.verdict = MacroVerdict::Warning;
        } else {
            self.verdict = MacroVerdict::Green;
        }
        if let Some(until) = self.no_trade_until {
            if Utc::now() < until && self.verdict == MacroVerdict::Red {
                return MacroVerdict::Red;
            }
        }
        self.verdict
    }

    pub fn set_no_trade_zone(&mut self, event: MacroEvent) {
        self.no_trade_until = Some(Utc::now() + chrono::Duration::hours(2));
        self.today_events.push(event);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty_calendar_green() {
        let mut filter = MacroFilter::new();
        assert_eq!(filter.evaluate(), MacroVerdict::Green);
    }

    #[test]
    fn test_red_event() {
        let mut filter = MacroFilter::new();
        filter.today_events.push(MacroEvent::Fomc);
        assert_eq!(filter.evaluate(), MacroVerdict::Red);
    }

    #[test]
    fn test_orange_event() {
        let mut filter = MacroFilter::new();
        filter.today_events.push(MacroEvent::Ppi);
        assert_eq!(filter.evaluate(), MacroVerdict::Orange);
    }

    #[test]
    fn test_earnings_warning() {
        let mut filter = MacroFilter::new();
        filter.today_events.push(MacroEvent::EarningsAapl);
        assert_eq!(filter.evaluate(), MacroVerdict::Warning);
    }

    #[test]
    fn test_red_overrides_orange() {
        let mut filter = MacroFilter::new();
        filter.today_events.push(MacroEvent::Ppi);
        filter.today_events.push(MacroEvent::Fomc);
        assert_eq!(filter.evaluate(), MacroVerdict::Red);
    }

    #[test]
    fn test_lot_multiplier() {
        assert!((MacroVerdict::Red.lot_multiplier() - 0.0).abs() < 1e-6);
        assert!((MacroVerdict::Orange.lot_multiplier() - 0.5).abs() < 1e-6);
        assert!((MacroVerdict::Warning.lot_multiplier() - 0.5).abs() < 1e-6);
        assert!((MacroVerdict::Green.lot_multiplier() - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_event_classification() {
        assert!(MacroEvent::Fomc.is_red());
        assert!(MacroEvent::Cpi.is_red());
        assert!(!MacroEvent::Ppi.is_red());
        assert!(MacroEvent::Ppi.is_orange());
        assert!(MacroEvent::EarningsNvda.is_earnings());
        assert!(!MacroEvent::Fomc.is_earnings());
    }
}
