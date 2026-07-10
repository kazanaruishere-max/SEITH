# AI SEITH — Tick-Level Backtest Engine Plan (Fase 1-5)

## Status Update

| Phase | Status | Detail |
|-------|--------|--------|
| **Fase 1** | ✅ | Dukascopy data accessible via Cloudflare DNS override. 2.88M ticks downloaded. |
| **Fase 2** | ✅ | Tick engine running on real data: 61 trades, 31.1% WR, PF 0.68 |
| **Fase 3** | ✅ | 3 detectors implemented: absorption, exhaustion, CVD divergence |
| **Fase 4** | ⬜ | Adaptive SL/TP kalibrasi |
| **Fase 5** | ⬜ | Session filter + final tuning |

## Files

| File | Purpose |
|------|---------|
| `download_ticks_dukascopy.py` | Download ticks via Cloudflare DNS bypass |
| `tick_data.rs` | Tick struct + CSV stream reader |
| `tick_engine.rs` | Engine + 3 pattern detectors |
| `seith_ticktest.rs` | Binary runner |

## Usage

```bash
# Download real ticks (run once, ~10 min for ~3M ticks)
python jupyter/download_ticks_dukascopy.py

# Run tick backtest
cargo run -p xauusd --bin seith-ticktest
```
