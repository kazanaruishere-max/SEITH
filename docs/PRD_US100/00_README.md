# AI SEITH US100 — Phase Implementation Guide

**Parent:** `docs/PRD_US100.md` (v1.5, 747 lines)
**Diagram:** `WORKFLOW US100.mmd` (304 lines, Mermaid)

## Phase Dependency Graph

```
1 (Setup) ──> 2 (L0) ──> 3 (L3) ──> 4 (Indicators Core) ──> 5 (Orderflow)
                                                              │
                                                              ▼
                                     9 (Integ) <── 7 (Exec) <── 6 (Pipeline)
                                         │
                                         ▼
8 (Analytics) ──> 10 (Backtest) ──> 11 (Live)
```

Phases wajib dikerjakan **berurutan** (1 → 2 → ... → 11). Setiap phase bergantung pada output phase sebelumnya.

## Target Metrics (Cross-Phase Reference)

| Metric | Target | Hard Limit |
|--------|--------|------------|
| Max DD | ≤ 6% | ≤ 10% (hard 12%) |
| Win Rate | ≥ 80% | ≥ 70% |
| Profit Factor | ≥ 4.0 | ≥ 3.8 |
| Recovery Factor | ≥ 4.0 | ≥ 3.0 |
| Consecutive Win | ≥ 9 | ≥ 6 |
| Consecutive Loss | ≤ 3 | ≤ 4 (auto-halt ≥ 5) |
| Max Risk/Trade | 0.75% (sniper) / 0.50% (scalp) | — |
| Max Daily Loss | 2.5% equity | Auto-halt |
| Max Open Position | 1 | — |

## State Machine (Cross-Phase Reference)

8 states: `BOOT → OPEN → NORMAL → LUNCH → POWER HOUR → CLOSE → SESSION_CLOSED`, plus `NO_TRADE`, `CRISIS`, `DAY_OFF`.

## Useful Commands

```bash
cargo build                                        # Verify compilation
cargo test                                         # All tests
cargo fmt --check && cargo clippy -- -D warnings   # Pre-commit gate
seith US100                                        # Run module
```
