# PRD — AI SEITH
**Autonomous XAUUSD Trading Intelligence System**
**v1.0 | 2026-07-07**

---

## 1. TARGET PROJECT

### 1.1 Target Utama

| No | Metric | Target | Threshold Minimum | Keterangan |
|----|--------|--------|-------------------|------------|
| 1 | **Maximum Drawdown** | ≤ 8% | ≤ 12% | Dari peak equity, hard limit 15% |
| 2 | **Consecutive Win** | ≥ 8 | ≥ 5 | Trades profit berturut-turut |
| 3 | **Consecutive Loss** | ≤ 3 | ≤ 5 | Trades rugi berturut-turut, sistem auto-halt jika ≥ 5 |
| 4 | **Win Rate** | ≥ 70% | ≥ 62% | Dari total trade yang tereksekusi |
| 5 | **Recovery Factor** | ≥ 4.0 | ≥ 2.5 | Net Profit ÷ Maximum Drawdown |
| 6 | **Profit Factor** | ≥ 2.0 | ≥ 1.5 | Gross Profit ÷ Gross Loss |

### 1.2 Target Pendukung

| No | Metric | Target |
|----|--------|--------|
| 7 | Average Win : Average Loss | ≥ 1.5 : 1 |
| 8 | Expectancy per Trade | ≥ +0.5× Risk |
| 9 | Trade Frequency | 2–8 trades per hari aktif |
| 10 | Average Holding Duration | 15–180 menit |

### 1.3 Risk Hard Limits

| Parameter | Nilai | Aksi |
|-----------|-------|------|
| Max Risk per Trade | 1.0% equity | Tidak ternegosiasi |
| Max Daily Loss | 3.0% equity | Auto-halt trading hari itu |
| Max Weekly Loss | 6.0% equity | Pause 48 jam |
| Max Open Position | 1 | Single position only |
| Spread Tolerance | ≤ 3.5 pips | Reject entry jika lebih |

---

## 2. ARSITEKTUR AI SEITH

### 2.1 Architecture Overview

```
┌─────────────────────────────────────────────────────────────────────┐
│                  LEVEL 0 — INFRASTRUKTUR                            │
│                                                                     │
│  ┌──────────────┐  ┌───────────────┐  ┌────────────────────────┐   │
│  │ MT5 API      │  │ Exness Broker │  │ SQLite Database        │   │
│  │ Terminal     │  │ Data Feed     │  │ (Histori + State)      │   │
│  └──────┬───────┘  └───────┬───────┘  └────────────┬───────────┘   │
│         └──────────────────┼────────────────────────┘               │
│                            ▼                                        │
│               ┌────────────────────────┐                            │
│               │ Jam Hantu Protector    │                            │
│               │ + Data Normalizer      │                            │
│               └────────────┬───────────┘                            │
└────────────────────────────┼────────────────────────────────────────┘
                             ▼
┌─────────────────────────────────────────────────────────────────────┐
│                  LEVEL 3 — MASTER CONTROL                          │
│                                                                     │
│               ┌────────────────────────┐                            │
│               │ AI Global Event Loop   │◄──────────────────┐       │
│               │ (M1/M15 Tick-Driven)   │                   │       │
│               └────────────┬───────────┘                   │       │
│                            ▼                               │       │
│               ┌────────────────────────┐                   │       │
│               │ Adaptive State Manager │                   │       │
│               │ (skip_strike_count)    │                   │       │
│               └────────────┬───────────┘                   │       │
│                            ▼                               │       │
│          ┌─────────────────┴─────────────────┐             │       │
│          ▼                                   ▼             │       │
│  ┌───────────────┐                ┌──────────────────┐    │       │
│  │ LEVEL 2:      │                │ LEVEL 1:         │    │       │
│  │ NEWS SNIPER   │                │ STRUCTURE NAV    │    │       │
│  │ ENGINE        │                │ ENGINE           │    │       │
│  └───────┬───────┘                └────────┬─────────┘    │       │
│          └─────────────────┬────────────────┘              │       │
│                            ▼                               │       │
│               ┌────────────────────────┐                   │       │
│               │ Statistical Brain      │                   │       │
│               │ + Anti-Paralysis       │                   │       │
│               └────────────┬───────────┘                   │       │
└────────────────────────────┼───────────────────────────────┘       │
                             ▼                                       │
┌─────────────────────────────────────────────────────────────────────┐
│                  EXECUTION LAYER                                    │
│                                                                     │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────────────────┐  │
│  │ Limit Order  │  │ Stop Order   │  │ Manual Instant Entry     │  │
│  │ (Sniper)     │  │ (Momentum)   │  │ (Rejection Confirm)      │  │
│  └──────┬───────┘  └──────┬───────┘  └────────────┬─────────────┘  │
│         └─────────────────┼────────────────────────┘               │
│                           ▼                                        │
│               ┌────────────────────┐                               │
│               │ MT5 API Execution  │                               │
│               └────────────┬───────┘                               │
└────────────────────────────┼────────────────────────────────────────┘
                             ▼
┌─────────────────────────────────────────────────────────────────────┐
│             CLOSED-LOOP SELF-LEARNING ENGINE                        │
│                                                                     │
│  ┌─────────────┐  ┌──────────────┐  ┌─────────────────────────┐   │
│  │ Post-Trade  │→ │ SQLite Write │→ │ Cognitive Re-Calibration│   │
│  │ Extract     │  │              │  │ (Slippage, Counter, Map)│   │
│  └─────────────┘  └──────────────┘  └────────────┬────────────┘   │
│                                                   │                │
│                                                   ▼                │
│               ┌────────────────────────┐                           │
│               │ Telegram Dispatcher    │                           │
│               │ (Chart + Signal + P/L) │                           │
│               └────────────┬───────────┘                           │
│                            │                                       │
│                            ▼                                       │
│               ┌────────────────────────┐                           │
│               │ Loop End → Reset →     │                           │
│               │ Back to Event Loop     │──────────────────────────┘
│               └────────────────────────┘
└─────────────────────────────────────────────────────────────────────┘
```

### 2.2 Level Descriptions

#### LEVEL 0 — Core Environment & Infrastructure

