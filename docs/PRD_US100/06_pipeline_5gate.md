# Fase 6: L1 Pipeline — 5-Gate Router + Signal Classifier

## Goal
5-Gate pipeline sequential berjalan: Macro → HV Z-Score → FRAMA → Orderflow → VWAP+Yield. Signal classifier 5/5 pass.

## Files Created
```
indices/US100/core/l1_pipeline/
├── mod.rs
├── macro_gate.rs           # Gate 0: Macro calendar + earnings filter
├── hv_compass.rs            # Gate 1: HV Z-Score regime detection (real-time)
├── pipeline_router.rs      # Gate 2-4: FRAMA -> Orderflow -> VWAP + Yield
├── signal_classifier.rs    # Final: 5/5 lolos? -> Valid / Skip / Crisis
```

```
indices/US100/signals/
├── mod.rs
├── signal_types.rs         # Enum: BuySignal, SellSignal, NoSignal
├── signal_validator.rs     # Validasi konfluensi sinyal 5 gate
├── signal_enricher.rs      # Tambah metadata: confidence, regime, yield, ofs
```

## PRD Reference

**Pipeline Alur (PRD §2.2 Pipeline):**
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

  GATE 1 — HV Z-Score -> regime detection (Phase 4 detail)
  GATE 2 — FRAMA       -> trend & pullback (Phase 4 detail)
  GATE 3 — Orderflow   -> OFS + absorption (Phase 5 detail)
  GATE 4 — VWAP + Yield -> level fair + yield Z + curve (Phase 4 detail)

  Jika 5/5 lolos -> SIGNAL VALID -> Eksekusi
  Jika < 5/5 -> SKIP (tunggu setup berikutnya, kecuali Crisis mode OFS relax)
```

**Crisis Mode (PRD §2.2 + §2.4):**
- skip ≥ 3: M7 → K (kembali ke pipeline dengan OFS relax 3→2)
- Bukan auto-valid — tetap harus lolos 5 gate dengan OFS longgar
- Jika tetap skip terus hingga ≥5: CRISIS CEILING (reset + standby)

## Key Decisions
- Pipeline sequential: jika gate N gagal, tidak lanjut ke gate N+1. Langsung SKIP.
- GATE 0 (macro): RED → skip total. ORANGE/WARNING → lanjut dengan flag lot 50%.
- Signal classifier: 5/5 = VALID. < 5/5 = SKIP (kecuali crisis relax).
- Crisis mode: skip_count 3-4 → relax OFS 3→2 → M7 → K (evaluasi ulang pipeline).
- Signal types: `BuySignal(points)`, `SellSignal(points)`, `NoSignal(reason)`.
- Enricher: tambah metadata ke signal — confidence level, regime label, yield Z, OFS value.

## Dependencies
- Phase 4 (indicators core)
- Phase 5 (orderflow)
- Phase 3 (state manager untuk skip_count + crisis mode)

## Acceptance Criteria
- Pipeline sequential: gate 0 → 1 → 2 → 3 → 4, berhenti di gate gagal pertama
- Macro gate klasifikasi RED/ORANGE/WARNING/GREEN benar
- HV compass threshold akurat (4 range, no dead-zone)
- Signal classifier 5/5 = VALID, < 5/5 = SKIP
- Crisis mode M7 → K routing berfungsi (relax + re-evaluate)
- Semua test lulus
