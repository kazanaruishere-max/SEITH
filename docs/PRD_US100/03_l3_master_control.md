# Fase 3: L3 Engine — Master Control

## Goal
Event loop, state manager, statistical brain, anti-paralysis. Phase-aware + skip classification.

## Files Created
```
indices/US100/core/l3_engine/
├── mod.rs
├── event_loop.rs              # Tokio loop 15s, M1/M15 tick, US hours aware
├── state_manager.rs           # State: US_MARKET, NO_TRADE, SESSION_CLOSED, CRISIS
├── statistical_brain.rs       # Statistical synthesis, entry/TP/SL calculation
├── anti_paralysis.rs          # Skip strike counter, threshold relaxation
```

## PRD Reference

**Level 3 Components (PRD §2.2 Level 3 table):**
- US100 Event Loop: event-driven loop aktif setiap penutupan lilin M1/M15, hanya jalan saat US Market Hours (14:30-21:00 UTC)
- Adaptive State Manager: membaca `skip_strike_count` dari SQLite sebelum setiap iterasi
- 5-Gate Indicator Pipeline: Macro → HV Z-Score → FRAMA → Orderflow → VWAP+Yield

**State Machine (PRD §2.4):**
```
[BOOT] -> [OPEN] -> (30m) -> [NORMAL] <----------+
                              | MODE=SNIPER     |
                     +-> [LUNCH] -> (90m)       |
                     |    MODE=NO TRADE          |
                     |          |               |
                     +----------+-> [NORMAL]    |
                                   | MODE=SNIPER |
                                   +-> [POWER H] |
                                        MODE=SCALP|
                                        |         |
                          +-- 21:00 UTC           |
                          v                       |
                    [FORCE_CLOSE]                 |
                          |                        |
                          v                        |
                    [SESSION_CLOSED] -> Standby    |
                                                    |
+- Macro Event (RED) -> [NO_TRADE]                 |
|     -> Auto-resume T+2 jam post-event            |
|                                                    |
+- Post-Trade -> Rekalibrasi -> ---------------------+
|                                                    |
+- Skip >= 3 -> [CRISIS_ADAPTATION]                  |
|     -> Relax OFS ke 2 (bukan semua gate)           |
|     -> MODE = SCALP (darurat, RR 1:0.5)            |
|                                                    |
+- Skip >= 5 -> [CRISIS_CEILING]                     |
|     -> Force reset skip_count = 0, standby 1 session
|     -> Telegram alert                              |
|                                                    |
+- Phase berubah -> [phase berikutnya]
|
+- Weekend / US Holiday -> [DAY_OFF] -> standby total
```

**Anti-Paralysis (PRD §7 + §2.4):**
- skip_count < 3: normal.
- skip_count 3-4: CRISIS — relax OFS 3→2, force SCALP mode, gate lain tetap.
- skip_count ≥ 5: CRISIS CEILING — reset skip_count=0, standby 1 session penuh, Telegram alert.
- Skip classification: "market condition" skip (HV extreme, Lunch, Gap, Macro) TIDAK increment. Hanya "signal reject" skip yang increment.

## Key Decisions
- Event loop: Tokio interval 15 detik, trigger on M1/M15 close. Skip loop di luar US hours.
- State transitions: time-based (OPEN→NORMAL→LUNCH→NORMAL→POWER HOUR→CLOSE) dan event-based (NO_TRADE, CRISIS).
- Skip counter: dibaca dari SQLite di awal setiap iterasi. Hanya signal reject yang increment.
- Crisis ceiling: standby 1 sesi penuh. Semua gate mati. Telegram alert dikirim.
- NO force entry — crisis hanya relax OFS threshold, bukan auto-enter.
- Power Hour: sub-mode of NORMAL. OFS relax 3→2, VWAP relax 2.5→2.0.

## Dependencies
- Phase 2 (L0 infra — session filter, macro filter, data feed)

## Acceptance Criteria
- Event loop berjalan di US hours, idle di luar
- State machine transisi sesuai timeline
- Skip counter akurat (tidak increment untuk market condition skip)
- Crisis mode aktif di skip ≥3, relax OFS only
- Crisis ceiling reset di skip ≥5, standby 1 session
- Semua test lulus