| Komponen | Fungsi |
|----------|--------|
| **Data Feed Raw** | Streaming harga XAUUSDm real-time dari Exness |
| **Jam Hantu Protector** | Force close semua posisi Tier 2 saat jam 20:45 server (risiko spread lock rollover) |
| **Data Normalizer** | Konversi desimal emas ke format 3 digit (multiplier 0.010) |
| **SQLite Database** | Penyimpanan histori trading, state sistem, dan counter adaptasi |
| **MT5 API Terminal** | Eksekusi order dan pengambilan data OHLCV |

#### LEVEL 3 — Master Control Pipeline & Event Loop

| Komponen | Fungsi |
|----------|--------|
| **AI Global Event Loop** | Event-driven loop yang aktif setiap penutupan lilin M1/M15 |
| **Adaptive State Manager** | Membaca `skip_strike_count` dari SQLite sebelum setiap iterasi |
| **Dual-Parallel Calendar Scraper** | Scraping berita simultan ke ForexFactory + Investing.com (async) |

#### LEVEL 2 — News Anomaly Sniper Engine

| Komponen | Fungsi | Threshold |
|----------|--------|-----------|
| **Red Folder Detector** | Identifikasi berita USD high-impact | T-30 s/d T-60 menit |
| **Regime Switch** | Aktifkan NEWS_ANOMALY_MODE | Matikan total Level 1 |
| **Fast Polling** | Async request milidetik | T-1 menit hingga rilis |
| **Data Extractor** | Ambil Actual, Forecast, Revised Previous | Dual-source + fallback |
| **Revision Handler** | Deteksi manipulasi angka bulan lalu | Revised vs Original |
| **Net_Dev Calculator** | Skor deviasi bersih | `(A-F) + (RP-OP)` |
| **Deviation Evaluator** | Validasi kekuatan deviasi | `\|Net_Dev\| ≥ 2.0` = valid |

#### LEVEL 1 — Market Structure Navigation Engine

4 Filter bertingkat saat kondisi pasar normal:

**Filter 1 — Bayesian Gatekeeper**
```
P(A|B) < 60%   → BLOCK (retail noise)
P(A|B) 60-74%  → TIER 2: Tactical Scalp, RR 1:1.0–1.2, SL di FRAMA
P(A|B) ≥ 75%   → TIER 1: Institutional Ride, RR 1:2.0–2.5, SL di Outer Liquidity Pool
```

**Filter 2 — CVaR Risk Engine**
```
Price Velocity ≥ 250 pts/sec → HIGH TAIL RISK: potong lot 50-75%
Price Velocity < 150 pts/sec → NORMAL: 100% lot standar
```

**Filter 3 — Market Compass (GVZ Z-Score)**
```
GVZ Z-Score > +1.0 → HIGH VOL: FRAMA Trend Rider M15
  → Z_FRAMA ≤ 0.5 = valid pullback → PASS
  → Z_FRAMA > 0.5 = overextended → BLOCK (FOMO)

GVZ Z-Score ≤ +1.0 → LOW VOL: AMT Volume Profile M15/H1
  → Di dalam POC ±5 pips = magnet zone → BLOCK
  → Di luar POC, VWAP band ±2.5 = overextended → BLOCK
  → Di luar POC, dalam VWAP band = fair area → PASS
```

**Filter 4 — Institutional Tracker (OFS)**
```
OFS = S_Delta + S_CVD + S_DOM
OFS = -1, 0, +1 → BLOCK (retail noise, tidak ada agresor kakap)
OFS ≥ +2 atau ≤ -2 → PASS (sinyal strategis valid)
```

#### Statistical Brain & Anti-Paralysis

```
skip_count < 3  → STATIC SAFE: izinkan skip, jaga ekuitas
skip_count ≥ 3  → CRISIS ADAPTATION:
                   - Turunkan jarak pending order
                   - Kurangi toleransi spread 15%
                   - Longgarkan OFS dari min 2 → cukup 1
                   - PAKSA eksekusi
```

#### Execution Layer — 3 Mode Peluru

| Mode | Trigger | Order Type |
|------|---------|------------|
| **Limit Order (Sniper)** | Harga overextended / antisipasi sumbu berlawanan | BUY_LIMIT / SELL_LIMIT di Outer Liquidity Pool + spread buffer |
| **Stop Order (Momentum)** | Kompresi sempit / breakout konfirmasi | BUY_STOP / SELL_STOP di luar range konsolidasi M1 |
| **Manual Instant Entry** | News abu-abu + rejection konfluen | Market Order: Real Body Ratio < 0.25 & velocity ≥ 200 pts/sec |

#### Closed-Loop Self-Learning

```
Tahap 1: Auto-Kill → hapus semua pending order (T+3 menit post-news)
Tahap 2: Extract → Win/Loss, Slippage, Spread, P/L Cent
Tahap 3: SQLite Write → simpan baris baru hasil eksekusi
Tahap 4: Rekalisbrasi → optimasi spacing buffer, reset counter, update win probability map
Tahap 5: Telegram Report → kirim laporan P/L akun cent riil
Tahap 6: Regime Reset → cek volatilitas, jika normal kembali ke NORMAL_REGULER_MODE
Tahap 7: Loop → kembali ke Event Loop awal
```

### 2.3 Data Flow

```
Tick Baru (M1/M15 close)
    │
    ├─→ Level 0: Validasi jam broker, normalisasi harga
    │
    ├─→ Level 3: Baca skip_strike_count
    │
    ├─→ Level 2: Cek Red Folder?
    │       ├─ YES → News Pipeline → Net_Dev ≥ 2.0 → Valid
    │       └─ NO  → Level 1 (4 Filters)
    │
    ├─→ Statistical Synthesis: Entry, TP, SL
    │
    ├─→ Execution: Pilih peluru → MT5 API
    │
    └─→ Telegram: Chart + sinyal
```

### 2.4 State Machine

```
[BOOT] → [NORMAL_REGULER_MODE] ←────────────────────┐
    │                                                 │
    ├─ Red Folder → [NEWS_ANOMALY_MODE] ──────────────┤
    │                                                  │
    ├─ Post-Trade → Rekalibrasi → ────────────────────┘
    │
    ├─ Skip ≥ 3 → [CRISIS_ADAPTATION] → Force Entry
    │
    └─ Jam 20:45 + Tier 2 → [FORCE_CLOSE] → Standby
```

