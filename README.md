# AI SEITH

**Autonomous Multi-Instrument Trading Intelligence System**

Broker: Exness (XAUUSDm, Cent account)  
Language: Rust (core) + Python (bridge) + Jupyter (stats)  
Interface: Terminal CLI only — **no web, no GUI, no TUI**

---

## Quick Start

```bash
# 1. Setup environment
cp .env.example .env
# Edit .env — fill MT5 credentials & Telegram token

# 2. Build & run
cargo build
cargo run -- XAUUSD
```

## Instrument Support

| Status | Instrument | Type |
|--------|-----------|------|
| ✅ Full pipeline | XAUUSD | Gold — Exness |
| 🔲 Placeholder | EURUSD, GBPUSD, USDJPY, USDCHF, USDCAD, AUDUSD | Majors |
| 🔲 Placeholder | BTCUSD, ETHUSD | Crypto |

## Architecture

```
L0: Infrastructure — Data Feed, Normalizer, Jam Hantu
L3: Master Control — Event Loop, State Manager, Statistical Brain, Anti-Paralysis
L2: News Sniper — Red Folder Detector, Fast Poller, Net_Dev Calculator
L1: 4 Filters — Bayesian, CVaR, Market Compass, Orderflow
Execution — Limit Order, Stop Order, Instant Entry
Self-Learning — Trade Journal, Rekalibrasi, Auto-Kill
```

See `PRD_AI_SEITH.md` for complete documentation.
