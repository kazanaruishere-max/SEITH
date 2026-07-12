# PRD — AI SEITH US100
**Autonomous US Tech 100 (Nasdaq) Trading Intelligence Module**
**v1.5 | 2026-07-12**

---

## 1. TARGET PROJECT

### 1.1 Target Utama

| No | Metric | Target | Threshold Minimum | Keterangan |
|----|--------|--------|-------------------|------------|
| 1 | **Maximum Drawdown** | ≤ 6% | ≤ 10% | Dari peak equity, hard limit 12% |
| 2 | **Consecutive Win** | ≥ 9 | ≥ 6 | Trades profit berturut-turut |
| 3 | **Consecutive Loss** | ≤ 3 | ≤ 4 | Trades rugi berturut-turut, auto-halt jika ≥ 5 |
| 4 | **Win Rate** | ≥ 80% | ≥ 70% | Dari total trade yang tereksekusi |
| 5 | **Recovery Factor** | ≥ 4.0 | ≥ 3.0 | Net Profit ÷ Maximum Drawdown |
| 6 | **Profit Factor** | ≥ 4.0 | ≥ 3.8 | Gross Profit ÷ Gross Loss |

### 1.2 Target Pendukung

| No | Metric | Target | Keterangan |
|----|--------|--------|------------|
| 7 | Average Win : Average Loss | ≥ 1.2 : 1 | Sniper mode (MORNING/AFTERNOON) |
| 8 | Scalp Win : Average Loss | ≥ 0.5 : 1 | Scalp mode (POWER HOUR / Crisis) |
| 9 | Expectancy per Trade | ≥ +0.3x Risk | Across all modes |
| 10 | Trade Frequency | 1-2 trades sniper + 0-2 scalp | Maks 4 trade/hari aktif |
| 11 | Average Hold Sniper | 30-240 menit | Hold lebih lama menunggu TP |
| 12 | Average Hold Scalp | 5-30 menit | Keluar cepat, profit kecil |

### 1.3 Risk Hard Limits

| Parameter | Nilai | Aksi |
|-----------|-------|------|
| Max Risk per Trade | 0.75% equity | Tidak ternegosiasi |
| Max Daily Loss | 2.5% equity | Auto-halt trading hari itu |
| Max Weekly Loss | 5.0% equity | Pause 48 jam |
| Max Open Position | 1 | Single position only |
| Spread Tolerance | ≤ 1.5 points | Reject entry jika lebih |

---

## 2. ARSITEKTUR US100

### 2.1 Architecture Overview

```
+-----------------------------------------------------------------------+
|                    LEVEL 0 — INFRASTRUKTUR                            |
|                                                                       |
|  +--------------+  +---------------+  +----------------------------+   |
|  | OANDA API    |  | OANDA Broker  |  | SQLite Database            |   |
|  | v20 REST     |  | Data Feed     |  | (Histori + State)          |   |
|  +------+-------+  +-------+-------+  +------------+---------------+   |
|         +----------+-------+------------------------+                   |
|                      |                                                  |
|          +-----------+-----------+                                      |
|          | Session Filter (US)   |                                      |
|          | + Macro Calendar     |                                      |
|          | + Data Normalizer     |                                      |
|          +-----------+-----------+                                      |
+----------------------+--------------------------------------------------+
                       |
+-----------------------------------------------------------------------+
|                    LEVEL 3 — MASTER CONTROL                            |
|                                                                       |
|          +---------------------------+                                 |
|          | US100 Event Loop          |  <---------------------+      |
|          | (M1/M15, US Hours)        |                        |      |
|          +------------+--------------+                        |      |
|                       |                                       |      |
|          +------------+--------------+                        |      |
|          | Adaptive State Manager    |                        |      |
|          | (skip_strike_count)       |                        |      |
|          +------------+--------------+                        |      |
|                       |                                       |      |
|          +------------+--------------+                        |      |
|          | 5-Gate Indicator Pipeline |                        |      |
|          | Macro -> HV -> FRAMA ->    |                        |      |
|          | Orderflow -> VWAP + Yield  |                        |      |
|          +------------+--------------+                        |      |
|                       |                                       |      |
|          +------------+--------------+                        |      |
|          | Statistical Brain         |                        |      |
|          | + Anti-Paralysis          |                        |      |
|          +------------+--------------+                        |      |
+----------------------+--------------------------------------------------+
                       |
+-----------------------------------------------------------------------+
|                    EXECUTION LAYER                                    |
|                                                                       |
|  +--------------+  +--------------+  +---------------------------+    |
|  | Limit Order  |  | Stop Order   |  | Market Entry             |    |
|  | (Sniper)     |  | (Momentum)   |  | (Breakout Confirm)       |    |
|  +------+-------+  +------+-------+  +------------+--------------+    |
|         +-------+--------+------------+                               |
|                  |                                                     |
|          +-------+-------+                                             |
|          | OANDA API     |                                             |
|          | Execution     |                                             |
|          +---------------+                                             |
+----------------------+--------------------------------------------------+
                       |
+-----------------------------------------------------------------------+
|              CLOSED-LOOP SELF-LEARNING ENGINE                          |
|                                                                       |
|  +-------------+  +------------+  +---------------------------+       |
|  | Post-Trade  |->| SQLite     |->| Cognitive Re-Calibration  |       |
|  | Extract     |  | Write      |  | (Slippage, Counter, Map)  |       |
|  +------+------+  +------------+  +-------------+-------------+       |
|                                                |                        |
|                                                v                        |
|          +---------------------------+                                  |
|          | Telegram Dispatcher      |                                  |
|          | (Chart + Signal + P/L)   |                                  |
|          +------------+--------------+                                  |
|                       |                                                 |
|                       v                                                 |
|          +---------------------------+                                  |
|          | Loop End -> Reset ->      |                                  |
|          | Back to Event Loop        |--------------------------------+
|          +---------------------------+
+-----------------------------------------------------------------------+
```