---

## 3. TECHNOLOGY STACK

| Layer | Bahasa | Kegunaan |
|-------|--------|----------|
| **Core Logic & Codebase** | **Rust** | Semua engine (L0–L3), execution, indicators, signals, risk management, database access |
| **Library Bridge** | **Python** | MT5 API wrapper, Telegram Bot API, web scraping (news), mplfinance charting |
| **Statistical Modeling** | **Jupyter Notebook** | Backtesting analysis, performance metrics, recalibration R&D, visualisasi statistik |
| **Infrastructure** | **Docker** | Cetak biru infrastruktur cadangan — development & running BOT dilakukan NATIVE di host OS lokal |
| **Version Control** | **Git** | Source control, branching strategy, CI/CD hooks |
| **Interface** | **Terminal (CLI)** | ❌ TIDAK ADA web server, TUI, atau GUI — semua interaksi via terminal/command line |

### 3.1 Interface: Terminal Only (CLI)

> ⚠️ **TIDAK ADA web server, TUI, atau GUI** — Semua interaksi dilakukan via terminal/command line.

| Aspek | Penjelasan |
|-------|------------|
| **Running Bot** | `seith XAUUSD` — output log ke terminal |
| **Monitoring** | Log text ke terminal + notifikasi Telegram |
| **Control** | Perintah via terminal atau Telegram bot commands |
| **Backtesting** | Jupyter Notebook (lokal, bukan web) |
| **Configuration** | File `.env` + `settings.rs` — edit manual, bukan web UI |
| **Database** | SQLite via terminal query atau programmatic access |

**❌ TIDAK ADA:**
- Web dashboard
- TUI (Terminal User Interface)
- GUI desktop
- REST API untuk frontend
- WebSocket server

**✅ YANG ADA:**
- Log output ke terminal
- Telegram Bot untuk notifikasi & control
- Jupyter Notebook untuk analisis statistik
- File-based configuration

---

### 3.2 Kenapa Rust untuk Core?

| Alasan | Penjelasan |
|--------|------------|
| **Zero-cost Abstraction** | Tinggi performa tanpa overhead runtime |
| **Memory Safety** | Tanpa garbage collector, cocok untuk real-time trading |
| **Concurrency** | `tokio` async runtime — ideal untuk event loop + parallel scraper |
| **Determinism** | Tidak ada GC pause yang bisa ganggu latency eksekusi |
| **Compile-time Guarantee** | Error ditangkap saat build, bukan saat live trading |

### 3.3 Daftar Framework & Library

#### Rust Crates (Core)

| Crate | Versi | Kegunaan | Lokasi |
|-------|-------|----------|--------|
| `tokio` | 1.x | Async runtime, event loop, spawn task | Semua modul |
| `sqlx` | 0.7 | Async SQLite pool, query, migrate — sinkron dengan tokio | `shared/data/` |
| `pyo3` | 0.20 | Python FFI bridge | `python/` crate |
| `thiserror` | 2.x | Custom error types | `shared/utils/exceptions.rs` |
| `anyhow` | 1.x | Error propagation (dev) | Semua modul |
| `chrono` | 0.4 | Timezone, timestamp, session check | `shared/utils/time_utils.rs` |
| `serde` | 1.x | JSON serialization | Semua modul |
| `serde_json` | 1.x | JSON parsing | Semua modul |
| `reqwest` | 0.12 | HTTP client (news aggregator) | `shared/external/news_aggregator.rs` |
| `tokio-macros` | 1.x | Macro async untuk test | `tests/` |
| `log` | 0.4 | Logging facade | Semua modul |
| `env_logger` | 0.11 | Logger backend | `main.rs` |
| `dotenvy` | 0.15 | Load `.env` file | `shared/config/env_loader.rs` |
| `uuid` | 1.x | Unique trade ID | `shared/` |
| `tracing` | 0.1 | Structured logging | Semua modul |
| `maturin` | 1.x | Build Python package dari Rust | `python/` |

#### Python Libraries (Bridge)

| Library | Fungsi | Lokasi |
|---------|--------|--------|
| `MetaTrader5` | MT5 API wrapper — koneksi ke broker Exness | `python/seith_bridge/mt5.py` |
| `python-telegram-bot` | Telegram Bot API — notifikasi + control | `python/seith_bridge/telegram.py` |
| `beautifulsoup4` | Web scraping — Forex Factory | `python/seith_bridge/scraper.py` |
| `playwright` | Browser automation — Investing.com | `python/seith_bridge/scraper.py` |
| `requests` | HTTP client — API calls | `python/seith_bridge/` |
| `mplfinance` | Chart generation — analisis visual | `jupyter/` |
| `pandas` | Data manipulation — trade log analysis | `jupyter/` |
| `numpy` | Numerical computation | `jupyter/` |
| `scipy` | Statistical analysis | `jupyter/` |
| `jupyter` | Interactive notebook | `jupyter/` |

#### Rust Crates (per Instrument — XAUUSD)

| Crate | Fungsi | Lokasi |
|-------|--------|--------|
| `polars` | DataFrame — OHLCV processing, rolling stats | `indicators/`, `core/` |
| `statrs` | Statistical distributions — Normal, Z-Score calc | `indicators/gvz.rs` |
| `float-cmp` | Floating point comparison — threshold check | `utils/` |

> **Catatan:** Tidak ada RSI, ATR, BB, MACD. Indikator spesifik workflow XAUUSD:
> - FRAMA, GVZ Z-Score, AMT Volume Profile, VWAP Bands, Orderflow (S_Delta + S_CVD + S_DOM), Body Ratio, Price Velocity

---

### 3.4 Kenapa Python Hanya untuk Library (Bridge)?

| Komponen | Alasan Python |
|----------|---------------|
| MT5 API | MetaTrader5 package hanya tersedia di Python |
| Telegram Bot | `python-telegram-bot` paling matang |
| Web Scraping | BeautifulSoup/Playwright lebih mudah di Python |
| Charting | `mplfinance` hanya ada di Python |
| **Koneksi Rust↔Python** | PyO3 / maturin untuk FFI bridge |

### 3.5 Kenapa Jupyter untuk Statistik?

