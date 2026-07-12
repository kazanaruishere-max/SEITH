#!/usr/bin/env python3
"""US100 Full Backtest — 1 Year OANDA M15 Data, 5-Gate Pipeline."""

import json, numpy as np
from datetime import datetime

print('Loading data...', flush=True)
with open('network/us100_m15_1y.json') as f:
    raw = json.load(f)
print(f'Loaded {len(raw)} bars', flush=True)

# ===== CONFIG (PRD defaults, will sweep) =====
HV_SKIP_LOW = -1.0
HV_SKIP_HIGH = 2.0
HV_ELEVATED = 2.0
FRAMA_OVEREXT = 0.5
OFS_NOISE = 1.0
OFS_TH = 2.0
VWAP_BAND = 2.5
RR_SNIPER = 1.5
RR_SCALP = 0.5

# ===== PREPARE DATA =====
p = np.array([x['close'] for x in raw], dtype=float)
h = np.array([x['high'] for x in raw], dtype=float)
l = np.array([x['low'] for x in raw], dtype=float)
v = np.array([x['volume'] for x in raw], dtype=float)
times = np.array([x['time'] for x in raw], dtype=int)
n = len(p)

# Date utilities
def ts_to_dt(ts):
    return datetime.fromtimestamp(ts)

def is_weekend(ts):
    return ts_to_dt(ts).weekday() >= 5

def is_us_hours(ts):
    dt = ts_to_dt(ts)
    mins = dt.hour * 60 + dt.minute
    return 13*60+30 <= mins < 20*60  # 13:30-20:00 UTC

def session_phase(ts):
    dt = ts_to_dt(ts)
    mins = dt.hour * 60 + dt.minute
    if mins < 14*60: return 'OPEN'
    if mins < 16*60+30: return 'NORMAL'
    if mins < 18*60: return 'LUNCH'
    if mins < 19*60+30: return 'NORMAL'
    if mins < 20*60: return 'POWER_HOUR'
    return 'CLOSE'

# ===== COMPUTE INDICATORS =====
print('Computing indicators...', flush=True)

# HV Z-Score (window=20)
hv_z = np.zeros(n)
ret = np.diff(p) / (p[:-1] + 1e-10)
for i in range(20, len(ret)):
    seg = ret[i-20:i]
    s = seg.std()
    hv_z[i] = (seg[-1] - seg.mean()) / s if s > 1e-10 else 0.0

# FRAMA (period=14)
zf = np.full(n, np.nan)
for i in range(14, n):
    w = p[i-14:i+1]
    r1 = max(w[:8].max() - w[:8].min(), 0.001)
    r2 = max(w[7:].max() - w[7:].min(), 0.001)
    r3 = max(w.max() - w.min(), 0.001)
    D = np.clip((np.log(r1+r2) - np.log(r3)) / np.log(2) + 1, 1, 2)
    alpha = np.exp(-4.6 * (D - 1))
    f = alpha * p[i] + (1 - alpha) * p[i-1]
    s10 = p[max(0, i-9):i+1].std()
    zf[i] = (p[i] - f) / s10 if s10 > 1e-10 else 0.0

# OFS Proxy (volume-weighted)
ratio = np.where(h - l > 1e-10, (p - l) / (h - l), 0.5)
s_delta = 2.0 * (ratio - 0.5)
raw_delta = s_delta * v
cvd = np.cumsum(raw_delta)
cvd_norm = np.zeros(n)
for i in range(20, n):
    seg = cvd[i-20:i+1]
    s = seg.std()
    z = (seg[-1] - seg.mean()) / s if s > 1e-10 else 0.0
    cvd_norm[i] = np.tanh(z / 3.0)
s_dom = np.zeros(n)
for i in range(20, n):
    rng = p[i-20:i].max() - p[i-20:i].min() + 1e-10
    pos = (p[i] - p[i-20]) / rng
    s_dom[i] = np.clip((0.5 - pos) * 2, -1, 1)
ofs = s_delta + cvd_norm + s_dom

# Session VWAP (daily)
vwap = np.full(n, np.nan)
vwap_u = np.full(n, np.nan)
vwap_l = np.full(n, np.nan)
prev_date = None; cpv = 0.0; cv = 0.0
for i in range(n):
    d = ts_to_dt(times[i]).date()
    if d != prev_date:
        cpv = 0.0; cv = 0.0; prev_date = d
    cpv += p[i] * v[i]; cv += v[i]
    if cv > 0:
        w = cpv / cv
        vwap[i] = w
        s = np.sqrt(max(np.mean((p[max(0,i-20):i+1] - w)**2), 1e-10))
        vwap_u[i] = w + VWAP_BAND * s
        vwap_l[i] = w - VWAP_BAND * s

print('Indicators done.', flush=True)