### 2.2 Level Descriptions

#### LEVEL 0 — Core Environment & Infrastructure

| Komponen | Fungsi |
|----------|--------|
| **Day Filter** | Deteksi weekend (Sabtu-Minggu) dan hari libur AS. Skip total saat market tutup |
| **Data Feed Raw** | Streaming harga US100.cash real-time dari OANDA via REST API v20. + **HV Z-Score** dihitung real-time dari rolling returns US100 (0 latensi). Auto-reconnect: retry 3x, standby 60s, Telegram alert jika gagal total |
| **Session Filter** | Filter waktu trading US Market (9:30-16:00 ET / 14:30-21:00 UTC). + **Deteksi 4 phase:** OPEN (skip 30m), NORMAL, LUNCH (block all), CLOSE (block all). + **Gap flag:** skip 30m pertama jika gap > 0.3%. **Short-circuit:** Jika ada posisi terbuka, skip 5-gate pipeline, langsung lompat ke monitor. |
| **Macro Calendar Filter** | Deteksi FOMC, FOMC Minutes, CPI, NFP, GDP, ISM PMI, PPI + **earnings window** (AAPL/MSFT/NVDA/AMZN/GOOGL/META). Klasifikasi RED (no-trade), ORANGE (50% lot), WARNING (earnings -> 50% lot). Data dari ForexFactory scraper |
| **Data Normalizer** | Normalisasi harga indeks 2 digit desimal |
| **SQLite Database** | Penyimpanan histori trading, state sistem, counter adaptasi, yield history |
| **OANDA API** | Eksekusi order, pengambilan data OHLCV, DOM, US10YB yield |

#### LEVEL 3 — Master Control Pipeline & Event Loop

| Komponen | Fungsi |
|----------|--------|
| **US100 Event Loop** | Event-driven loop aktif setiap penutupan lilin M1/M15, hanya jalan saat US Market Hours (14:30-21:00 UTC) |
| **Adaptive State Manager** | Membaca `skip_strike_count` dari SQLite sebelum setiap iterasi |
| **5-Gate Indicator Pipeline** | Macro -> HV Z-Score -> FRAMA -> Orderflow -> VWAP+Yield |

#### 5-Gate Indicator Pipeline

Pipeline sequential 5-layer. Session phase + earnings sudah di-filter di L0.

```
PRE-TRADE GATE (eksekusi setiap M1):

  GATE 0 — Macro Calendar Filter
    FOMC / FOMC Minutes -> RED -> NO-TRADE ZONE T-2 jam hingga rilis
    CPI / NFP / GDP -> RED -> NO-TRADE ZONE T-2 jam hingga rilis
    PPI / ISM PMI / Retail Sales -> ORANGE -> kurangi lot 50%
    Earnings Window (AAPL/MSFT/NVDA/AMZN/GOOGL/META) -> WARNING -> lot 50%
    -> RED -> SKIP semua
    -> ORANGE / WARNING -> kurangi lot, lanjut
    -> GREEN -> lanjut

    GATE 1 — HV Z-Score (Market Regime, Real-time)
    Hitung dari rolling std dev returns US100 (window 10-20 bar M1/M15)
    HV Z-Score < -1.0 -> volatilitas sangat rendah, false breakout risk -> SKIP
    HV Z-Score -1.0 s.d +1.5 -> volatilitas normal, sweet spot -> lanjut
    HV Z-Score +1.5 s.d +2.0 -> volatilitas meningkat, masih manageable -> lanjut (confidence turun)
    HV Z-Score > +2.0 -> volatilitas ekstrem, reversal risk -> SKIP

    Catatan: VIX dari yfinance dipakai setiap pagi untuk kalibrasi threshold harian. Jika fetch gagal, gunakan baseline default 18.0 (fallback logic).


  GATE 2 — FRAMA Trend (Direction)
    Z_FRAMA <= 0.5 -> pullback valid dalam trend -> lanjut
    Z_FRAMA > 0.5 -> overextended, FOMO risk -> BLOCK

  GATE 3 — Orderflow (Institutional Confirmation)
    OFS = S_Delta + S_CVD + S_DOM
    OFS >= +3 / <= -3 -> institusional kuat -> PASS
    OFS >= +2 / <= -2 (POWER HOUR) -> PASS
    OFS -1 s/d +1 -> retail noise / tidak yakin -> BLOCK

    Absorption Detection:
      Limit order besar termakan tanpa price reversal?
      -> YES -> konfirmasi breakout arah absorpsi -> PASS
      -> NO -> normal

  GATE 4 — VWAP Bands + Yield (Level & Macro)
    VWAP:
      Di luar +-2.5 band -> overextended -> BLOCK
      Dalam band, jauh dari POC -> FAIR -> PASS

    Yield (US10Y Z-Score + Curve Spread):
      Yield Z > +1.5 -> CONFIRM sell / BLOCK buy (bond selloff, tech bearish)
      Yield Z < -1.5 -> CONFIRM buy / BLOCK sell (bond rally, tech bullish)
      Curve inverted (spread < 0) -> preferensi sell
      Curve normal (spread > 0) -> preferensi buy
      **Data kosong/rollover -> skip yield check, return NEUTRAL (tidak block, tidak bonus)**

  Jika 5/5 lolos -> SIGNAL VALID -> Eksekusi
  Jika < 5/5 -> SKIP (tunggu setup berikutnya, kecuali Crisis mode OFS relax)
```