| Use Case | Penjelasan |
|----------|------------|
| Backtesting Report | Analisis performa histori dengan visualisasi interaktif |
| Statistical Modeling | Kalkulasi ulang threshold, recalibration parameter |
| Performance Metrics | Hitung DD, WR, PF, RF, CW, CL dengan visual charts |
| R&D Prototyping | Uji strategi baru sebelum diimplementasi ke Rust |

---

## 4. KERANGKA FOLDER AI SEITH

### 4.1 Struktur Folder

```
AI_SEITH/
│
├── shared/                            # Layer Cross-Instrument (infrastruktur bersama)
│   │
│   ├── config/                        #   Konfigurasi global
│   │   ├── mod.rs                     #     Module declaration
│   │   ├── settings.rs                #     Risk limits, thresholds, session hours
│   │   └── env_loader.rs              #     Load .env → MT5 credentials, Telegram token
│   │
│   ├── external/                      #   Koneksi ke layanan eksternal
│   │   ├── mod.rs
│   │   ├── mt5_bridge.rs              #     PyO3 bridge → Python MT5 API wrapper
│   │   ├── telegram_bridge.rs         #     PyO3 bridge → Python Telegram Bot API
│   │   └── news_aggregator.rs         #     Async dual-parallel calendar scraper
│   │
│   ├── utils/                         #   Fungsi utilitas umum
│   │   ├── mod.rs
│   │   ├── time_utils.rs              #     Konversi waktu broker, session detector
│   │   ├── math_utils.rs              #     Z-Score, CVaR, Bayesian probability
│   │   ├── async_helpers.rs            #     Tokio retry, timeout, semaphore
│   │   └── exceptions.rs              #     Custom error types (thiserror)
│   │
│   ├── data/                          #   Database shared
│   │   ├── mod.rs
│   │   ├── sqlite_manager.rs          #     sqlx async pool — connection, read, write
│   │   ├── migrations/                #     Schema versioning
│   │   │   ├── v001_init.sql
│   │   │   └── v002_add_columns.sql
│   │   └── seed/                      #     Data awal jika diperlukan
│   │
│   └── analytics/                     #   Performance tracking shared
│       ├── mod.rs
│       ├── performance_tracker.rs     #     Hitung DD, WR, PF, RF, CW, CL
│       └── report_generator.rs        #     Format laporan ke Telegram / file
│
├── commodities/                       # Module Komoditas
│   │
│   └── XAUUSD/                        #   Gold — Exness (FULL PIPELINE)
│       │
│       ├── Cargo.toml                 #     Crate config untuk XAUUSD module
│       ├── config/
│       │   ├── mod.rs
│       │   ├── settings.rs            #       Spread tolerance, digit normalization, session
│       │   └── thresholds.rs          #       Net_Dev, OFS, GVZ, Bayesian thresholds
│       │
│       ├── core/
│       │   ├── mod.rs
│       │   ├── L0_infra/
│       │   │   ├── mod.rs
│       │   │   ├── data_feed.rs
│       │   │   ├── normalizer.rs
│       │   │   └── jam_hantu.rs
│       │   ├── L3_engine/
│       │   │   ├── mod.rs
│       │   │   ├── event_loop.rs
│       │   │   ├── state_manager.rs
│       │   │   ├── statistical_brain.rs
│       │   │   └── anti_paralysis.rs
│       │   ├── L2_news/
│       │   │   ├── mod.rs
│       │   │   ├── red_folder_detector.rs
│       │   │   ├── regime_switch.rs
│       │   │   ├── fast_poller.rs
│       │   │   ├── data_extractor.rs
│       │   │   ├── revision_handler.rs
│       │   │   └── net_dev_calculator.rs
│       │       ├── L1_structure/              #   4 Filter Pipeline (urutan wajib)
│       │       │   ├── mod.rs
│       │       │   ├── filter1_bayesian.rs    #     Filter 1 — Probabilitas Bayesian (P(A|B) >= 60%)
│       │       │   ├── filter2_cvar.rs        #     Filter 2 — CVaR Risk Engine (Price Velocity check)
│       │       │   ├── filter3_market_compass.rs  # Filter 3 — GVZ Regime → FRAMA atau AMT+VWAP
│       │       │   ├── filter4_orderflow.rs   #     Filter 4 — OFS Score (S_Delta+S_CVD+S_DOM >= 2)
│       │       │   └── signal_classifier.rs   #     Tier Classification (T1=Inst Ride, T2=Tactical Scalp)
│       │   └── execution/
│       │       ├── mod.rs
│       │       ├── limit_order.rs
│       │       ├── stop_order.rs
│       │       ├── instant_entry.rs
│       │       ├── order_manager.rs
│       │       └── risk_manager.rs
│       │
│       ├── signals/
│       │   ├── mod.rs
│       │   ├── signal_types.rs        #       Enum: BuySignal, SellSignal, NoSignal
│       │   ├── signal_validator.rs
│       │   └── signal_enricher.rs
│       │
│       ├── indicators/
│       │   ├── mod.rs
│       │   ├── frama.rs                  #       Flexible Moving Average — Trend Rider M15
│       │   ├── gvz.rs                    #       GVZ Z-Score Calculator — Market Regime Detector
│       │   ├── amt_volume_profile.rs     #       AMT Volume Profile — POC/VAH/VAL/Magnet Zone
│       │   ├── vwap_bands.rs             #       VWAP Session Deviation Bands ±2.5
│       │   ├── orderflow/
│       │   │   ├── mod.rs
│       │   │   ├── s_delta.rs            #       Delta Score — Buy vs Sell Aggressor
│       │   │   ├── s_cvd.rs              #       CVD Divergence — Cumulative Volume Delta
│       │   │   └── s_dom.rs              #       DOM Heatmap Z-Score — Order Book Imbalance
│       │   ├── body_ratio.rs             #       Real Body Ratio — Rejection Detection
│       │   └── price_velocity.rs         #       Price Velocity — Points Per Second
│       │
│       ├── analytics/
│       │   ├── mod.rs
│       │   ├── trade_journal.rs
│       │   ├── rekalibrasi.rs
│       │   ├── win_probability_map.rs
│       │   └── auto_kill.rs
│       │
│       ├── data/
│       │   ├── mod.rs
│       │   ├── schema.sql
│       │   └── queries.rs
│       │
│       └── external/
│           ├── mod.rs
│           ├── forex_factory.rs       #       PyO3 bridge → Python scraper
│           └── investing_com.rs       #       PyO3 bridge → Python scraper
│
├── majors/                            # Module Major Pairs (PLACEHOLDER — beda sistem per pair)
│   │
│   ├── EURUSD/                        #   Euro / US Dollar
│   │   ├── Cargo.toml
│   │   ├── config/
│   │   │   ├── mod.rs
│   │   │   ├── settings.rs
│   │   │   └── thresholds.rs
│   │   ├── core/
│   │   │   ├── mod.rs
│   │   │   └── strategy.rs            #       Strategy skeleton — TODO
│   │   ├── signals/
│   │   │   ├── mod.rs
│   │   │   └── signal_types.rs
│   │   ├── indicators/
│   │   │   ├── mod.rs
│   │   │   └── custom.rs
│   │   ├── analytics/
│   │   │   ├── mod.rs
│   │   │   └── trade_journal.rs
│   │   └── data/
│   │       ├── mod.rs
│   │       ├── schema.sql
│   │       └── queries.rs
│   │
│   ├── GBPUSD/                        #   British Pound / US Dollar
│   │   ├── Cargo.toml
│   │   ├── config/
│   │   │   ├── mod.rs
│   │   │   ├── settings.rs
│   │   │   └── thresholds.rs
│   │   ├── core/
│   │   │   ├── mod.rs
│   │   │   └── strategy.rs
│   │   ├── signals/
│   │   │   ├── mod.rs
│   │   │   └── signal_types.rs
│   │   ├── indicators/
│   │   │   ├── mod.rs
│   │   │   └── custom.rs
│   │   ├── analytics/
│   │   │   ├── mod.rs
│   │   │   └── trade_journal.rs
│   │   └── data/
│   │       ├── mod.rs
│   │       ├── schema.sql
│   │       └── queries.rs
│   │
│   ├── USDJPY/                        #   US Dollar / Japanese Yen
│   │   ├── Cargo.toml
│   │   ├── config/
│   │   │   ├── mod.rs
│   │   │   ├── settings.rs
│   │   │   └── thresholds.rs
│   │   ├── core/
│   │   │   ├── mod.rs
│   │   │   └── strategy.rs
│   │   ├── signals/
│   │   │   ├── mod.rs
│   │   │   └── signal_types.rs
│   │   ├── indicators/
│   │   │   ├── mod.rs
│   │   │   └── custom.rs
│   │   ├── analytics/
│   │   │   ├── mod.rs
│   │   │   └── trade_journal.rs
│   │   └── data/
│   │       ├── mod.rs
│   │       ├── schema.sql
│   │       └── queries.rs
│   │
│   ├── USDCHF/                        #   US Dollar / Swiss Franc
│   │   ├── Cargo.toml
│   │   ├── config/
│   │   │   ├── mod.rs
│   │   │   ├── settings.rs
│   │   │   └── thresholds.rs
│   │   ├── core/
│   │   │   ├── mod.rs
│   │   │   └── strategy.rs
│   │   ├── signals/
│   │   │   ├── mod.rs
│   │   │   └── signal_types.rs
│   │   ├── indicators/
│   │   │   ├── mod.rs
│   │   │   └── custom.rs
│   │   ├── analytics/
│   │   │   ├── mod.rs
│   │   │   └── trade_journal.rs
│   │   └── data/
│   │       ├── mod.rs
│   │       ├── schema.sql
│   │       └── queries.rs
│   │
│   ├── USDCAD/                        #   US Dollar / Canadian Dollar
│   │   ├── Cargo.toml
│   │   ├── config/
│   │   │   ├── mod.rs
│   │   │   ├── settings.rs
│   │   │   └── thresholds.rs
│   │   ├── core/
│   │   │   ├── mod.rs
│   │   │   └── strategy.rs
│   │   ├── signals/
│   │   │   ├── mod.rs
│   │   │   └── signal_types.rs
│   │   ├── indicators/
│   │   │   ├── mod.rs
│   │   │   └── custom.rs
│   │   ├── analytics/
│   │   │   ├── mod.rs
│   │   │   └── trade_journal.rs
│   │   └── data/
│   │       ├── mod.rs
│   │       ├── schema.sql
│   │       └── queries.rs
│   │
│   └── AUDUSD/                        #   Australian Dollar / US Dollar
│       ├── Cargo.toml
│       ├── config/
│       │   ├── mod.rs
│       │   ├── settings.rs
│       │   └── thresholds.rs
│       ├── core/
│       │   ├── mod.rs
│       │   └── strategy.rs
│       ├── signals/
│       │   ├── mod.rs
│       │   └── signal_types.rs
│       ├── indicators/
│       │   ├── mod.rs
│       │   └── custom.rs
│       ├── analytics/
│       │   ├── mod.rs
│       │   └── trade_journal.rs
│       └── data/
│           ├── mod.rs
│           ├── schema.sql
│           └── queries.rs
│
├── crypto/                            # Module Cryptocurrency (PLACEHOLDER — beda sistem per aset)
│   │
│   ├── BTCUSD/                        #   Bitcoin / US Dollar
│   │   ├── Cargo.toml
│   │   ├── config/
│   │   │   ├── mod.rs
│   │   │   ├── settings.rs            #       Crypto: 24/7 session, higher vol
│   │   │   └── thresholds.rs
│   │   ├── core/
│   │   │   ├── mod.rs
│   │   │   └── strategy.rs            #       Strategy skeleton — TODO
│   │   ├── signals/
│   │   │   ├── mod.rs
│   │   │   └── signal_types.rs
│   │   ├── indicators/
│   │   │   ├── mod.rs
│   │   │   └── custom.rs
│   │   ├── analytics/
│   │   │   ├── mod.rs
│   │   │   └── trade_journal.rs
│   │   └── data/
│   │       ├── mod.rs
│   │       ├── schema.sql
│   │       └── queries.rs
│   │
│   └── ETHUSD/                        #   Ethereum / US Dollar
│       ├── Cargo.toml
│       ├── config/
│       │   ├── mod.rs
│       │   ├── settings.rs
│       │   └── thresholds.rs
│       ├── core/
│       │   ├── mod.rs
│       │   └── strategy.rs
│       ├── signals/
│       │   ├── mod.rs
│       │   └── signal_types.rs
│       ├── indicators/
│       │   ├── mod.rs
│       │   └── custom.rs
│       ├── analytics/
│       │   ├── mod.rs
│       │   └── trade_journal.rs
│       └── data/
│           ├── mod.rs
│           ├── schema.sql
│           └── queries.rs
│
├── jupyter/                           # Jupyter Notebooks — Statistical Modeling & Performance
│   │
│   ├── backtest_analysis/             #   Analisis hasil backtest
│   │   ├── xauusd_backtest.ipynb
│   │   ├── eurusd_backtest.ipynb
│   │   └── btcusd_backtest.ipynb
│   │
│   ├── performance/                   #   Performa live trading
│   │   ├── daily_report.ipynb
│   │   ├── weekly_summary.ipynb
│   │   └── drawdown_analysis.ipynb
│   │
│   ├── modeling/                      #   Statistical modeling & R&D
│   │   ├── bayesian_calibration.ipynb
│   │   ├── threshold_optimization.ipynb
│   │   ├── cvar_risk_modeling.ipynb
│   │   └── recalibration_experiments.ipynb
│   │
│   └── shared/                        #   Shared notebooks utilities
│       └── utils.py                   #       Helper functions untuk notebooks
│
├── python/                            # Python Library Bridge (PyO3 / maturin)
│   │
│   ├── pyproject.toml                 #     Python package config
│   ├── Cargo.toml                     #     Rust crate for PyO3
│   ├── src/
│   │   ├── lib.rs                     #       PyO3 module entry
│   │   ├── mt5_wrapper.rs             #       Python MT5 API → Rust FFI
│   │   ├── telegram_wrapper.rs        #       Python Telegram Bot → Rust FFI
│   │   └── scraper_wrapper.rs         #       Python web scraping → Rust FFI
│   └── python/
│       └── seith_bridge/
│           ├── __init__.py
│           ├── mt5.py                  #         MT5 wrapper functions
│           ├── telegram.py             #         Telegram wrapper functions
│           └── scraper.py              #         Scraper wrapper functions
│
├── tests/                             # Test Suite (Rust: #[cfg(test)])
│   │
│   ├── unit/
│   │   ├── shared/
│   │   │   ├── time_utils_test.rs
│   │   │   ├── math_utils_test.rs
│   │   │   ├── sqlite_manager_test.rs
│   │   │   └── performance_tracker_test.rs
│   │   ├── commodities/
│   │   │   └── XAUUSD/
│   │   │       ├── bayesian_gatekeeper_test.rs
│   │   │       ├── cvar_risk_engine_test.rs
│   │   │       ├── market_compass_test.rs
│   │   │       ├── institutional_tracker_test.rs
│   │   │       ├── net_dev_calculator_test.rs
│   │   │       ├── risk_manager_test.rs
│   │   │       └── signal_validator_test.rs
│   │   ├── majors/
│   │   │   └── EURUSD/
│   │   │       └── .gitkeep
│   │   └── crypto/
│   │       └── BTCUSD/
│   │           └── .gitkeep
│   │
│   ├── integration/
│   │   ├── shared/
│   │   │   ├── mt5_bridge_test.rs
│   │   │   ├── telegram_bridge_test.rs
│   │   │   └── news_aggregator_test.rs
│   │   ├── commodities/
│   │   │   └── XAUUSD/
│   │   │       ├── event_loop_test.rs
│   │   │       ├── news_pipeline_test.rs
│   │   │       ├── full_cycle_test.rs
│   │   │       └── self_learning_test.rs
│   │   ├── majors/
│   │   │   └── EURUSD/
│   │   │       └── .gitkeep
│   │   └── crypto/
│   │       └── BTCUSD/
│   │           └── .gitkeep
│   │
│   └── backtest/
│       ├── scenarios/
│       │   ├── XAUUSD/
│       │   │   ├── normal_market_test.rs
│       │   │   ├── news_anomaly_test.rs
│       │   │   ├── high_volatility_test.rs
│       │   │   └── crisis_adaptation_test.rs
│       │   ├── majors/
│       │   │   └── EURUSD/
│       │   │       └── .gitkeep
│       │   └── crypto/
│       │       └── BTCUSD/
│       │           └── .gitkeep
│       └── results/
│           ├── XAUUSD/
│           │   └── .gitkeep
│           ├── majors/
│           │   └── EURUSD/
│           │       └── .gitkeep
│           └── crypto/
│               └── BTCUSD/
│                   └── .gitkeep
│
├── scripts/                           # Automation scripts
│   ├── run_backtest.sh               #   Jalankan backtest (--instrument flag)
│   ├── run_paper_trade.sh            #   Jalankan paper trading
│   ├── run_live.sh                   #   Jalankan live trading
│   ├── db_migrate.sh                 #   Jalankan migrasi database
│   └── generate_report.sh            #   Generate laporan performa
│
├── docker/                            # Docker Configuration (BLUEPRINT — Infrastruktur Cadangan)
│   │                                 #   ⚠️ TIDAK DIGUNAKAN — Semua berjalan NATIVE di host OS lokal
│   │                                 #   Alasan: efisiensi memori + interaksi native MT5 API
│   ├── Dockerfile.rust                #   Build stage: Rust core → production binary
│   ├── Dockerfile.python              #   Python bridge + MT5/Telegram dependencies
│   ├── Dockerfile.jupyter             #   Jupyter environment untuk analisis
│   ├── docker-compose.yml             #   Orchestrate semua services
│   ├── docker-compose.dev.yml         #   Development overrides (hot reload, volumes)
│   └── .dockerignore
│
├── .git/                              # Git Repository
│   ├── hooks/                         #   Custom hooks
│   │   ├── pre-commit                #     cargo fmt + cargo clippy
│   │   └── pre-push                 #     cargo test
│   └── config                         #   Repository config
│
├── .github/                           # GitHub Actions CI/CD
│   └── workflows/
│       ├── ci.yml                     #   Build + Test + Lint on push/PR
│       ├── release.yml                #   Build release binary on tag
│       └── docker.yml                 #   Build + push Docker image
│
├── docs/                              # Dokumentasi
│   ├── WORKFLOW_XAUUSD.mmd
│   ├── PRD_AI_SEITH.md
│   └── architecture.md
│
├── Cargo.toml                         # Workspace root — Rust workspace config
├── Cargo.lock                         # Dependency lock file
├── .env                               # Environment variables (MT5, Telegram)
├── .gitignore                         # Ignore target/, db, logs, .env, *.pyc
├── Dockerfile                         # Default Dockerfile
├── README.md                          # Setup guide & usage
└── rust-toolchain.toml               # Pin Rust version (stable/nightly)
```

