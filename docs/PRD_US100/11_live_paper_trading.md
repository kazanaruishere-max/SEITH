# Fase 11: Live Paper Trading & Monitoring

## Goal
Demo account running, live monitoring, parameter tuning berdasarkan real market condition.

## Files Created
```
scripts/monitor_us100.py      # Monitoring script (Python)
seith-bin/src/main.rs         # Final routing + graceful shutdown
```

## PRD Reference

**Architecture (PRD §2.1):** All layers integrated — L0 → L3 → L1 → Execution → Self-Learning.

**Binary Alias (PRD §6):** `seith US100`

**Workflow Pipeline (PRD §7):** Full pipeline dari boot → day filter → session → 5-gate → execution → self-learning → loop.

## Key Decisions
- **Paper trading:** OANDA demo account. Real market data, fake money.
- **Monitoring:** Track per trade: entry reason, exit reason, slippage, gate breakdown.
- **Tuning windows:** Rekalibrasi parameter setelah 10-20 trades pertama.
- **Gradual upgrade:** Dari demo → kecil (0.01 lot) → full. Hanya setelah 50+ trades dengan WR ≥ 70%.
- **Kill switch:** Manual override — `--no-trade` flag untuk monitoring only.
- **Graceful shutdown:** SIGINT handler — close posisi, cancel pending, save state, log.

## Dependencies
- Phase 10 (parameter optimal dari backtest)

## Acceptance Criteria
- `seith US100` running di OANDA demo
- 50+ trades di paper trading
- WR ≥ 70% (minimum threshold)
- DD tidak exceed 10%
- Siap upgrade ke live (0.01 lot)