#### Execution Layer — Hybrid RR: Sniper (Primary) + Scalp (Darurat)

Sistem menggunakan **2 mode RR** tergantung session phase dan kondisi:

| Mode | Session | RR | SL | Hold Time | Risk/Trade |
|------|---------|----|----|-----------|------------|
| **Sniper** | NORMAL | 1:1.2 - 1:1.5 | FRAMA / Support | 30-240 min | 0.75% equity |
| **Scalp** | POWER HOUR / Crisis | 1:0.5 - 1:0.7 | 5-10 points | 5-30 min | 0.50% equity |

Threshold berbeda per session phase. 4 phase: OPEN, NORMAL, LUNCH, CLOSE. POWER HOUR jadi sub-mode di NORMAL.

| Fase | Waktu | OFS Min | VWAP Band | Max Lot | Mode | RR |
|------|-------|---------|-----------|---------|------|----|
| OPEN | 14:30-15:00 | SKIP | SKIP | 0 | Stabilisasi (gap skip 30m) | - |
| NORMAL | 15:00-16:30 | 3 | 2.5 | 100% | **SNIPER** | **1:1.5** |
| LUNCH | 16:30-18:00 | SKIP | SKIP | 0 | Block semua sinyal | - |
| NORMAL | 18:00-19:30 | 3 | 2.5 | 100% | **SNIPER** | **1:1.5** |
| POWER HOUR | 19:30-20:30 | 2 | 2.0 | 75% | **SCALP** | **1:0.5** |
| CLOSE | 20:30-21:00 | SKIP | SKIP | 0 | Block semua sinyal | - |

| Mode | Prioritas | Session | Trigger | Order Type |
|------|-----------|---------|---------|------------|
| **Stop Order (Sniper)** | **PRIMARY** | NORMAL | Breakout konfirmasi orderflow OFS >= 3 | BUY_STOP / SELL_STOP di luar range konsolidasi M1 |
| **Limit Order (Sniper)** | Secondary | NORMAL | Pullback ke FRAMA saat OFS >= 3 | BUY_LIMIT / SELL_LIMIT di support + buffer |
| **Market Entry (Scalp)** | **POWER HOUR** | POWER HOUR / Crisis | OFS >= 2 + momentum kuat | Market Order, TP 8-15pts, SL tight |
| **Emergency** | Crisis skip >= 3 | Crisis Adaptation | Paksa entry mode SCALP dengan OFS relax | Market Order, scalping RR |

Alur pemilihan mode + lot sizing:

```
Signal 5/5 lolos?
  |
  +-> Phase = NORMAL?
  |     -> MODE = SNIPER
  |     -> BaseUnit = Equity x 0.0075 / (SL x PointValue)
  |     -> Lot = BaseUnit x ConfidenceMult x 1.0 x MacroMult
  |     -> Stop/Limit order, RR 1:1.5, SL di FRAMA
  |
  +-> Phase = POWER HOUR?
  |     -> MODE = SCALP
  |     -> BaseUnit = Equity x 0.0050 / (SL x PointValue)
  |     -> Lot = BaseUnit x ConfidenceMult x 0.67 x MacroMult
  |     -> Market/Stop order, RR 1:0.5, SL 5-10pts, TP 8-15pts
  |
  +-> skip_count >= 3 (Crisis)?
        -> MODE = SCALP (darurat)
        -> BaseUnit = Equity x 0.0050 / (SL x PointValue)
        -> Lot = BaseUnit x 0.5 x 0.67 x 1.0
        -> Relax OFS ke 2, exit cepat, risk 0.5% (crisis, gate lain tetap jalan)
```

#### Closed-Loop Self-Learning

```
Tahap 1: Auto-Kill -> hapus semua pending order (T+5 menit market close)
Tahap 2: Extract -> Win/Loss, Slippage, Spread, P/L
Tahap 3: SQLite Write -> simpan baris baru hasil eksekusi
Tahap 4: Rekalibrasi (sniper: immediate. scalp: trade1 immediate, trade2+ batch 2. slippage darurat -> force immediate) -> optimasi buffer, reset counter, update win probability map
Tahap 5: Telegram Report -> kirim laporan P/L akun riil
Tahap 6: Loop -> kembali ke Event Loop awal
```

### 2.3 Data Flow