# ===== PIPELINE + EXIT SIMULATION =====
def backtest(ofs_th, rr_ratio):
    sigs = []
    total_bars = 0
    gate_b = {'G1_hv':0, 'G2_frama':0, 'G3_ofs':0, 'G4_vwap':0}
    
    for i in range(60, n):
        ts = times[i]
        if is_weekend(ts):
            continue
        if not is_us_hours(ts):
            continue
        ph = session_phase(ts)
        if ph in ('OPEN', 'LUNCH', 'CLOSE'):
            continue
        
        total_bars += 1
        
        # G1: HV
        if hv_z[i] < HV_SKIP_LOW or hv_z[i] > HV_SKIP_HIGH:
            gate_b['G1_hv'] += 1; continue
        
        # G2: FRAMA
        if np.isnan(zf[i]) or zf[i] > FRAMA_OVEREXT:
            gate_b['G2_frama'] += 1; continue
        
        # G3: OFS
        if abs(ofs[i]) <= OFS_NOISE or abs(ofs[i]) < ofs_th:
            gate_b['G3_ofs'] += 1; continue
        
        # G4: VWAP
        if p[i] > vwap_u[i] or p[i] < vwap_l[i]:
            gate_b['G4_vwap'] += 1; continue
        
        # 5/5 → SIGNAL
        dirn = 'buy' if s_delta[i] > 0 else 'sell'
        entry = p[i]
        is_ph = (ph == 'POWER_HOUR')
        
        if is_ph:
            sl = entry * 0.997 if dirn == 'buy' else entry * 1.003
            tp = entry * (1 + rr_ratio * 0.003) if dirn == 'buy' else entry * (1 - rr_ratio * 0.003)
        else:
            sl = entry * 0.995 if dirn == 'buy' else entry * 1.005
            tp = entry * (1 + rr_ratio * 0.005) if dirn == 'buy' else entry * (1 - rr_ratio * 0.005)
        
        # Exit simulation (max 12 bars = 3 hours)
        win = None; exit_pl = 0.0
        for j in range(1, min(13, n - i)):
            bh = h[i+j]; bl = l[i+j]
            if dirn == 'buy':
                if bh >= tp: win = True; exit_pl = tp - entry; break
                if bl <= sl: win = False; exit_pl = sl - entry; break
            else:
                if bl <= tp: win = True; exit_pl = entry - tp; break
                if bh >= sl: win = False; exit_pl = entry - sl; break
        
        sigs.append({
            'time': ts, 'dir': dirn, 'entry': entry,
            'sl': sl, 'tp': tp, 'ofs': ofs[i], 'hv': hv_z[i],
            'zf': zf[i], 'win': win, 'pl': exit_pl,
            'phase': ph, 'rr_used': rr_ratio,
        })
    
    return sigs, gate_b, total_bars

# ===== PARAMETER SWEEP =====
print('\n=== FULL BACKTEST: OANDA US100 M15, 1 TAHUN ===')
print('Bars:', n, '| Days:', len(set(ts_to_dt(t).date() for t in times)))

sweep_results = []
for ofs_th in [x*0.1 for x in range(5, 30)]:
    for rr in [1.5, 2.0, 2.5, 3.0, 3.5, 4.0]:
        sigs, gates, tb = backtest(ofs_th, rr)
        completed = [s for s in sigs if s['win'] is not None]
        wins = [s for s in completed if s['win']]
        losses = [s for s in completed if not s['win']]
        nc = len(completed)
        nw = len(wins); nl = len(losses)
        wr = nw / nc * 100 if nc > 0 else 0
        avg_w = np.mean([s['pl'] for s in wins]) if nw > 0 else 0
        avg_l = abs(np.mean([s['pl'] for s in losses])) if nl > 0 else 1
        pf = (nw * avg_w) / (nl * avg_l) if nl > 0 and avg_l > 0 else 0
        avg_rr = avg_w / avg_l if avg_l > 0 else 0
        
        sweep_results.append({
            'ofs_th': ofs_th, 'rr': rr, 'sigs': len(sigs),
            'comp': nc, 'wr': wr, 'pf': pf, 'avg_rr': avg_rr,
        })

# Best by WR (minimum 30 completed trades)
best_wr = max((r for r in sweep_results if r['comp'] >= 30), key=lambda r: r['wr'])
# Best by PF (minimum 30 completed)
best_pf = max((r for r in sweep_results if r['comp'] >= 30), key=lambda r: r['pf'])
# Best balanced (WR * PF)
best_bal = max((r for r in sweep_results if r['comp'] >= 30), key=lambda r: r['wr'] * r['pf'])

print(f'\n=== TOP RESULTS (min 30 trades) ===')
print(f'{"OFS":>4} {"RR":>4} {"Sig":>5} {"Comp":>5} {"WR%":>6} {"PF":>6} {"RR_avg":>6}')
for tag, r in [('BEST WR', best_wr), ('BEST PF', best_pf), ('BEST BAL', best_bal)]:
    print(f'{tag}: {r["ofs_th"]:4.1f} {r["rr"]:4.1f} {r["sigs"]:5} {r["comp"]:5} {r["wr"]:5.1f}% {r["pf"]:6.2f} {r["avg_rr"]:6.2f}')

# Check target: WR 55% + PF 4.0
print(f'\n=== TARGET ANALYSIS ===')
target_results = [r for r in sweep_results if r['wr'] >= 50 and r['pf'] >= 3.0 and r['comp'] >= 30]
if target_results:
    print(f'Params achieving WR>=50% + PF>=3.0: {len(target_results)}')
    for r in sorted(target_results, key=lambda x: -x['wr'] * x['pf'])[:3]:
        print(f'  OFS={r["ofs_th"]:.1f} RR={r["rr"]:.1f} WR={r["wr"]:.1f}% PF={r["pf"]:.2f} trades={r["comp"]}')
else:
    print('No param set reaches WR>=50% + PF>=3.0 with 30+ trades')

# Full table (filtered)
print(f'\n=== FULL TABLE (sorted by WR*PF, top 15) ===')
ranked = sorted([r for r in sweep_results if r['comp'] >= 20], key=lambda r: -r['wr'] * r['pf'])
for r in ranked[:15]:
    print(f'  OFS={r["ofs_th"]:.1f} RR={r["rr"]:.1f} sig={r["sigs"]} comp={r["comp"]} WR={r["wr"]:.1f}% PF={r["pf"]:.2f}')
