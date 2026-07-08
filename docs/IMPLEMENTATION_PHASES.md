# IMPLEMENTATION PHASES — AI SEITH

**Dokumen ini adalah breakdown implementasi bertahap berdasarkan PRD_AI_SEITH.md.**
Setiap fase memiliki deliverable spesifik: `cargo fmt --check`, `cargo clippy -- -D warnings`, `cargo build`, dan `cargo test` harus pass.

---

## ✅ FASE 0 — Pondasi & Kerangka (COMPLETED)

| Area | Detail |
|------|--------|
| **Workspace** | 12 crate: `seith-bin`, `shared`, `python`, `XAUUSD`, 6 majors, 2 crypto |
| **Shared crate** | config, external bridge stubs, utils, data, analytics |
| **XAUUSD crate** | Full folder structure L0–L3, execution, indicators (stub) |
| **Placeholders** | EURUSD, GBPUSD, USDJPY, USDCHF, USDCAD, AUDUSD, BTCUSD, ETHUSD |
| **Python bridge** | PyO3 crate + Python stubs (MT5, Telegram, scraper) |
| **CI/CD** | ci.yml, release.yml, docker.yml (blueprint) |
| **Git hooks** | pre-commit (fmt+clippy), pre-push (test) |
| **Config** | .env.example, .gitignore, rust-toolchain.toml, README.md |

**GitHub:** https://github.com/kazanaruishere-max/SEITH

---

## 🔜 FASE 1 — L0 Infrastructure (Data Layer)

### Target PRD
Implementasi Level 0: Data Feed, Normalizer, Jam Hantu, SQLite migrations.

### Files
| File | Implementasi |
|------|-------------|
| `shared/data/sqlite_manager.rs` | sqlx pool + migrations |
| `shared/data/migrations/v001_init.sql` | Trades, State, Config tables |
| `commodities/XAUUSD/core/l0_infra/data_feed.rs` | MT5 price streaming via PyO3 |
| `commodities/XAUUSD/core/l0_infra/normalizer.rs` | 3-digit desimal multiplier 0.010 |
| `commodities/XAUUSD/core/l0_infra/jam_hantu.rs` | 20:45 force close trigger |

### Deliverable
```
seith XAUUSD
  → MT5 connected
  → Price normalized
  → Jam Hantu: Standby
```

---

## 🔜 FASE 2 — L3 Master Control (Event Loop)

### Target PRD
Implementasi Level 3: Event loop, State Manager, Statistical Brain, Anti-Paralysis.

### Files
| File | Implementasi |
|------|-------------|
| `l3_engine/event_loop.rs` | Tokio async loop per M1/M15 tick |
| `l3_engine/state_manager.rs` | NORMAL_MODE, NEWS_MODE, CRISIS_ADAPTATION |
| `l3_engine/statistical_brain.rs` | Volatility, slippage, spread analysis |
| `l3_engine/anti_paralysis.rs` | skip_strike_count ≥ 3 → threshold relaxation |

### Data Flow
```
Tick → L0 (validasi) → L3 (baca skip_count) → L2 atau L1
```

---

## 🔜 FASE 3 — Python Bridge Implementation

### Target PRD
Implementasi bridge Python via PyO3 untuk MT5, Telegram, scraper.

### Files
| File | Implementasi |
|------|-------------|
| `python/python/seith_bridge/mt5.py` | MetaTrader5 wrapper |
| `python/python/seith_bridge/telegram.py` | python-telegram-bot wrapper |
| `python/python/seith_bridge/scraper.py` | BeautifulSoup + Playwright |
| `python/src/mt5_wrapper.rs` | FFI → Rust |
| `python/src/telegram_wrapper.rs` | FFI → Rust |
| `python/src/scraper_wrapper.rs` | FFI → Rust |

### Requirements
- Python 3.10+
- `pip install MetaTrader5 python-telegram-bot beautifulsoup4 playwright requests`

---

## 🔜 FASE 4 — L2 News Sniper Engine

### Target PRD
Implementasi Level 2: Red Folder detection, Fast Polling, Net_Dev Calculation.

### Workflow (dari `WORKFLOW XAUUSD.mmd`)
```
News check → T-30 to T-60 min? → YES → NEWS_ANOMALY_MODE
  → Disable Level 1 → T-1 min fast polling
  → Dual scrape (ForexFactory + Investing.com)
  → Extract Actual, Forecast, Revised
  → Net_Dev = (Actual - Forecast) + (Revised_Previous - Original_Previous)
  → |Net_Dev| ≥ 2.0 → VALID → lanjut eksekusi
  → |Net_Dev| < 2.0 → BLOCK → skip
```