```
Tick Baru (M1/M15 close)
    |
    +-> Level 0:
    |     Validasi day filter (weekend/holiday)
    |     Validasi session US Market + phase detection
    |     Cek Macro Calendar + Earnings filter
    |     **Hitung HV Z-Score dari rolling returns US100 (real-time)**
    |     **Fetch VIX via yfinance untuk kalibrasi threshold harian (1x per session)**
    |     Fetch US10Y + US02Y yield via OANDA
    |     Normalisasi harga 2 digit
    |
    +-> Level 3: Baca skip_strike_count (klasifikasi: hanya "signal reject" skip yang increment)
    |
    +-> 5-Gate Pipeline:
    |     0. Macro Filter  -> RED? -> SKIP. ORANGE/Earnings? -> kurangi lot
    |     1. **HV Z-Score** -> regime detection (real-time dari US100 price)
    |     2. FRAMA         -> trend & pullback
    |     3. Orderflow     -> OFS (Delta+CVD+DOM) + Absorption
    |     4. VWAP + Yield  -> level fair + yield Z-Score + curve spread
    |
    +-> Mode Decision: SNIPER (NORMAL) atau SCALP (POWER HOUR/Crisis)
    |
    +-> Lot Sizing Hitung: BaseUnit x ConfidenceMult x ModeMult x MacroMult (OANDA balance real-time)
    |
    +-> Statistical Synthesis: Entry, TP, SL (sesuai mode RR + lot)
    |
    +-> Execution: OANDA API (Stop/Limit untuk Sniper, Market untuk Scalp)
    |     Jika gagal: retry 1x. Jika tetap gagal: log error, skip rekalibrasi
    |
    +-> if execution success -> Telegram: Chart + sinyal
    |   if failed -> Telegram: Error alert
```

### 2.4 State Machine

```
[BOOT] -> [OPEN] -> (30m) -> [NORMAL] <----------+
    |     MODE=PHASE_DETECT     | MODE=SNIPER     |
    |                  +-> [LUNCH] -> (90m)       |
    |                  |    MODE=NO TRADE          |
    |                  |          |               |
    |                  +----------+-> [NORMAL]    |
    |                              | MODE=SNIPER  |
    |                              +-> [POWER H] |
    |                                   MODE=SCALP|
    |                                   |         |
    |                     +-- 21:00 UTC           |
    |                     v                       |
    |               [FORCE_CLOSE]                 |
    |                     |                        |
    |                     v                        |
    |               [SESSION_CLOSED] -> Standby    |
    |                                                                       |
    +- Macro Event (RED) -> [NO_TRADE]                                      |
    |      -> Auto-resume T+2 jam post-event                                |
    |                                                                       |
    +- Post-Trade -> Rekalibrasi -> ----------------------------------------+
    |                                                                       |
    +- Skip >= 3 -> [CRISIS_ADAPTATION]                                     |
    |      -> Relax OFS ke 2 (bukan semua gate)                             |
    |      -> MODE = SCALP (darurat, RR 1:0.5)                              |
    |                                                                       |
    +- Skip >= 5 -> [CRISIS_CEILING]                                        |
    |      -> Force reset skip_count = 0, standby 1 session                 |
    |      -> Telegram alert                                                 |
    |                                                                       |
    +- Phase berubah -> [phase berikutnya]
    |
    +- Weekend / US Holiday -> [DAY_OFF] -> standby total
```

---

## 3. PERBEDAAN DENGAN XAUUSD

| Aspek | XAUUSD | US100 |
|-------|--------|-------|
| **Broker** | Exness (MT5) | **OANDA** (REST API v20) |
| **Symbol** | XAUUSDm | US100.cash |
| **Sesi Trading** | 24/5 | US Market Hours (14:30-21:00 UTC) |
| **Desimal** | 3 digit | 2 digit |
| **Vol Index** | GVZ (Gold VIX) | **HV Z-Score** dari US100 price (real-time) + VIX yfinance kalibrasi harian |
| **Market Sifat** | Mean-reverting | **Trending** |
| **Macro Filter** | Tidak ada | **Ada** (FOMC/CPI/NFP no-trade zone) |
| **Yield** | Tidak ada | **Ada** (US10Y Z-Score confirmation) |
| **Filters** | 4 (Bayesian, CVaR, Compass, OFS) | **5-Gate Pipeline** (Macro->HV->FRAMA->Orderflow->VWAP+Yield) |
| **Gate Strictness** | OFS >= 2 | **OFS >= 3** (lebih ketat utk WR 80%+) |
| **DOM** | Exness terbatas | **OANDA full DOM** |
| **Prior Probability** | OANDA sentiment | **HV Z-Score** (volatility-based, real-time dari US100 price) |
| **Execution API** | MT5 (PyO3) | **OANDA v20** (PyO3) |

---

## 4. TECHNOLOGY STACK

### 4.1 Stack Overview

| Layer | Bahasa | Kegunaan |
|-------|--------|----------|
| **Core Logic** | **Rust** | Semua engine, execution, indicators, risk management, database |
| **Library Bridge** | **Python** | OANDA API wrapper, Telegram Bot API, mplfinance charting |
| **Statistical Modeling** | **Jupyter Notebook** | Backtesting analysis, recalibration R&D |
| **Interface** | **Terminal (CLI)** | **TIDAK ADA** web server, TUI, atau GUI |

### 4.2 Rust Crates (Core)

