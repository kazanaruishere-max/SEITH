# Fase 8: Analytics — Self-Learning Loop

## Goal
Closed-loop self-learning engine: trade journal, recalibration (immediate + batch), auto-kill pending orders, win probability map.

## Files Created
```
indices/US100/analytics/
├── mod.rs
├── trade_journal.rs        # Post-trade logging ke SQLite
├── rekalibrasi.rs          # Cognitive recalibration (immediate sniper / batch scalp)
├── win_probability_map.rs  # Win probability map update
├── auto_kill.rs            # Auto-kill pending orders T+5 menit market close
```

```
indices/US100/data/
├── mod.rs
├── schema.rs               # US100-specific SQLite schema
├── queries.rs              # SQL queries for analytics
```

## PRD Reference

**Closed-Loop Self-Learning (PRD §2.2):**
```
Tahap 1: Auto-Kill -> hapus semua pending order (T+5 menit market close)
Tahap 2: Extract -> Win/Loss, Slippage, Spread, P/L
Tahap 3: SQLite Write -> simpan baris baru hasil eksekusi
Tahap 4: Rekalibrasi -> sniper: immediate. scalp: trade1 immediate, trade2+ batch 2.
         slippage darurat (>50%) -> force immediate.
         optimasi buffer, reset counter, update win probability map
Tahap 5: Telegram Report -> kirim laporan P/L akun riil
Tahap 6: Loop -> kembali ke Event Loop awal
```

**Skip Classification (PRD §2.4 + §7):**
- "Market condition" skip (HV extreme, Lunch, Gap, Macro) — TIDAK increment skip_count
- "Signal reject" skip — increment skip_count
- skip_count < 3: normal
- skip_count 3-4: crisis (relax OFS, SCALP mode)
- skip_count ≥ 5: crisis ceiling (reset 0, standby 1 session)

## Key Decisions
- **Auto-Kill:** T+5 menit market close (21:00 UTC). Hapus semua pending order via OANDA API.
- **Extract:** Win/Loss (price-based), slippage (entry vs signal price), spread (at entry), P/L.
- **SQLite Write:** 1 baris per trade. Kolom: id, symbol, direction, entry, exit, SL, TP, slippage, spread, P/L, confidence, gate_result, mode, timestamp.
- **Recalibration Sniper:** immediate — update buffer setelah 1 trade.
- **Recalibration Scalp:** trade 1 immediate, trade 2+ batch 2 (kumpulin 2 trade dulu).
- **Slippage > 50%:** force immediate recalibration regardless of mode.
- **Reset counter:** reset skip_count = 0 setelah trade sukses.
- **Win probability map:** update setelah recalibration.
- **Telegram Report:** P/L, equity curve, trade summary. Dispatch di Tahap 5.

## Dependencies
- Phase 7 (execution — trade result sebagai input)
- Phase 3 (state manager — skip_count management)

## Acceptance Criteria
- Auto-kill hapus pending order di 21:05 UTC
- Trade journal write + read dari SQLite benar
- Rekalibrasi sniper: immediate setelah trade close
- Rekalibrasi scalp: trade1 immediate, trade2+ batch 2
- Slippage darurat >50%: force immediate
- Win probability map terupdate
- Skip_count reset setelah trade sukses
- Semua test lulus