### Files
| File | Threshold |
|------|-----------|
| `l2_news/red_folder_detector.rs` | Impact = RED, Currency = USD |
| `l2_news/regime_switch.rs` | NORMAL ↔ NEWS |
| `l2_news/fast_poller.rs` | Async 100ms polling |
| `l2_news/data_extractor.rs` | Actual, Forecast, Previous |
| `l2_news/revision_handler.rs` | Revised vs Original |
| `l2_news/net_dev_calculator.rs` | `\|Net_Dev\| ≥ 2.0` |

---

## 🔜 FASE 5 — L1 4 Filter Pipeline

### Target PRD
Implementasi 4 filter bertingkat sesuai workflow.

### Filter 1 — Bayesian Gatekeeper
```
P(A|B) = (P(B|A) × P(A)) / P(B)
P(A|B) < 60%   → BLOCK
P(A|B) 60-74%  → TIER 2 (scalp, RR 1:1.0-1.2, SL di FRAMA)
P(A|B) ≥ 75%   → TIER 1 (institutional, RR 1:2.0-2.5, SL di outer liquidity)
```

### Filter 2 — CVaR Risk Engine
```
Price Velocity ≥ 250 pts/sec → HIGH TAIL RISK: cut lot 50-75%
Price Velocity < 150 pts/sec → NORMAL: 100% lot
```

### Filter 3 — Market Compass (GVZ → FRAMA / AMT + VWAP)
```
GVZ Z-Score > +1.0 → HIGH VOL: FRAMA Trend Rider
  → |Z_FRAMA| ≤ 0.5 → valid pullback → PASS
  → |Z_FRAMA| > 0.5 → overextended → BLOCK

GVZ Z-Score ≤ +1.0 → LOW VOL: AMT Volume Profile
  → Dalam POC ±5 pips → BLOCK (magnet zone)
  → Luar POC, VWAP ±2.5 overextended → BLOCK
  → Luar POC, dalam VWAP → PASS
```

### Filter 4 — Institutional Tracker (OFS)
```
OFS = S_Delta + S_CVD + S_DOM
OFS = -1, 0, +1 → BLOCK (retail noise)
OFS ≥ +2 atau ≤ -2 → PASS (institutional valid)
```

### Files
| File | Filter |
|------|--------|
| `l1_structure/filter1_bayesian.rs` | #1: Probabilitas Bayesian |
| `l1_structure/filter2_cvar.rs` | #2: Price Velocity + CVaR |
| `l1_structure/filter3_market_compass.rs` | #3: GVZ → FRAMA / AMT+VWAP |
| `l1_structure/filter4_orderflow.rs` | #4: OFS Score |
| `l1_structure/signal_classifier.rs` | Tier 1 vs Tier 2 |

---

## 🔜 FASE 6 — Indicator Implementations

### Target PRD
Implementasi 7 indicator spesifik workflow XAUUSD.

### Files & Formulas
| File | Indicator | Formula |
|------|-----------|---------|
| `indicators/frama.rs` | FRAMA | Flexible Moving Average — fractal dimension |
| `indicators/gvz.rs` | GVZ Z-Score | `(GVZ_curr - μ20) / σ20` |
| `indicators/amt_volume_profile.rs` | AMT | POC, VAH, VAL, magnet zone |
| `indicators/vwap_bands.rs` | VWAP | Session VWAP ±2.5 bands |
| `indicators/orderflow/s_delta.rs` | S_Delta | Buy aggressor - Sell aggressor |
| `indicators/orderflow/s_cvd.rs` | S_CVD | CVD divergence detection |
| `indicators/orderflow/s_dom.rs` | S_DOM | DOM heatmap Z-score |
| `indicators/body_ratio.rs` | Body Ratio | Real body / Total range `< 0.25` |
| `indicators/price_velocity.rs` | Price Velocity | Points per second `≥ 250` |

---

## 🔜 FASE 7 — Execution Layer

### Target PRD
Implementasi 3 mode eksekusi + risk management.

### Files
| File | Mode |
|------|------|
| `execution/limit_order.rs` | BUY_LIMIT / SELL_LIMIT — sniper |
| `execution/stop_order.rs` | BUY_STOP / SELL_STOP — momentum |
| `execution/instant_entry.rs` | Market order — rejection confirm |
| `execution/order_manager.rs` | Order lifecycle + validation |
| `execution/risk_manager.rs` | Lot sizing, DD limits, SL/TP |