| Crate | Versi | Kegunaan |
|-------|-------|----------|
| `tokio` | 1.x | Async runtime, event loop |
| `sqlx` | 0.7 | Async SQLite pool (shared via workspace) |
| `pyo3` | 0.20 | Python FFI bridge untuk OANDA API |
| `anyhow` | 1.x | Error propagation |
| `thiserror` | 2.x | Custom error types |
| `chrono` | 0.4 | Timezone (US/Eastern), session check |
| `serde` / `serde_json` | 1.x | OANDA API response parsing |
| `reqwest` | 0.12 | HTTP client (cadangan OANDA REST fallback) |
| `log` / `env_logger` | - | Logging |
| `dotenvy` | 0.15 | Load `.env` |
| `uuid` | 1.x | Unique trade ID |
| `polars` | 0.43 | OHLCV processing |
| `statrs` | 0.17 | Statistical distributions, Z-Score |

### 4.3 Python Libraries (Bridge)

| Library | Fungsi |
|---------|--------|
| `oandapy` / `requests` | OANDA REST API v20 |
| `python-telegram-bot` | Telegram Bot API |
| `yfinance` | VIX kalibrasi harian (bukan real-time), `^VIX` symbol |
| `mplfinance` | Chart generation |
| `pandas` | Data manipulation |

### 4.4 Indikator US100

> **TIDAK ADA:** RSI, ATR, BB, MACD, Stochastic, atau indikator tradisional.
> **TIDAK ADA:** Bayesian Gatekeeper, CVaR Risk Engine.

**YANG ADA:**
- **HV Z-Score** -- Historical Volatility Z-Score (Market Regime Detector) dihitung real-time dari rolling std dev returns US100. 0 latensi.
- **VIX (kalibrasi harian)** -- Download via yfinance setiap pagi untuk set threshold bias, bukan real-time.
- **US10Y Yield Z-Score + Curve Spread** -- 10Y Z-Score + 10Y-2Y spread (1 indicator, 2 dimensi) via OANDA US10YB + US02YB
- **FRAMA** -- Flexible Moving Average (Trend Rider M15)
- **AMT Volume Profile** -- POC, VAH, VAL, Magnet Zone
- **VWAP Deviation Bands** -- Session VWAP +-2.5 bands (threshold berbeda per phase)
- **Orderflow (OFS)** -- 3 komponen:
  - _S_Delta_ -- Single bar buy/sell aggressor delta
  - _S_CVD_ -- Cumulative volume delta divergence
  - _S_DOM_ -- DOM imbalance Z-Score + absorption detection (internal)

---

## 5. KERANGKA FOLDER

### 5.1 Struktur Folder US100

```
indices/
+-- US100/                              # US Tech 100 -- OANDA (5-GATE PIPELINE)
    |
    +-- Cargo.toml                      #     Crate config untuk US100 module
    |
    +-- config/
    |   +-- mod.rs                      #     Module declaration
    |   +-- settings.rs                 #     Session hours, risk limits, symbol config
    |   +-- thresholds.rs              #     HV, FRAMA, OFS, VWAP, Yield thresholds
    |
    +-- core/
    |   +-- mod.rs                      #     Module declaration
    |   |
    |   +-- l0_infra/                   #     Level 0 -- Data & Infrastructure
    |   |   +-- mod.rs
    |   |   +-- data_feed.rs            #       OANDA streaming price + OHLCV via PyO3
    |   |   +-- normalizer.rs           #       Normalisasi 2 digit desimal
    |   |   +-- session_filter.rs       #       US Market Hours + 4 phase detection + gap flag
    |   |   +-- macro_filter.rs         #       FOMC/CPI/NFP/GDP/PPI/ISM + earnings (all-in-one)
    |   |
    |   +-- l3_engine/                  #     Level 3 -- Master Control
    |   |   +-- mod.rs
    |   |   +-- event_loop.rs           #       Tokio loop 15s, M1/M15 tick, US hours aware
    |   |   +-- state_manager.rs        #       State: US_MARKET, NO_TRADE, SESSION_CLOSED, CRISIS
    |   |   +-- statistical_brain.rs    #       Statistical synthesis, entry/TP/SL calculation
    |   |   +-- anti_paralysis.rs       #       Skip strike counter, threshold relaxation
    |   |
    |   +-- l1_pipeline/               #     Level 1 -- 5-Gate Indicator Pipeline (Macro->HV->FRAMA->OFS->VWAP+Yield)
    |   |   +-- mod.rs
    |   |   +-- macro_gate.rs          #       Gate 0: Macro calendar + earnings filter
    |   |   +-- hv_compass.rs           #       Gate 1: HV Z-Score regime detection (real-time)
    |   |   +-- pipeline_router.rs     #       Gate 2-4: FRAMA -> Orderflow -> VWAP + Yield
    |   |   +-- signal_classifier.rs   #       Final: 5/5 lolos?
    |   |
    |   +-- execution/                  #     Execution Layer
    |       +-- mod.rs
    |       +-- limit_order.rs          #       Limit order (sniper)
    |       +-- stop_order.rs           #       Stop order (momentum)
    |       +-- market_entry.rs         #       Market entry (breakout confirmation)
    |       +-- order_manager.rs        #       Order routing, validation, lifecycle
    |       +-- risk_manager.rs         #       Risk limits, lot sizing hybrid multiplier, SL/TP
    |
    +-- signals/
    |   +-- mod.rs
    |   +-- signal_types.rs            #       Enum: BuySignal, SellSignal, NoSignal
    |   +-- signal_validator.rs        #       Validasi konfluensi sinyal 5 gate
    |   +-- signal_enricher.rs         #       Tambah metadata: confidence, regime, yield, ofs
    |
    +-- indicators/
    |   +-- mod.rs
    |   +-- vix.rs                      #       HV Z-Score (real-time dari US100 price) + VIX yfinance kalibrasi harian
    |   +-- yield.rs                    #       US10Y Z-Score + curve spread (2-in-1)
    |   +-- frama.rs                    #       Flexible Moving Average
    |   +-- amt_volume_profile.rs       #       Volume Profile POC/VAH/VAL
    |   +-- vwap_bands.rs              #       VWAP Deviation Bands +-2.5
    |   +-- orderflow/
    |   |   +-- mod.rs
    |   |   +-- s_delta.rs             #       Delta Score (single bar)
    |   |   +-- s_cvd.rs               #       CVD Divergence
    |   |   +-- s_dom.rs               #       DOM Heatmap Z-Score + absorption detection
    |
    +-- analytics/
    |   +-- mod.rs
    |   +-- trade_journal.rs           #       Post-trade logging
    |   +-- rekalibrasi.rs             #       Cognitive recalibration
    |   +-- win_probability_map.rs     #       Win probability map
    |   +-- auto_kill.rs               #       Auto-kill pending orders
    |
    +-- data/
    |   +-- mod.rs
    |   +-- schema.rs                  #       US100-specific SQLite schema
    |   +-- queries.rs                 #       SQL queries
    |
    +-- external/
        +-- mod.rs
        +-- oanda_feed.rs              #       OANDA bridge (PyO3 -> python/oanda.py)
        +-- calendar_scraper.rs        #       Macro calendar filter (PyO3 -> python/scraper.py)
```

