# Fase 5: Indicators — Orderflow (Delta, CVD, DOM)

## Goal
3 komponen orderflow siap: S_Delta, S_CVD, S_DOM (dengan absorption detection internal).

## Files Created
```
indices/US100/indicators/orderflow/
├── mod.rs
├── s_delta.rs             # Delta Score (single bar buy/sell aggressor)
├── s_cvd.rs               # CVD Divergence (cumulative volume delta)
├── s_dom.rs               # DOM Heatmap Z-Score + absorption detection
```

## PRD Reference

**Gate 3 — Orderflow (PRD §2.2 Pipeline):**
```
OFS = S_Delta + S_CVD + S_DOM
OFS >= +3 / <= -3 -> institusional kuat -> PASS
OFS >= +2 / <= -2 (POWER HOUR) -> PASS
OFS -1 s/d +1 -> retail noise / tidak yakin -> BLOCK

Absorption Detection:
  Limit order besar termakan tanpa price reversal?
  -> YES -> konfirmasi breakout arah absorpsi -> PASS
  -> NO -> normal
```

**OFS Threshold per Phase (PRD §2.2 Execution table):**
| Fase | OFS Min |
|------|---------|
| OPEN | SKIP |
| NORMAL | 3 |
| LUNCH | SKIP |
| NORMAL | 3 |
| POWER HOUR | 2 (relax) |
| CLOSE | SKIP |

**Crisis Mode (PRD §2.4):**
- skip ≥ 3: relax OFS 3→2, SCALP mode
- Gate lain TETAP (HV/FRAMA/VWAP/Yield tidak diubah)

## Key Decisions
- S_Delta: buy volume - sell volume untuk 1 bar M1.
- S_CVD: cumulative delta divergence — tren harga vs tren delta.
- S_DOM: DOM imbalance Z-Score + absorption detection (internal di s_dom.rs).
- OFS formula: `S_Delta + S_CVD + S_DOM`, range -5 s.d +5 (expected).
- Signed: OFS positif = bullish, OFS negatif = bearish. Gate 3 pass untuk kedua arah.
- Absorption: limit order besar termakan tanpa reversal → konfirmasi breakout.
- Crisis mode: OFS relax from 3 to 2. Gate lain tetap.

## Dependencies
- Phase 4 (indicators core — FRAMA digunakan sebagai referensi level)

## Acceptance Criteria
- S_Delta akurat hitung aggressor buy/sell
- S_CVD deteksi divergensi kumulatif
- S_DOM detect DOM imbalance + absorption
- OFS final correct signed (positif/negatif)
- OFS threshold sesuai phase (3 normal, 2 power hour)
- Crisis relax OFS berfungsi
- Semua test lulus
