# Fase 1: Project Setup & Scaffolding

## Goal
Folder structure, Cargo.toml, config, mod.rs skeleton, stubs. `cargo build` pass.

## Files Created

### Root Level
```
indices/US100/
├── Cargo.toml
```

### Config
```
indices/US100/config/
├── mod.rs
├── settings.rs              # Session hours, risk limits, symbol config
├── thresholds.rs            # HV, FRAMA, OFS, VWAP, Yield thresholds
```

### Core (stubs only — semua mod.rs + function signatures kosong)
```
indices/US100/core/
├── mod.rs
├── l0_infra/mod.rs
├── l3_engine/mod.rs
├── l1_pipeline/mod.rs
├── execution/mod.rs

indices/US100/signals/mod.rs
indices/US100/indicators/mod.rs
indices/US100/indicators/orderflow/mod.rs
indices/US100/analytics/mod.rs
indices/US100/data/mod.rs
indices/US100/external/mod.rs
```

### Root workspace
```
Cargo.toml root → tambah member "indices/US100"
seith-bin/src/main.rs → tambah routing "US100" => us100::run().await
```

## PRD Reference

**Technology Stack (PRD §4):**
- Rust: tokio, sqlx, pyo3, anyhow, thiserror, chrono, serde, reqwest, log, dotenvy, uuid, polars, statrs
- Python: oandapy, python-telegram-bot, yfinance, mplfinance, pandas
- Interface: Terminal CLI only. NO web, NO GUI, NO TUI.

**Folder Structure (PRD §5):** Full tree di `docs/PRD_US100.md` §5.1.

**Binary (PRD §6):** `seith US100` → `"US100" => us100::run().await`

**Difference from XAUUSD (PRD §3, §9.1):**
- Broker OANDA (REST v20) bukan Exness MT5
- Symbol US100.cash
- 2 digit desimal (bukan 3)
- HV Z-Score (real-time) bukan GVZ
- Trending market (bukan mean-reverting)
- Macro filter + Yield indicators (tidak ada di XAUUSD)
- Session US Market Hours (14:30-21:00 UTC), bukan 24/5

**Shared Crate Usage (PRD §9.3):**
- `shared/external/oanda_bridge.rs` — sentiment / OANDA bridge
- `shared/external/telegram_bridge.rs` — Telegram dispatch
- `shared/utils/time_utils.rs` — US/Eastern timezone
- `shared/utils/math_utils.rs` — Z-Score, probability
- `shared/utils/async_helpers.rs` — retry, timeout
- `shared/data/sqlite_manager.rs` — database pool
- `shared/analytics/performance_tracker.rs` — DD, WR, PF
- `shared/analytics/report_generator.rs` — Telegram report

## Key Decisions
- Semua mod.rs pakai `pub mod` + function stub `pub async fn run() -> Result<()>`
- Config di-load dari env var + file konfigurasi
- Error handling: thiserror untuk domain errors, anyhow untuk propagation

## Dependencies
None (fase pertama)

## Acceptance Criteria
- `cargo build` pass tanpa error
- `cargo test` pass (test kosong di tiap module)
- Root workspace members include `indices/US100`
- `seith US100` CLI recognized