### 5.2 Mapping Layer ke Folder

| Layer | Folder | Bahasa | Keterangan |
|-------|--------|--------|------------|
| **Infrastructure** | `l0_infra/` | Rust | OANDA feed, normalizer, session filter, macro filter |
| **Master Control** | `l3_engine/` | Rust | Event loop, state manager, statistical brain, anti-paralysis |
| **5-Gate Pipeline** | `l1_pipeline/` | Rust | Macro -> HV Z-Score -> FRAMA -> Orderflow -> VWAP+Yield |
| **Execution** | `execution/` | Rust | Limit/stop/market, risk manager, order manager |
| **Signals** | `signals/` | Rust | Signal types, validator, enricher |
| **Indicators** | `indicators/` | Rust | HV Z-Score (real-time), VIX (kalibrasi harian), Yield, FRAMA, AMT, VWAP, Orderflow (3 komponen) |
| **Analytics** | `analytics/` | Rust | Trade journal, rekalibrasi, auto-kill |
| **Data** | `data/` | Rust | Schema, queries |
| **External** | `external/` | Rust | OANDA bridge, calendar scraper bridge |

### 5.3 Dependency Flow

```
seith US100
  |
  +-> indices/US100/core/
        |
        +-- l0_infra/         -> shared/external/oanda_bridge.rs (-> python OANDA API)
        |                       shared/data/sqlite_manager.rs (sqlx async)
        |                       shared/utils/time_utils.rs (US/Eastern timezone)
        |                       python/seith_bridge/scraper.py (macro calendar)
        |
        +-- l3_engine/        -> shared/utils/time_utils.rs
        |                       shared/utils/math_utils.rs
        |                       l1_pipeline/
        |
        +-- l1_pipeline/      -> indicators/ (HV Z-Score, VIX kalibrasi, Yield, FRAMA, Orderflow, VWAP)
        |                       l0_infra/ (session_phases, macro_filter, gap_detector)
        |
        +-- execution/        -> shared/external/oanda_bridge.rs
                                config/thresholds.rs
```

---

## 6. BINARY ALIAS

**Command:**
```bash
seith US100
```

**Main.rs routing:**
```rust
"US100" => us100::run().await,
```

**Cargo.toml (root workspace -- tambah member):**
```toml
members = [
    # ... existing members ...
    "indices/US100",
]
```

---

## 7. WORKFLOW PIPELINE

```
[Start] -> Cek Hari: Weekend atau US Holiday?
    |     -> YES -> [DAY_OFF] standby penuh
    |     -> NO  -> lanjut

[US Market Open 14:30 UTC]
    |
    +-> L0: Inisialisasi OANDA feed + auto-reconnect
    |       Hitung HV Z-Score dari rolling returns US100 (real-time)
    |       Fetch VIX yfinance 1x untuk kalibrasi threshold harian
    |       Fetch US10Y + US02Y
    |       Cek Macro Calendar (merge earnings)
    |       Deteksi phase (OPEN/NORMAL/LUNCH/CLOSE) + gap flag
    |
    +-> L3: Event loop, phase-aware, skip classification
    |
    +-> 5-Gate Pipeline:
    |     0. Macro Gate  -> RED? -> SKIP / ORANGE? -> -50% lot
    |     1. HV Z-Score  -> real-time dari US100 price -> regime
    |     2. FRAMA       -> Trend valid? -> lanjut
    |     3. Orderflow   -> OFS >= 3? + Absorption? -> PASS
    |     4. VWAP+Yield  -> Level fair + yield Z + curve? -> PASS
    |
    +-> Mode Decision:
    |     NORMAL       -> SNIPER (RR 1:1.5, SL di FRAMA)
    |     POWER HOUR   -> SCALP (RR 1:0.5, SL tight)
    |     Crisis>=3    -> SCALP DARURAT (OFS relax ke 2)
    |
    +-> Lot Sizing: BaseUnit x ConfidenceMult x ModeMult x MacroMult
    |     Equity dari OANDA balance API
    |
    +-> Statistical Brain -> Entry/TP/SL/Lot
    +-> OANDA Execution -> Telegram post-exec
    +-> Self-Learning -> loop

Phase Progression:
  14:30 -> [OPEN] skip 30m -> [NORMAL] SNIPER
  16:30 -> [LUNCH] block -> [NORMAL] SNIPER
  19:30 -> [POWER HOUR] SCALP -> [CLOSE]
  21:00 -> [FORCE_CLOSE]
```

