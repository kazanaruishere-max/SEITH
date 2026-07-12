# Fase 2: L0 Infrastructure

## Goal
Data feed, normalizer, session filter, macro filter + SQLite layer. Level 0 siap.

## Files Created
```
indices/US100/core/l0_infra/
├── mod.rs
├── data_feed.rs              # OANDA streaming price + OHLCV via PyO3
├── normalizer.rs             # Normalisasi 2 digit desimal
├── session_filter.rs         # US Market Hours + 4 phase detection + gap flag
├── macro_filter.rs           # FOMC/CPI/NFP/GDP/PPI/ISM + earnings (all-in-one)
```

Plus: `indices/US100/data/schema.rs` dan `indices/US100/data/queries.rs`.

## PRD Reference

**Level 0 Components (PRD §2.2 Level 0 table):**

| Komponen | Fungsi |
|----------|--------|
| Day Filter | Deteksi weekend (Sabtu-Minggu) dan hari libur AS. Skip total saat market tutup |
| Data Feed Raw | Streaming US100.cash real-time dari OANDA REST API v20. HV Z-Score dihitung real-time dari rolling returns US100 (0 latensi). Auto-reconnect: retry 3x, standby 60s, Telegram alert jika gagal total |
| Session Filter | Filter waktu US Market (9:30-16:00 ET / 14:30-21:00 UTC). Deteksi 4 phase: OPEN (skip 30m), NORMAL, LUNCH (block all), CLOSE (block all). Gap flag: skip 30m pertama jika gap > 0.3%. Short-circuit: skip 5-gate pipeline jika posisi terbuka |
| Macro Calendar Filter | Deteksi FOMC, FOMC Minutes, CPI, NFP, GDP, ISM PMI, PPI + earnings AAPL/MSFT/NVDA/AMZN/GOOGL/META. Klasifikasi RED (no-trade), ORANGE (50% lot), WARNING (earnings → 50% lot). Data dari ForexFactory scraper |
| Data Normalizer | Normalisasi harga indeks 2 digit desimal |
| SQLite Database | Penyimpanan histori trading, state sistem, counter adaptasi, yield history |
| OANDA API | Eksekusi order, pengambilan data OHLCV, DOM, US10YB yield |

**Data Flow — Level 0 (PRD §2.3):**
```
Tick Baru (M1/M15 close)
    |
    +-> Level 0:
          Validasi day filter (weekend/holiday)
          Validasi session US Market + phase detection
          Cek Macro Calendar + Earnings filter
          Hitung HV Z-Score dari rolling returns US100 (real-time)
          Fetch VIX via yfinance untuk kalibrasi threshold harian (1x per session)
          Fetch US10Y + US02Y yield via OANDA
          Normalisasi harga 2 digit
```

**Session Phase Table (PRD §2.2 Execution Layer):**

| Fase | Waktu UTC | OFS Min | VWAP Band | Max Lot | Mode |
|------|-----------|---------|-----------|---------|------|
| OPEN | 14:30-15:00 | SKIP | SKIP | 0 | Stabilisasi (gap skip 30m) |
| NORMAL | 15:00-16:30 | 3 | 2.5 | 100% | SNIPER |
| LUNCH | 16:30-18:00 | SKIP | SKIP | 0 | Block semua sinyal |
| NORMAL | 18:00-19:30 | 3 | 2.5 | 100% | SNIPER |
| POWER HOUR | 19:30-20:30 | 2 | 2.0 | 75% | SCALP |
| CLOSE | 20:30-21:00 | SKIP | SKIP | 0 | Block semua sinyal |

## Key Decisions
- **Day Filter:** weekend Sabtu-Minggu + US holiday list (static config). Skip total.
- **Session Filter:** 4 phase + POWER HOUR sub-mode. Gap flag = boolean di struct SessionState.
- **Macro Filter:** RED = T-2 jam hingga rilis. Data dari ForexFactory scraper Python bridge.
- **Earnings:** Hanya 6 big tech (AAPL/MSFT/NVDA/AMZN/GOOGL/META) → WARNING → lot 50%.
- **Auto-reconnect:** 3x retry, 60s standby, Telegram alert jika gagal total.
- **Short-circuit:** Jika `has_open_position == true`, skip pipeline, langsung monitor posisi.

## Dependencies
- Phase 1 (folder structure + config)

## Acceptance Criteria
- Data feed connect ke OANDA
- Session filter deteksi phase benar (OPEN/NORMAL/LUNCH/POWER HOUR/CLOSE)
- Macro filter klasifikasi RED/ORANGE/GREEN benar
- HV Z-Score terhitung real-time (stub: panggil `compute_hv_zscore()`)
- Normalizer ubah 3.des -> 2.des
- SQLite schema terbuat
- Semua test lulus