### 4.2 Mapping Layer → Folder

| Layer | Folder | Bahasa | Keterangan |
|-------|--------|--------|------------|
| **Cross-Instrument** | `shared/` | Rust | Infrastruktur bersama (config, MT5 bridge, telegram bridge, utils, database, analytics) |
| **Commodities** | `commodities/XAUUSD/` | Rust | Full pipeline Gold — L0–L3, execution, signals, indicators, analytics |
| **Major Pairs** | `majors/{PAIR}/` | Rust | Placeholder per pasangan — beda sistem, beda strategi |
| **Crypto** | `crypto/{ASSET}/` | Rust | Placeholder per aset — beda sistem, 24/7 session |
| **Python Bridge** | `python/` | Rust+Python | PyO3 FFI bridge ke MT5, Telegram, Scraper |
| **Statistical** | `jupyter/` | Python | Backtest analysis, performance metrics, modeling |
| **Test Unit** | `tests/unit/` | Rust | `#[cfg(test)]` per-fungsi |
| **Test Integration** | `tests/integration/` | Rust | Test pipeline end-to-end |
| **Test Backtest** | `tests/backtest/` | Rust | Skenario simulasi + hasil |
| **Docker** | `docker/` | Dockerfile | Cetak biru infrastruktur cadangan (tidak digunakan saat ini) |

