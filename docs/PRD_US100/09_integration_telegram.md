# Fase 9: Integration — Signal Types + Telegram Dispatch + Full Cycle

## Goal
Signal types, signal validator + enricher final. Telegram dispatch. Integration test full cycle dari tick → signal → execution → journal.

## Files Created
```
indices/US100/signals/
├── signal_types.rs          # Enum: BuySignal, SellSignal, NoSignal
├── signal_validator.rs      # Validasi konfluensi sinyal 5 gate
├── signal_enricher.rs       # Tambah metadata: confidence, regime, yield, ofs
```

## PRD Reference

**Signal Flow (PRD §2.3 Data Flow — final part):**
```
+-> Statistical Synthesis: Entry, TP, SL (sesuai mode RR + lot)
+-> Execution: OANDA API (Stop/Limit untuk Sniper, Market untuk Scalp)
|     Jika gagal: retry 1x. Jika tetap gagal: log error, skip rekalibrasi
|
+-> if execution success -> Telegram: Chart + sinyal
|   if failed -> Telegram: Error alert
```

**Self-Learning Tahap 5 (PRD §2.2):** Telegram Report — kirim laporan P/L akun riil.

## Key Decisions
- **Signal types:** `BuySignal` (price target, confidence, time), `SellSignal`, `NoSignal(reason)`.
- **Validator:** periksa konfluensi 5 gate. Gate pass count → confidence multiplier.
- **Enricher:** tambahkan HV regime label, yield Z value, OFS score, gate breakdown.
- **Telegram dispatch:** 2 tipe pesan: (a) order success → chart + signal detail, (b) error → alert.
- **Integration test:** full cycle dari tick input → final journal entry. Test semua path (success, fail, crisis, ceiling).

## Dependencies
- Phase 6 (pipeline — signal output)
- Phase 7 (execution — order result)
- Phase 8 (analytics — journal entry)

## Acceptance Criteria
- Signal types enum lengkap (Buy/Sell/NoSignal + metadata)
- Validator periksa 5 gate correct
- Enricher tambah semua metadata
- Telegram dispatch: success + error message
- Integration test: full cycle pass semua path
- Semua test lulus