---

## 8. RISK MANAGEMENT

| Parameter | US100 | Alasan |
|-----------|-------|--------|
| **Max Risk/Trade (Sniper)** | 0.75% equity | NORMAL session |
| **Max Risk/Trade (Scalp)** | 0.50% equity | POWER HOUR / Crisis |
| **Daily Loss Limit** | 2.5% equity | Auto-halt trading hari itu |
| **Weekly Loss Limit** | 5.0% equity | Pause 48 jam |
| **Max Open Position** | 1 | Single position only |
| **SL Sniper** | Wajib | FRAMA / ATR, RR 1:1.2 minimal |
| **SL Scalp** | Wajib tight | 5-10 points, RR 1:0.5 |
| **TP Scalp** | Wajib | 8-15 points, exit cepat |
| **Spread Tolerance** | <= 1.5 points | Spread indeks biasanya ketat |
| **Force Close** | 21:00 UTC | Market close + posisi harus bersih |
| **Macro RED** | T-2 jam s/d rilis | FOMC, FOMC Minutes, CPI, NFP, GDP -> NO-TRADE |
| **Macro ORANGE** | Kurangi lot 50% | PPI, ISM PMI, Retail Sales |
| **Earnings Window** | Kurangi lot 50% | Hari earnings AAPL/MSFT/NVDA/AMZN/GOOGL/META |
| **Gap Open** | SKIP 30m pertama | Stabilisasi indikasi setelah gap |
| **Lunch Phase** | SKIP semua sinyal | 16:30-18:00 UTC, false breakout zone |
| **Power Hour** | OFS relax ke 2, VWAP relax ke 2.0 | 19:30-20:30 UTC, volume tertinggi |
| **Close Phase** | SKIP semua sinyal | 20:30-21:00 UTC, risk hold overnight |
| **Crisis Ceiling** | skip >= 5 -> force reset + standby | Telegram alert, 1 session off |
| **Weekend / Holiday** | DAY_OFF -> standby total | Sabtu, Minggu, hari libur AS |
| **Order Failed** | Retry 1x + log | Jika tetap gagal: Telegram error alert, tidak masuk rekalibrasi |
| **OANDA Reconnect** | 3x retry, 60s standby | Telegram alert jika disconnect total |

### 8.1 Lot Sizing Hybrid Multiplier

Lot dihitung dinamis berdasarkan formula:

```
Lot = BaseUnit x ConfidenceMult x ModeMult x MacroMult

BaseUnit = Equity x RiskPerTrade / (SL_Points x PointValue)
  Equity         = balance OANDA (real-time via API)
  RiskPerTrade   = 0.0075 (Sniper) / 0.0050 (Scalp)
  SL_Points      = jarak SL dalam points
  PointValue     = nilai per point (US100 = 1.0)

ConfidenceMult:
  5/5 gate pass       -> 1.0
  HV Z-Score 1.5-2.0  -> 0.75
  Crisis (skip 3-4)   -> 0.5 (OFS relax mode)

ModeMult:
  Sniper (NORMAL)     -> 1.0
  Scalp (POWER HOUR)  -> 0.67

MacroMult:
  Hijau (no event)    -> 1.0
  ORANGE / WARNING    -> 0.5
  RED                 -> 0 (no trade, sudah di-block)
```

Contoh: Equity $10,000, SL 10pt, 5/5 gate, Sniper, normal:
`Lot = (10000 x 0.0075) / (10 x 1) x 1.0 x 1.0 x 1.0 = 7.5 unit`

---

## 9. CATATAN ARSITEKTUR

### 9.1 Perubahan dari XAUUSD

1. **Tidak ada L2 News Engine** -- Macro diganti dengan calendar filter pre-trade: RED (no-trade) dan ORANGE (kurangi lot).
2. **Tidak ada Bayesian Gatekeeper** -- Prior probability diganti **HV Z-Score** (real-time dari US100 price). VIX yfinance untuk kalibrasi harian.
3. **Tidak ada CVaR Risk Engine** -- Price velocity untuk indeks trending kurang meaningful.
4. **5-Gate Pipeline bukan 4 Filter** -- Macro -> HV -> FRAMA -> Orderflow -> VWAP+Yield.
5. **OFS threshold 3 bukan 2** -- Lebih ketat untuk WR 80%+, relax ke 2 saat POWER HOUR.
6. **Hybrid RR: Sniper + Scalp** -- Sniper RR 1:1.5 di NORMAL. Scalp RR 1:0.5 di POWER HOUR / Crisis.
7. **Session 4 phase** -- OPEN/NORMAL/LUNCH/CLOSE. POWER HOUR sub-mode. Gap flag di session_filter.rs.
8. **Yield + Curve merged** -- 1 indicator `yield.rs` berisi Z-Score + curve spread.
9. **Orderflow 3 komponen** -- Delta + CVD + DOM (absorption detection internal di s_dom.rs).
10. **Macro + Earnings merged** -- 1 filter `macro_filter.rs` untuk RED/ORANGE/WARNING.
11. **Gap flag** -- Bukan module terpisah. Cukup boolean `gap_detected` di session_filter.rs.
12. **Body Ratio & Price Velocity dihapus** -- Mean-reversion indicator, tidak relevan untuk Nasdaq trending.
13. **DeltaAcc & DOM Depth dihapus** -- Multi-bar accumulation overkill untuk breakout scalping.
14. **OANDA API bukan MT5** -- Semua eksekusi via OANDA REST API v20.