### 4.3 Dependency Flow

```
seith XAUUSD
  │
  └─→ commodities/XAUUSD/core/  (atau majors/EURUSD/ atau crypto/BTCUSD/)
        │
        ├── L0_infra/        → shared/external/mt5_bridge.rs (→ python MT5 API)
        │                      shared/data/sqlite_manager.rs (sqlx async)
        │
        ├── L3_engine/       → shared/utils/time_utils.rs
        │                      shared/utils/math_utils.rs
        │                      L2_news/
        │                      L1_structure/
        │
        ├── L2_news/         → shared/external/news_aggregator.rs
        │                      {instrument}/external/ (→ python scraper)
        │
        ├── L1_structure/    → {instrument}/indicators/
        │
        └── execution/       → shared/external/mt5_bridge.rs
                               {instrument}/config/thresholds.rs
```

### 4.4 Binary Alias: `seith`

**Setup (sekali saja):**
```bash
# Dari root project AI_SEITH/
cargo install --path .
```

**Gunakan:**
```bash
# Running XAUUSD
seith XAUUSD

# Running EURUSD (future)
seith EURUSD

# Running BTCUSD (future)
seith BTCUSD
```

**Cargo.toml (root):**
```toml
[[bin]]
name = "seith"
path = "src/main.rs"
```

