#!/usr/bin/env python3
"""AI SEITH US100 — Live paper trading on OANDA via MT5."""

import sys, json, os, time
import numpy as np
from datetime import datetime, timezone

sys.path.insert(0, 'python/python')
from seith_bridge.mt5 import init_mt5, get_rates_json, get_tick, get_dom, shutdown
import MetaTrader5 as mt5

# === CONFIG ===
SYMBOL = 'US100.cash'
TIMEFRAME = mt5.TIMEFRAME_M1
BARS_LOAD = 200
HV_W = 20
FRAMA_P = 14
CVD_W = 20
OFS_TH = 2.0  # dari validasi live P80=1.81, P95=2.97
RR_SNIPER = 1.5
RR_SCALP = 0.5
RISK_SNIPER = 0.0075
RISK_SCALP = 0.0050

# === STATE ===
pending_signals = []
trade_log = []


def compute_indicators(rates):
    """Compute all 5-gate indicators from OANDA rates."""
    p = np.array([x['close'] for x in rates], dtype=float)
    h = np.array([x['high'] for x in rates], dtype=float)
    l = np.array([x['low'] for x in rates], dtype=float)
    v = np.array([x['volume'] for x in rates], dtype=float)
    n = len(p)

    # HV Z-Score
    hv_z = np.zeros(n)
    ret = np.diff(p) / (p[:-1] + 1e-10)
    for i in range(HV_W, len(ret)):
        seg = ret[i - HV_W:i]
        s = seg.std()
        hv_z[i] = (seg[-1] - seg.mean()) / s if s > 1e-10 else 0.0

    # FRAMA
    zf = np.full(n, np.nan)
    for i in range(FRAMA_P, n):
        w = p[i - FRAMA_P:i + 1]
        half = FRAMA_P // 2
        n1, x1 = w[:half + 1].min(), w[:half + 1].max()
        n2, x2 = w[half:].min(), w[half:].max()
        n3, x3 = w.min(), w.max()
        r1, r2, r3 = max(x1 - n1, .001), max(x2 - n2, .001), max(x3 - n3, .001)
        D = np.clip((np.log(r1 + r2) - np.log(r3)) / np.log(2) + 1, 1, 2)
        alpha = np.exp(-4.6 * (D - 1))
        prev = p[i - 1]
        f = alpha * p[i] + (1 - alpha) * prev
        std10 = p[max(0, i - 9):i + 1].std()
        zf[i] = (p[i] - f) / std10 if std10 > 1e-10 else 0.0

    # OFS Proxy (volume-weighted, live tuned)
    ratio = np.where(h - l > 1e-10, (p - l) / (h - l), 0.5)
    s_delta = 2.0 * (ratio - 0.5)
    raw_delta = s_delta * v
    cvd = np.cumsum(raw_delta)
    cvd_norm = np.zeros(n)
    for i in range(CVD_W, n):
        seg = cvd[i - CVD_W:i + 1]
        s = seg.std()
        z = (seg[-1] - seg.mean()) / s if s > 1e-10 else 0.0
        cvd_norm[i] = np.tanh(z / 3.0)

    s_dom = np.zeros(n)
    for i in range(20, n):
        pos = (p[i] - p[i - 20]) / (p[i - 20:i].max() - p[i - 20:i].min() + 1e-10)
        s_dom[i] = np.clip((0.5 - pos) * 2, -1, 1)

    ofs = s_delta + cvd_norm + s_dom

    return {
        'close': p, 'high': h, 'low': l, 'volume': v,
        'hv_z': hv_z, 'z_frama': zf, 's_delta': s_delta,
        'cvd_norm': cvd_norm, 's_dom': s_dom, 'ofs': ofs,
    }


