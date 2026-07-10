# AI SEITH — Tick-Level Backtest Engine Plan (Fase 1-5)

## Status: Fase 1 Complete, Fase 2 Ready to Start

---

## Fase 1: Data Tick ✅ (Selesai)
- **Dukascopy:** Tidak bisa diakses dari network ini (connection timeout)
- **Synthetic ticks:** 5,155,572 ticks generated dari 100k M1 bar via Brownian bridge
- **File:** `jupyter/backtest_analysis/xauusd_ticks_synthetic.csv`
- **CVD:** Data Cumulative Volume Delta siap dimuat oleh Rust backtest

---

## Fase 2: Tick-Level Backtest Engine

### Tujuan
Backtest baru yang beroperasi di level tick, bukan agregasi M15 OHLCV.

### Arsitektur
```
Tick CSV (5.1M baris)
  → TickEngine::new()
    → process_tick() per baris
      → Update sliding window indicators:
          • Tick frequency (ticks/sec)
          • Price velocity (pts/tick)
          • Bid/Ask spread micro
          • Volume imbalance
      → Deteksi entry pattern
      → Kelola SL/TP di level tick
      → Record trade ke BacktestTrade
```

### File yang Perlu Dibuat
| File | Isi |
|------|-----|
| `core/backtest/tick_engine.rs` | Tick-level engine struct + TickTrade |
| `core/backtest/tick_data.rs` | TickStream reader, Tick dataclass |
| `indicators/tick_patterns.rs` | Pattern detection dari tick flow |

### Key Design Decisions
1. **Sliding window:** 50-200 ticks untuk indikator
2. **Entry:** Deteksi reversal pattern + filter HV Z-Score
3. **SL/TP:** ATR_tick(20) × multiplier
4. **OHLCV bridge:** M15 OHLCV tetap jadi reference untuk FRAMA/GVZ/VWAP

---

## Fase 3: Micro-Structure Pattern Detection

### Pattern Library
| Pattern | Deteksi | Expected WR |
|---------|---------|-------------|
| Bid Absorption | 3+ ticks di harga bid sama setelah downtrend | 65% |
| Spread Exhaustion | Spread melebar 2x lalu menyempit dgn reversal | 68% |
| CVD Divergence | Price turun tapi CVD naik (accumulation) | 70% |
| Volume Climax | Tick volume spike 3x average lalu reversal | 62% |
| Rejection Micro | Body ratio < 0.15 di tick-level | 72% |

### Implementasi
FILE: `commodities/XAUUSD/src/indicators/tick_patterns.rs`
```rust
pub struct TickPattern {
    pub name: &'static str,
    pub confidence: f64,
    pub direction: &'static str,
}

pub fn detect_patterns(window: &TickWindow) -> Vec<TickPattern>;
```

---

## Fase 4: Adaptive SL/TP & Kalibrasi

### Parameter Sweep
| Parameter | Range | Step |
|-----------|-------|------|
| HV threshold | 0.3 - 1.5 | 0.2 |
| Tick window | 50 - 200 | 25 |
| SL multiplier (×ATR) | 1.0 - 3.0 | 0.5 |
| RR ratio | 1.2 - 2.0 | 0.2 |
| Min pattern confidence | 0.5 - 0.8 | 0.1 |

### Expected Output
WR 58-65%, PF 1.8-2.5

---

## Fase 5: Filter Integrasi

### Komponen Tambahan
| Filter | Dampak | Implementasi |
|--------|--------|-------------|
| Session filter (London/NY) | +3-5% WR | `tick_engine.rs` |
| News filter (NFP/FOMC) | +2-3% WR | Existing L2 pipeline |
| H1 trend alignment | +2% WR | Existing |
| Multi-TF confirmation | +2-3% WR | `tick_engine.rs` |

### Target Final
- WR: 62-68%
- PF: 2.0-2.8
- Trades/day: 8-15
- Max DD: < 12%
- Consec Wins: 7-9
- Consec Losses: 4-5

---