---

### 4.5 Docker Architecture (BLUEPRINT — Infrastruktur Cadangan)

> ⚠️ **TIDAK DIGUNAKAN SAAT INI** — Semua development, riset Jupyter, dan running BOT harian dilakukan **NATIVE** di host OS lokal.
> 
> **Alasan:**
> - Efisiensi memori (tidak perlu VM/container overhead)
> - Interaksi native MT5 API (MetaTrader5 package hanya jalan di Windows + Python native)
> - Kemudahan debugging lokal
> 
> Docker hanya disiapkan sebagai cetak biru untuk deployment ke Linux server di masa depan jika diperlukan.

```
┌─────────────────────────────────────────────────────┐
│  docker-compose.yml                                  │
│                                                      │
│  ┌──────────────────┐  ┌──────────────────────────┐ │
│  │  seith-core       │  │  seith-python-bridge     │ │
│  │  (Rust binary)    │←→│  (Python 3.11 + MT5)     │ │
│  │  Port: 8080       │  │  Port: 5000              │ │
│  └────────┬─────────┘  └──────────────────────────┘ │
│           │                                          │
│           ▼                                          │
│  ┌──────────────────┐  ┌──────────────────────────┐ │
│  │  seith-db         │  │  seith-jupyter           │ │
│  │  (SQLite volume)  │  │  (Statistical modeling)  │ │
│  │  Volume: ./data   │  │  Port: 8888              │ │
│  └──────────────────┘  └──────────────────────────┘ │
│                                                      │
│  ┌──────────────────┐                                │
│  │  seith-telegram   │                                │
│  │  (Bot dispatcher) │                                │
│  └──────────────────┘                                │
└─────────────────────────────────────────────────────┘
```

### 4.6 Git Strategy

| Branch | Fungsi |
|--------|--------|
| `main` | Production-ready code, protected branch |
| `develop` | Integration branch untuk fitur baru |
| `feature/*` | Branch per fitur (e.g., `feature/xauusd-frama`) |
| `fix/*` | Bug fix branches |
| `release/*` | Release preparation branches |

**Conventional Commits:**
```
feat(XAUUSD): add FRAMA indicator
fix(shared): correct SQLite connection pool
docs(docker): add Docker blueprint as backup infrastructure
test(EURUSD): add basic strategy test
```

**Pre-commit Hook:**
```bash
cargo fmt --check
cargo clippy -- -D warnings
cargo test
```

### 4.7 CI/CD Pipeline (GitHub Actions)

#### Workflow: `ci.yml` — Build + Test + Lint