def run_pipeline(indicators, idx, now):
    """5-Gate pipeline. Returns (signal_dict, gate_breakdown)."""
    gate = {'G0_macro': False, 'G1_hv': False, 'G2_frama': False, 'G3_ofs': False, 'G4_vwap': False}

    # G0: Macro (skip — implemented via date filter in loop)
    gate['G0_macro'] = True

    # G1: HV Z-Score
    hv = indicators['hv_z'][idx]
    if hv < -1.0 or hv > 2.0:
        return None, gate
    gate['G1_hv'] = True

    # G2: FRAMA
    zf_val = indicators['z_frama'][idx]
    if np.isnan(zf_val) or zf_val > 0.5:
        return None, gate
    gate['G2_frama'] = True

    # G3: OFS
    ofs = indicators['ofs'][idx]
    if abs(ofs) <= 1.0 or abs(ofs) < OFS_TH:
        return None, gate
    gate['G3_ofs'] = True

    # G4: VWAP (approximate — simple SMA for live)
    p = indicators['close'][max(0, idx - 20):idx + 1]
    vwap = p.mean()
    band = p.std() * 2.5
    close = indicators['close'][idx]
    if close > vwap + band or close < vwap - band:
        return None, gate
    gate['G4_vwap'] = True

    # 5/5 → SIGNAL
    direction = 'buy' if indicators['s_delta'][idx] > 0 else 'sell'
    entry = close
    mode = 'SNIPER'  # could check power hour based on time
    sl = entry * 0.995 if direction == 'buy' else entry * 1.005
    tp = entry * (1 + RR_SNIPER * 0.005) if direction == 'buy' else entry * (1 - RR_SNIPER * 0.005)

    return {
        'time': now.isoformat() if hasattr(now, 'isoformat') else str(now),
        'direction': direction, 'entry': entry, 'sl': sl, 'tp': tp,
        'mode': mode, 'ofs': ofs, 'hv_z': hv,
        'z_frama': zf_val,
    }, gate


def print_signal(sig):
    """Print a formatted signal."""
    print(f'  [{sig["mode"]}] {sig["direction"].upper()} @ {sig["entry"]:.2f}')
    print(f'    SL={sig["sl"]:.2f} TP={sig["tp"]:.2f}')
    print(f'    OFS={sig["ofs"]:.2f} HV={sig["hv_z"]:.2f} ZFRAMA={sig["z_frama"]:.2f}')


# === MAIN ===
print('=' * 55)
print('  AI SEITH US100 — Paper Trading (OANDA Live)')
print(f'  OFS threshold: {OFS_TH}')
print('=' * 55)

if not init_mt5():
    print('FAIL: MT5 init')
    exit(1)
print(f'MT5 OK')

cycle = 0
try:
    while True:
        cycle += 1
        now = datetime.now(timezone.utc)
        h = now.hour
        m = now.minute

        # Skip outside US hours (13:30-20:00 UTC)
        if h < 13 or h >= 20 or (h == 13 and m < 30):
            if cycle % 60 == 1:
                print(f'[{now.strftime("%H:%M:%S")}] Outside US hours, sleeping...')
            time.sleep(10)
            continue

        # Skip weekends
        if now.weekday() >= 5:
            time.sleep(300)
            continue

        # Fetch fresh rates
        rates_json = get_rates_json(SYMBOL, BARS_LOAD, TIMEFRAME)
        if not rates_json:
            time.sleep(5)
            continue
        rates = json.loads(rates_json)

        # Compute indicators
        ind = compute_indicators(rates)

        # Check last 3 bars for signals
        for offset in [1, 2, 3]:
            idx = len(ind['close']) - offset
            if idx < 50:
                continue

            signal, gates = run_pipeline(ind, idx, rates[idx])
            if signal:
                # Check if this signal was already logged
                sig_time = rates[idx]['time']
                if not any(abs(t['time'] - sig_time) < 60 for t in trade_log[-20:]):
                    print(f'\n[{now.strftime("%H:%M:%S")}] === 5/5 SIGNAL ===')
                    print_signal(signal)
                    trade_log.append({'time': sig_time, **signal})

        # Print status every 10 cycles
        if cycle % 10 == 0:
            tick = get_tick(SYMBOL)
            if tick:
                spread = tick['ask'] - tick['bid']
                print(f'[{now.strftime("%H:%M:%S")}] bid={tick["bid"]:.1f} ask={tick["ask"]:.1f} '
                      f'spread={spread:.1f} signals_today={len(trade_log)}')

        time.sleep(15)  # 15-second cycle

except KeyboardInterrupt:
    print(f'\n\n=== Session Summary ===')
    print(f'Total signals: {len(trade_log)}')
    for i, s in enumerate(trade_log):
        print(f'  #{i+1}: {s["direction"]} @ {s["entry"]:.2f} OFS={s["ofs"]:.2f}')
    shutdown()
    print('Done.')
