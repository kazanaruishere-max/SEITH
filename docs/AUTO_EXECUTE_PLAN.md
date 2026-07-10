# AUTO-EXECUTE IMPLEMENTATION PLAN

**Target:** AI SEITH auto-pasang Limit/Stop Order ke OANDA MT5 + Telegram signal

---

## A. Arsitektur Auto-Execute

```
[Signal Trigger] → risk_manager::can_trade()
                       │
                  ┌────┴────┐
                  │  PASS   │  FAIL → skip + log reason
                  └────┬────┘
                       │
                  ┌────┴────┐
                  │  plan_execution()  ← AI SEITH analysis
                  │  → Limit / Stop
                  └────┬────┘
                       │
                  ┌────┴────┐
                  │  Lot Scalable (Logistic S-Curve)
                  │  65%  → 0.00 skip
                  │  70%  → 0.01
                  │  75%  → 0.03
                  │  78%  → 0.04
                  │  80%+ → 0.05
                  └────┬────┘
                       │
                  ┌────┴────┐
                  │  MT5 place_pending_limit (IOC Pending Order)
                  │  + Telegram send_signal()
                  └─────────┘
```

---

## B. Files & Perubahan Detail

### B1. `shared/external/mt5_bridge.rs` — Method Baru `place_pending_limit()`

**Lokasi:** Setelah `place_order()`
**Isi:**
```rust
/// Hanya pending order (BUY_LIMIT / SELL_LIMIT / BUY_STOP / SELL_STOP).
/// TRADE_ACTION_PENDING — tanpa filling (IOC hanya untuk market order).
pub async fn place_pending_limit(
    &self,
    order_type: &str,   // "BUY_LIMIT", "SELL_LIMIT", "BUY_STOP", "SELL_STOP"
    volume: f64,
    price: f64,
    sl: f64,
    tp: f64,
) -> Result<u64> {
    let mt5_type = match order_type {
        "BUY_LIMIT" => 2,
        "SELL_LIMIT" => 3,
        "BUY_STOP" => 4,
        "SELL_STOP" => 5,
        _ => anyhow::bail!("Invalid pending order type: {}", order_type),
    };
    pyo3::Python::with_gil(|py| {
        let mt5 = pyo3::types::PyModule::import(py, "seith_bridge.mt5")?;
        let ticket: Option<i64> = mt5
            .call_method1("place_pending_order",
                (&self.symbol, mt5_type, volume, price, sl, tp))?
            .extract()?;
        ticket.map(|t| t as u64)
            .ok_or_else(|| anyhow::anyhow!("Pending order failed"))
    })
}
```

### B2. `python/seith_bridge/mt5.py` — Fungsi Baru `place_pending_order()`

**Lokasi:** Setelah `place_order()`
**Isi:**
```python
def place_pending_order(
    symbol: str,
    order_type: int,      # ORDER_TYPE_BUY_LIMIT (2) / SELL_LIMIT (3) / BUY_STOP (4) / SELL_STOP (5)
    volume: float,
    price: float,
    sl: float = 0.0,
    tp: float = 0.0,
    comment: str = "AI SEITH",
) -> Optional[int]:
    """Pasang pending order (Limit/Stop) dengan SL/TP bracket."""
    request = {
        "action": mt5.TRADE_ACTION_PENDING,
        "symbol": symbol,
        "volume": volume,
        "type": order_type,
        "price": price,
        "sl": sl,
        "tp": tp,
        "deviation": 10,
        "magic": 1001,
        "comment": comment,
        "type_time": mt5.ORDER_TIME_GTC,
    }
    result = mt5.order_send(request)
    if result.retcode != mt5.TRADE_RETCODE_DONE:
        print(f"[MT5] Pending order failed: {result.comment} (code {result.retcode})")
        return None
    print(f"[MT5] Pending order placed: ticket={result.order}")
    return result.order
```

### B3. `execution/order_manager.rs` — Hapus Instant Entry Path

**Perubahan di `plan_execution()`:**
```rust
// BEFORE:
if can_instant_entry(body_ratio_val, velocity) {
    ExecutionPlan::Instant(...)
}

// AFTER:
if can_instant_entry(body_ratio_val, velocity) {
    return ExecutionPlan::None;  // HARAM market order
}
```

### B4. `execution/risk_manager.rs` — Update Equity ke $4,500

| Field | Sebelum | Sesudah |
|-------|---------|---------|
| `max_risk_percent` | 1.0 | 1.0 ✅ |
| `max_daily_loss_percent` | 3.0 | 3.0 ✅ |
| Risk per trade max | $100 (1% of $10k) | **$45 (1% of $4.5k)** |
| Daily loss max | $300 | **$135** |

> **Note:** Modal $4,500 di-handle di event loop via `mt5.get_account()` balance real-time.

### B5. `event_loop.rs` — Wiring Lengkap + Scalable Lot