```yaml
name: CI

on:
  push:
    branches: [main, develop]
  pull_request:
    branches: [main, develop]

env:
  CARGO_TERM_COLOR: always
  RUST_TOOLCHAIN: stable

jobs:
  check:
    name: Code Quality
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4

      - name: Install Rust toolchain
        uses: dtolnay/rust-toolchain@stable
        with:
          components: rustfmt, clippy

      - name: Cache cargo
        uses: actions/cache@v4
        with:
          path: |
            ~/.cargo/registry
            ~/.cargo/git
            target
          key: ${{ runner.os }}-cargo-${{ hashFiles('**/Cargo.lock') }}

      - name: Check formatting
        run: cargo fmt --check

      - name: Run clippy
        run: cargo clippy --workspace --all-targets -- -D warnings

  test:
    name: Unit & Integration Tests
    runs-on: ubuntu-latest
    needs: check
    steps:
      - uses: actions/checkout@v4

      - name: Install Rust toolchain
        uses: dtolnay/rust-toolchain@stable

      - name: Cache cargo
        uses: actions/cache@v4
        with:
          path: |
            ~/.cargo/registry
            ~/.cargo/git
            target
          key: ${{ runner.os }}-cargo-${{ hashFiles('**/Cargo.lock') }}

      - name: Run unit tests
        run: cargo test --workspace --lib

      - name: Run integration tests
        run: cargo test --workspace --test '*'

      - name: Run with all features
        run: cargo test --workspace --all-features

  build:
    name: Build Release
    runs-on: ubuntu-latest
    needs: test
    steps:
      - uses: actions/checkout@v4

      - name: Install Rust toolchain
        uses: dtolnay/rust-toolchain@stable

      - name: Cache cargo
        uses: actions/cache@v4
        with:
          path: |
            ~/.cargo/registry
            ~/.cargo/git
            target
          key: ${{ runner.os }}-cargo-${{ hashFiles('**/Cargo.lock') }}

      - name: Build release binary
        run: cargo build --workspace --release

      - name: Upload binary artifact
        uses: actions/upload-artifact@v4
        with:
          name: seith-binary-${{ github.sha }}
          path: target/release/seith
          retention-days: 7
```

#### Workflow: `release.yml` — Build + Tag Release

```yaml
name: Release

on:
  push:
    tags:
      - 'v*'

env:
  CARGO_TERM_COLOR: always

jobs:
  release:
    name: Build & Release
    runs-on: ubuntu-latest
    permissions:
      contents: write
    steps:
      - uses: actions/checkout@v4

      - name: Install Rust toolchain
        uses: dtolnay/rust-toolchain@stable

      - name: Cache cargo
        uses: actions/cache@v4
        with:
          path: |
            ~/.cargo/registry
            ~/.cargo/git
            target
          key: ${{ runner.os }}-cargo-${{ hashFiles('**/Cargo.lock') }}

      - name: Build release binary
        run: cargo build --workspace --release

      - name: Create GitHub Release
        uses: softprops/action-gh-release@v2
        with:
          files: |
            target/release/seith
          generate_release_notes: true
          draft: false
          prerelease: false
```

#### Workflow: `docker.yml` — Build + Push Docker Image (BLUEPRINT — Future Linux Deployment Only)

```yaml
name: Docker

on:
  push:
    branches: [main]
    tags: ['v*']

env:
  REGISTRY: ghcr.io
  IMAGE_NAME: ${{ github.repository }}

jobs:
  build-and-push:
    name: Build & Push
    runs-on: ubuntu-latest
    permissions:
      contents: read
      packages: write
    steps:
      - uses: actions/checkout@v4

      - name: Log in to Container Registry
        uses: docker/login-action@v3
        with:
          registry: ${{ env.REGISTRY }}
          username: ${{ github.actor }}
          password: ${{ secrets.GITHUB_TOKEN }}

      - name: Extract metadata
        id: meta
        uses: docker/metadata-action@v5
        with:
          images: ${{ env.REGISTRY }}/${{ env.IMAGE_NAME }}
          tags: |
            type=ref,event=branch
            type=semver,pattern={{version}}
            type=sha

      - name: Build and push Docker image
        uses: docker/build-push-action@v5
        with:
          context: .
          push: true
          tags: ${{ steps.meta.outputs.tags }}
          labels: ${{ steps.meta.outputs.labels }}
          cache-from: type=gha
          cache-to: type=gha,mode=max
```

### 4.8 CI/CD Flow Diagram

```
Push/PR to main/develop
        │
        ▼
┌──────────────────┐
│   ci.yml          │
│                   │
│  1. cargo fmt     │──── Fail → Block PR
│  2. cargo clippy  │──── Fail → Block PR
│  3. cargo test    │──── Fail → Block PR
│  4. cargo build   │──── Fail → Block PR
│                   │
│  All Pass → ✓     │
└────────┬─────────┘
         │
         ▼
   Merge to main
         │
         ▼
┌──────────────────┐
│   docker.yml      │
│                   │
│  1. Build image   │
│  2. Push to GHCR  │
│  3. Tag: sha/br   │
└──────────────────┘

Tag v* pushed
        │
        ▼
┌──────────────────┐
│   release.yml     │
│                   │
│  1. Build binary  │
│  2. Create GitHub │
│     Release       │
│  3. Attach binary │
└──────────────────┘
```

### 4.9 CI/CD Requirements

| Requirement | Detail |
|-------------|--------|
| **GitHub Secrets** | `GITHUB_TOKEN` (otomatis untuk container registry) |
| **Runner** | `ubuntu-latest` (build + test), opsional `self-hosted` untuk MT5 test |
| **Caching** | Cargo registry + target dir di-cache per branch |
| **Artifacts** | Binary release disimpan 7 hari |
| **Container Registry** | GitHub Container Registry (ghcr.io) |
| **Release Trigger** | Push tag `v*` (e.g., `v1.0.0`) |
| **Branch Protection** | `main` branch: require CI pass sebelum merge |

### 4.10 Kenapa Arsitektur Ini?

| Prinsip | Penjelasan |
|---------|------------|
| **Rust Core** | Semua logic trading berjalan di Rust — performa tinggi, memory safety, determinism |
| **Python Bridge** | Hanya untuk library yang tidak tersedia di Rust (MT5, Telegram, scraping) |
| **Jupyter for Stats** | Analisis performa dan R&D dilakukan di notebook interaktif |
| **Shared First** | MT5, Telegram, database, utils → dipakai semua instrumen |
| **Instrument Isolation** | XAUUSD, EURUSD, BTCUSD → masing-masing punya sistem sendiri |
| **Docker (Blueprint)** | Cetak biru infrastruktur cadangan — semua berjalan NATIVE di host OS lokal |
| **Git Discipline** | Conventional commits + pre-commit hooks → codebase bersih |

---

*PRD ini merupakan cetak arsitektur AI SEITH berdasarkan WORKFLOW XAUUSD.mmd*