### Risk Rules
| Parameter | Limit |
|-----------|-------|
| Max Risk per Trade | 1.0% equity |
| Max Daily Loss | 3.0% → auto-halt |
| Max Weekly Loss | 6.0% → pause 48h |
| Max Open Position | 1 |
| Spread Tolerance | ≤ 3.5 pips |

---

## 🔜 FASE 8 — Self-Learning Engine

### Target PRD
Implementasi closed-loop post-trade learning.

### Flow
```
Auto-Kill (T+3 min) → Extract (Win/Loss, Slippage, Spread)
  → SQLite Write → Rekalibrasi (spacing, counter, win map)
  → Telegram Report → Volatility Check → Loop
```

### Files
| File | Fungsi |
|------|--------|
| `analytics/trade_journal.rs` | Write trade to SQLite |
| `analytics/rekalibrasi.rs` | Optimasi threshold pasca-trade |
| `analytics/win_probability_map.rs` | Bayesian win probability |
| `analytics/auto_kill.rs` | Delete pending orders T+3 min |

### Telegram Reports
| Event | Format |
|-------|--------|
| Signal Entry | Chart + Entry/TP/SL coordinates |
| Trade Result | P/L, Slippage, Spread |
| Daily Summary | DD, WR, PF, RF |

---

## 🔜 FASE 9 — Jupyter Backtesting & Analytics

### Target PRD
Setup Jupyter notebooks untuk backtesting, performance metrics, recalibration.

### Notebooks
| Notebook | Isi |
|----------|-----|
| `backtest_analysis/xauusd_backtest.ipynb` | Full backtest engine |
| `performance/daily_report.ipynb` | Daily P/L, DD, WR |
| `performance/weekly_summary.ipynb` | Weekly aggregates |
| `modeling/bayesian_calibration.ipynb` | P(A|B) threshold optimization |
| `modeling/threshold_optimization.ipynb` | All threshold tuning |
| `modeling/cvar_risk_modeling.ipynb` | CVaR parameter fitting |

---

## 🔜 FASE 10 — Unit & Integration Tests

### Target PRD
Test coverage ≥ 80%.

### Test Files
| Test | Type |
|------|------|
| `tests/unit/commodities/XAUUSD/` | Per-function unit tests |
| `tests/integration/commodities/XAUUSD/` | Pipeline integration |
| `tests/backtest/scenarios/XAUUSD/` | Scenario backtests |

### Scenarios
| Scenario | Kondisi |
|----------|---------|
| Normal Market | Trend naik, trend turun, sideways |
| News Anomaly | Red folder trigger, Net_Dev valid/gagal |
| High Volatility | GVZ > +1.0, FRAMA mode |
| Crisis Adaptation | skip_count ≥ 3, threshold relaxation |

---

## 📊 Estimasi Timeline

| Fase | Durasi | Dependensi |
|------|--------|-----------|
| **F0** Pondasi | ✅ Selesai | — |
| **F1** L0 Infra | 2-3 hari | PyO3 bridge ready |
| **F2** L3 Engine | 3-4 hari | F1 |
| **F3** Python Bridge | 2-3 hari | F1 |
| **F4** L2 News | 4-5 hari | F3 (scraper), F2 (event loop) |
| **F5** L1 Filters | 5-7 hari | F2 (state), F4 (skip) |
| **F6** Indicators | 5-7 hari | F5 (data for filters) |
| **F7** Execution | 3-4 hari | F3 (MT5), F5/F6 (signal) |
| **F8** Self-Learning | 3-4 hari | F7 (trade data) |
| **F9** Jupyter | 2-3 hari | F8 (data) |
| **F10** Tests | 3-5 hari | Semua fase |

**Total estimasi:** 4-6 minggu

---

## ✅ Checklist Per Fase

```
FASE 0: [✅] Pondasi & Kerangka
FASE 1: [  ] L0 Infrastructure
FASE 2: [  ] L3 Master Control
FASE 3: [  ] Python Bridge
FASE 4: [  ] L2 News Sniper
FASE 5: [  ] L1 4 Filter Pipeline
FASE 6: [  ] Indicators
FASE 7: [  ] Execution
FASE 8: [  ] Self-Learning
FASE 9: [  ] Jupyter Backtesting
FASE 10: [  ] Tests (≥80% coverage)
```

> **Catatan:** Setiap fase harus lolos `cargo fmt --check`, `cargo clippy -- -D warnings`, `cargo build`, dan `cargo test` sebelum dianggap selesai.
