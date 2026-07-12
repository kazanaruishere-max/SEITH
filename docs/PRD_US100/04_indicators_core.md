# Fase 4: Indicators — Core (HV, VIX, Yield, FRAMA, AMT, VWAP)

## Goal
5 indikator inti siap: HV Z-Score (real-time), VIX yfinance (kalibrasi harian), US10Y+US02Y Yield, FRAMA, AMT Volume Profile, VWAP Bands.

## Files Created
```
indices/US100/indicators/
├── mod.rs
├── vix.rs                    # HV Z-Score (real-time dari US100 price) + VIX yfinance kalibrasi harian
├── yield.rs                  # US10Y Z-Score + curve spread (2-in-1)
├── frama.rs                  # Flexible Moving Average
├── amt_volume_profile.rs     # Volume Profile POC/VAH/VAL
├── vwap_bands.rs             # VWAP Deviation Bands +-2.5
```

## PRD Reference

**Gate 1 — HV Z-Score (PRD §2.2 Pipeline):**
```
HV Z-Score < -1.0       -> volatilitas sangat rendah, false breakout risk -> SKIP
HV Z-Score -1.0 s.d +1.5 -> volatilitas normal, sweet spot -> lanjut
HV Z-Score +1.5 s.d +2.0 -> volatilitas meningkat, masih manageable -> lanjut (confidence turun)
HV Z-Score > +2.0       -> volatilitas ekstrem, reversal risk -> SKIP
```
Catatan: VIX yfinance untuk kalibrasi threshold harian. Jika gagal fetch, gunakan baseline 18.0.

**Gate 2 — FRAMA Trend (PRD §2.2 Pipeline):**
```
Z_FRAMA <= 0.5 -> pullback valid dalam trend -> lanjut
Z_FRAMA > 0.5  -> overextended, FOMO risk -> BLOCK
```

**Gate 4 — VWAP Bands + Yield (PRD §2.2 Pipeline):**
```
VWAP:
  Di luar +-2.5 band -> overextended -> BLOCK
  Dalam band, jauh dari POC -> FAIR -> PASS

Yield (US10Y Z-Score + Curve Spread):
  Yield Z > +1.5 -> CONFIRM sell / BLOCK buy (bond selloff, tech bearish)
  Yield Z < -1.5 -> CONFIRM buy / BLOCK sell (bond rally, tech bullish)
  Curve inverted (spread < 0) -> preferensi sell
  Curve normal (spread > 0) -> preferensi buy
  Data kosong/rollover -> skip yield check, return NEUTRAL
```

**Indicator List (PRD §4.4):**
- **HV Z-Score** — Historical Volatility Z-Score. Real-time dari rolling std dev returns US100. 0 latensi.
- **VIX (kalibrasi harian)** — Download via yfinance setiap pagi untuk set threshold bias.
- **US10Y Yield Z-Score + Curve Spread** — 10Y Z-Score + 10Y-2Y spread via OANDA US10YB + US02YB.
- **FRAMA** — Flexible Moving Average (Trend Rider M15).
- **AMT Volume Profile** — POC, VAH, VAL, Magnet Zone.
- **VWAP Deviation Bands** — Session VWAP +-2.5 bands (threshold berbeda per phase).

## Key Decisions
- HV Z-Score: compute dari rolling std dev returns US100 (window 10-20 bar). Pattern reuse dari XAUUSD `compute_hv_zscore()`.
- VIX: fetch 1x per session via yfinance `^VIX`. Fallback baseline 18.0 jika gagal.
- Yield: merge Z-Score + curve spread dalam 1 file `yield.rs`. Data via OANDA US10YB + US02YB.
- Yield empty/rollover: return NEUTRAL, tidak block.
- FRAMA: Flexible Moving Average. Z_FRAMA untuk deteksi overextended.
- AMT Volume Profile: POC, VAH, VAL, Magnet Zone.
- VWAP: Session VWAP. Band +-2.5 (relax ke 2.0 di POWER HOUR).

## Dependencies
- Phase 2 (L0 — data feed untuk harga)

## Acceptance Criteria
- HV Z-Score output valid untuk semua range (tidak ada dead-zone)
- VIX fetch + fallback berfungsi
- Yield kalkulasi Z-Score + curve spread benar
- Yield empty return NEUTRAL, tidak panic
- FRAMA Z_FRAMA <= 0.5 / > 0.5 akurat
- AMT Volume Profile hitung POC/VAH/VAL
- VWAP bands sesuai konfigurasi per phase
- Semua test lulus