### 9.2 Perubahan dari PRD v1

| Perubahan | v1 (sebelum) | v2 (sekarang) |
|-----------|-------------|---------------|
| **Trade Frequency** | 1-4 trade/hari | **1-2 trade/hari** |
| **Pipeline** | 4 layer (VIX->FRAMA->OFS->VWAP) | **5-Gate** (Macro->HV->FRAMA->Orderflow->VWAP+Yield) |
| **Session** | Satu filter | **4 phase** (OPEN/NORMAL/LUNCH/CLOSE) + gap flag |
| **Macro Events** | FOMC/CPI/NFP/GDP | + **FOMC Minutes, PPI, ISM PMI, Retail Sales, Earnings** (RED/ORANGE/WARNING) |
| **OFS Threshold** | >= 2 | **>= 3** (relax ke 2 di Power Hour) |
| **RR Strategy** | Fixed | **Hybrid: Sniper RR 1:1.5 (NORMAL) + Scalp RR 1:0.5 (POWER HOUR/Crisis)** |
| **Orderflow** | 3 komponen (Delta, CVD, DOM) | **3 komponen** (+ Absorption detect internal) |
| **Data Source** | OANDA US100 saja | OANDA US100 + **US10YB+US02YB** + **HV Z-Score (real-time)** + **VIX yfinance kalibrasi harian** + **Macro Calendar** |
| **Indicators** | 7 indikator | **6 indikator** (HV, Yield, FRAMA, AMT, VWAP, Orderflow) |
| **State** | 4 states | **8 states** (OPEN, NORMAL, LUNCH, POWER HOUR, CLOSE, NO_TRADE, CRISIS, DAY_OFF) |
| **Anti-paralysis** | Force entry, relax all gates | **Relax OFS only + mode SCALP** (NO force entry) |
| **Crisis Ceiling** | Tidak ada | **Skip >= 5 -> force reset + standby** |
| **Win Rate Target** | >= 65% | **>= 80%** |
| **Profit Factor** | >= 1.8 | **>= 4.0** |

### 9.3 Shared Crate Usage

```
shared/                          -> Digunakan US100?
+-- config/                      OK settings.rs (env loader)
+-- external/
|   +-- mt5_bridge.rs            TIDAK (US100 pakai OANDA)
|   +-- oanda_bridge.rs          OK (sentiment / bridge)
|   +-- telegram_bridge.rs       OK (Telegram dispatch)
|   +-- news_aggregator.rs       TIDAK (pakai scraper langsung)
+-- utils/
|   +-- time_utils.rs            OK (US/Eastern conversion)
|   +-- math_utils.rs            OK (Z-Score, probability)
|   +-- async_helpers.rs         OK (retry, timeout)
|   +-- exceptions.rs            OK (error types)
+-- data/
|   +-- sqlite_manager.rs        OK (database pool)
+-- analytics/
    +-- performance_tracker.rs   OK (DD, WR, PF)
    +-- report_generator.rs      OK (Telegram report)
```

---

## 10. IMPLEMENTASI BERTAHAP

| Fase | Isi | Deliverable |
|------|-----|-------------|
| **Fase 1** | Folder structure, Cargo.toml, config, mod.rs skeleton, stubs | `cargo build` pass |
| **Fase 2** | L0 Infra: data_feed, normalizer, session_filter, macro_filter + tests | L0 siap (28 files) |
| **Fase 3** | L3 Engine: event_loop, state_manager, anti_paralysis + tests | Event loop phase-aware + skip classification |
| **Fase 4** | Indicators: HV Z-Score, VIX (kalibrasi harian), Yield, FRAMA, AMT, VWAP + tests | 5 indikator siap |
| **Fase 5** | Indicators Orderflow: S_Delta, CVD, S_DOM (absorb internal) + tests | Orderflow 3 komponen siap |
| **Fase 6** | L1 Pipeline: macro_gate, hv_compass, pipeline_router, signal_classifier + tests | 5-Gate pipeline jalan |
| **Fase 7** | Execution + OANDA bridge: limit, stop, market, risk_manager + order failure + reconnect + tests | Order placement resilient |
| **Fase 8** | Analytics: trade_journal, rekalibrasi, auto_kill + tests | Self-learning loop (batch untuk scalp) |
| **Fase 9** | Signals, integration test, Telegram dispatch | Full cycle siap |
| **Fase 10** | Backtesting, parameter optimization | Parameter optimal |
| **Fase 11** | Live paper trading + monitoring | Running di demo |
