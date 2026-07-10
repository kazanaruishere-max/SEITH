// Tick data structures and stream reader for tick-level backtest
// CSV columns: time_ms,bid,ask,bid_vol,ask_vol

use anyhow::{Context, Result};

#[derive(Debug, Clone, Copy)]
pub struct Tick {
    pub time_ms: i64,
    pub bid: f64,
    pub ask: f64,
    pub spread: f64,
    pub bid_vol: i64,
    pub ask_vol: i64,
}

impl Tick {
    pub fn mid(&self) -> f64 {
        (self.bid + self.ask) / 2.0
    }

    pub fn spread_pips(&self) -> f64 {
        (self.ask - self.bid).abs()
    }
}

/// Memory-mapped chunk of ticks for fast iteration
pub struct TickStream {
    ticks: Vec<Tick>,
    pos: usize,
}

impl TickStream {
    /// Load ticks from synthetic CSV
    pub fn from_csv(path: &str, max_ticks: Option<usize>) -> Result<Self> {
        let content =
            std::fs::read_to_string(path).with_context(|| format!("Cannot read: {}", path))?;

        let mut ticks = Vec::new();
        for line in content.lines().skip(1) {
            let line = line.trim();
            if line.is_empty() {
                continue;
            }
            let parts: Vec<&str> = line.split(',').collect();
            if parts.len() < 5 {
                continue;
            }
            let time_ms: i64 = parts[0].trim().parse()?;
            let bid: f64 = parts[1].trim().parse()?;
            let ask: f64 = parts[2].trim().parse()?;
            let bid_vol: i64 = parts[3].trim().parse()?;
            let ask_vol: i64 = parts[4].trim().parse()?;

            ticks.push(Tick {
                time_ms,
                bid,
                ask,
                spread: (ask - bid).abs(),
                bid_vol,
                ask_vol,
            });

            if let Some(max) = max_ticks {
                if ticks.len() >= max {
                    break;
                }
            }
        }

        if ticks.is_empty() {
            anyhow::bail!("No ticks loaded from {}", path);
        }

        Ok(Self { ticks, pos: 0 })
    }

    pub fn len(&self) -> usize {
        self.ticks.len()
    }

    pub fn is_empty(&self) -> bool {
        self.ticks.is_empty()
    }

    pub fn remaining(&self) -> usize {
        self.ticks.len() - self.pos
    }

    pub fn has_more(&self) -> bool {
        self.pos < self.ticks.len()
    }

    /// Get next tick, advancing position
    pub fn read(&mut self) -> Option<&Tick> {
        let t = self.ticks.get(self.pos)?;
        self.pos += 1;
        Some(t)
    }

    /// Peek at current position without advancing
    pub fn peek(&self) -> Option<&Tick> {
        self.ticks.get(self.pos)
    }

    /// Reset to beginning
    pub fn reset(&mut self) {
        self.pos = 0;
    }

    /// Get a window of ticks ending at current position
    pub fn window(&self, n: usize) -> &[Tick] {
        let start = self.pos.saturating_sub(n);
        &self.ticks[start..self.pos]
    }

    /// Get all ticks in range
    pub fn slice(&self, start: usize, end: usize) -> &[Tick] {
        let end = end.min(self.ticks.len());
        &self.ticks[start..end]
    }
}
