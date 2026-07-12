# Fase 10: Backtesting & Parameter Optimization

## Goal
Backtesting framework, parameter sweep, walk-forward analysis, Monte Carlo simulation. Optimize threshold untuk target WR ≥ 80%, PF ≥ 4.0.

## Files Created
```
notebooks/us100_backtest.ipynb
notebooks/us100_parameter_optimization.ipynb
```

(Jupyter notebooks — Python, bukan Rust)

## PRD Reference

**Technology Stack (PRD §4.1):** Statistical Modeling — Jupyter Notebook untuk backtesting analysis, recalibration R&D.

**Target Metrics (PRD §1):**

| Metric | Target | Hard Limit |
|--------|--------|------------|
| Max DD | ≤ 6% | ≤ 10% |
| Win Rate | ≥ 80% | ≥ 70% |
| Profit Factor | ≥ 4.0 | ≥ 3.8 |
| Recovery Factor | ≥ 4.0 | ≥ 3.0 |
| Consecutive Loss | ≤ 3 | ≤ 4 (auto-halt ≥ 5) |

## Key Decisions
- **Data source:** Historical OHLCV dari OANDA via Python bridge.
- **Parameters to optimize:** HV Z-Score window, FRAMA period, OFS threshold, VWAP band width, RR ratio.
- **Walk-forward:** Train 3 bulan, test 1 bulan. Sliding window.
- **Monte Carlo:** 1000+ simulasi untuk confidence interval metrics.
- **Crisis mode backtest:** Validasi bahwa crisis ceiling prevent prolonged drawdown.
- **Batch vs immediate:** Validasi bahwa scalp batch 2 tidak degrade performance.

## Dependencies
- Phase 1-9 (full implementation siap untuk di-backtest)

## Acceptance Criteria
- Backtest hasil > 100 trades
- WR ≥ 80%, PF ≥ 4.0 tercapai di test set
- Max DD ≤ 6%
- Parameter optimal terdokumentasi
