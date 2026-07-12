# Fase 7: Execution + OANDA Bridge

## Goal
Order placement resilient: limit/stop/market order, risk manager (lot sizing hybrid multiplier), OANDA bridge, order failure handling, auto-reconnect.

## Files Created
```
indices/US100/core/execution/
├── mod.rs
├── limit_order.rs          # Limit order (sniper secondary)
├── stop_order.rs            # Stop order (sniper primary / momentum)
├── market_entry.rs          # Market entry (scalp / breakout confirmation)
├── order_manager.rs         # Order routing, validation, lifecycle. Phantom position detection via GET /trades
├── risk_manager.rs          # Risk limits, lot sizing hybrid multiplier, SL/TP
```

```
indices/US100/external/
├── mod.rs
├── oanda_feed.rs            # OANDA bridge (PyO3 -> python/oanda.py)
├── calendar_scraper.rs      # Macro calendar filter (PyO3 -> python/scraper.py)
```

## PRD Reference

**Execution Modes (PRD §2.2 Execution Layer):**

| Mode | Prioritas | Session | Trigger | Order Type |
|------|-----------|---------|---------|------------|
| Stop Order (Sniper) | PRIMARY | NORMAL | Breakout konfirmasi orderflow OFS >= 3 | BUY_STOP / SELL_STOP |
| Limit Order (Sniper) | Secondary | NORMAL | Pullback ke FRAMA saat OFS >= 3 | BUY_LIMIT / SELL_LIMIT |
| Market Entry (Scalp) | POWER HOUR | POWER HOUR / Crisis | OFS >= 2 + momentum kuat | Market Order |
| Emergency | Crisis skip >= 3 | Crisis Adaptation | Paksa mode SCALP dengan OFS relax | Market Order |

**Mode Selection (PRD §2.2):**
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
        -> Relax OFS ke 2, exit cepat, risk 0.5%
```

**Risk Management (PRD §8):**

| Parameter | US100 |
|-----------|-------|
| Max Risk/Trade (Sniper) | 0.75% equity |
| Max Risk/Trade (Scalp) | 0.50% equity |
| Daily Loss Limit | 2.5% equity → auto-halt |
| Weekly Loss Limit | 5.0% equity → pause 48 jam |
| Max Open Position | 1 |
| Spread Tolerance | <= 1.5 points |
| Force Close | 21:00 UTC |
| Order Failed | Retry 1x + log. Jika tetap gagal: Telegram error, skip rekalibrasi. **Phantom risk: GET /trades untuk verifikasi** |
| OANDA Reconnect | 3x retry, 60s standby, Telegram alert |

**Lot Sizing Hybrid Multiplier (PRD §8.1):**
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
  RED                 -> 0 (no trade)
```

## Key Decisions
- **Sniper entry:** Stop Order default. Limit Order hanya jika Z_FRAMA dekat 0 + support.
- **Order failure:** Retry 1x. Jika gagal total → GET /trades untuk deteksi phantom position.
- **Phantom position:** OANDA API kadang return error tapi order terkirim. Verifikasi via GET /trades.
- **Risk manager:** Validasi semua limit sebelum eksekusi (lot max, daily loss, spread).
- **Force close:** 21:00 UTC — market order close semua posisi.
- **Lot rounding:** Sesuai OANDA lot size constraints.
- **Equity:** Real-time dari OANDA balance API setiap kalkulasi lot.

## Dependencies
- Phase 6 (pipeline — signal classifier output sebagai input)
- Phase 3 (state manager untuk crisis/normal mode)

## Acceptance Criteria
- Stop order sniper: BUY_STOP/SELL_STOP dengan buffer 2 point
- Limit order sniper: BUY_LIMIT/SELL_LIMIT di level pullback
- Market order scalp: eksekusi segera, TP/SL ketat
- Order failure → retry 1x → log + Telegram → skip rekalibrasi
- Phantom position terdeteksi via GET /trades
- Lot sizing formula akurat (hybrid multiplier)
- Risk limits enforce (max lot, daily loss, spread)
- Force close 21:00 UTC
- Semua test lulus