**Method baru:**
```rust
fn calculate_scalable_lot(confidence: f64) -> f64 {
    // Skip if confidence < 70% (noise filter untuk PF 4.0)
    if confidence < 70.0 { return 0.0; }
    // Logistic S-Curve: 70% -> 0.01, 75% -> 0.03, 80%+ -> 0.05
    let norm = ((confidence - 70.0) / 25.0).clamp(0.0, 1.0);
    let lot = 0.05 / (1.0 + std::f64::consts::E.powf(-6.0 * (norm - 0.5)));
    (lot * 100.0).round().max(1.0) / 100.0
}
```

**Alur `run_strategy()` setelah di-wire:**
```
1. risk_manager::can_trade(session, limits, spread, price)
   → FAIL → log + return

2. lot = calculate_scalable_lot(confidence)
   → 0.0 → skip

3. order_manager::plan_execution()
   → Limit/Stop → OK
   → Instant → skip (haram)
   → None → skip

4. mt5.place_pending_limit(order_type, lot, entry, sl, tp).await
   → success → log ticket
   → fail → log error

5. send_signal() ke Telegram (chart + format + ID)
```

### B6. `backtest/simulator.rs` — Update Initial Balance

| Line | Sebelum | Sesudah |
|------|---------|---------|
| Balance | `10_000.0` | `4_500.0` |

### B7. `backtest/reporter.rs` — Update Hardcoded Equity

| Line | Sebelum | Sesudah |
|------|---------|---------|
| Final balance calc | `10000.0 + net_pips` | `4500.0 + net_pips` |
| PnL calc | `final_balance - 10000.0` | `final_balance - 4500.0` |

---

## C. Data Flow Diagram

```
M15 Tick ──→ run_strategy()
                 │
    ┌────────────┴────────────┐
    │ 1. Session filter       │  Hour [5,12,19] UTC
    │ 2. HV Z-Score > 0.5     │
    │ 3. Direction contrarian  │
    └────────────┬────────────┘
                 │
    ┌────────────┴────────────┐
    │ RISK CHECK              │
    │ • can_trade()           │──→ FAIL → log + skip
    │ • is_jam_hantu?         │
    │ • news_mode?            │
    └────────────┬────────────┘
                 │ PASS
    ┌────────────┴────────────┐
    │ LOT SIZING              │
    │ calculate_scalable_lot  │──→ 0.0 → skip
    │ (confidence → lot)      │
    └────────────┬────────────┘
                 │
    ┌────────────┴────────────┐
    │ ORDER MANAGER           │
    │ plan_execution()        │──→ None → skip
    │ → Limit / Stop          │
    └────────────┬────────────┘
                 │
    ┌────────────┴────────────┐
    │ MT5 EXECUTION           │
    │ place_pending_limit()   │──→ FAIL → log error
    │ (IOC Pending Order)     │
    └────────────┬────────────┘
                 │
    ┌────────────┴────────────┐
    │ TELEGRAM                │
    │ send_signal()           │
    │ → chart + format + ID   │
    └─────────────────────────┘
```

---

## D. Urutan Implementasi (6 Steps)

| Step | File | Perubahan | Estimasi |
|------|------|-----------|----------|
| **1** | `mt5.py` | +`place_pending_order()` | 5 menit |
| **2** | `mt5_bridge.rs` | +`place_pending_limit()` | 10 menit |
| **3** | `order_manager.rs` | Hapus Instant path | 5 menit |
| **4** | `event_loop.rs` | Wiring + scalable lot | 20 menit |
| **5** | `simulator.rs` + `reporter.rs` | Update equtiy 4500 | 5 menit |
| **6** | fmt + clippy + test + commit | — | 5 menit |

**Total estimasi:** ~50 menit

---

## E. Testing Plan

| Test | Command | Expected |
|------|---------|----------|
| **Unit tests** | `cargo test -p xauusd --lib` | 135 passed |
| **Clippy** | `cargo clippy -D warnings` | 0 errors |
| **Fmt** | `cargo fmt --check` | Clean |
| **Live dry-run** | `cargo run --bin seith XAUUSD` | MT5 connect + Telegram |

---

## F. Rollback Plan

1. Edit `event_loop.rs` → comment baris `mt5.place_pending_limit()`
2. System fallback ke signal-only (seperti sekarang)
3. Atau set env var: `SEITH_NO_EXECUTE=1`

---

## G. Lot Mapping (Logistic S-Curve)

| Confidence | Lot | Risk ($4,500) | % Equity |
|-----------|-----|--------------|----------|
| < 70% | **0.00 (skip)** | $0 | 0% |
| 70% | 0.01 | $3 | 0.07% |
| 73% | 0.02 | $6 | 0.13% |
| 75% | 0.03 | $9 | 0.20% |
| 78% | 0.04 | $12 | 0.27% |
| 80%+ | 0.05 | $15 | 0.33% |

```
Lot
0.05 ┤          ═══════════
     │         ╔╝
0.04 ┤       ╔╝
     │      ╔╝
0.03 ┤     ╔╝
     │    ╔╝
0.02 ┤   ╔╝
     │  ╔╝
0.01 ┤ ╔╝
     │╔╝
     └──╨─────╨─────╨─────╨─────╨──
     70%   75%   80%   85%   90%
```

---

*Dokumen ini dibuat 2026-07-10. AI SEITH v1.0 — Auto-Execute Phase*
