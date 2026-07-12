#!/usr/bin/env python3
"""Backtest v2: trend-following exit (trailing stop), dynamic SL based on HV."""

import json, numpy as np
from datetime import datetime

with open('network/us100_m15_1y.json') as f:
    raw = json.load(f)
print(f'Loaded {len(raw)} bars', flush=True)

p = np.array([x['close'] for x in raw], dtype=float)
h = np.array([x['high'] for x in raw], dtype=float)
l = np.array([x['low'] for x in raw], dtype=float)
v = np.array([x['volume'] for x in raw], dtype=float)
times = np.array([x['time'] for x in raw], dtype=int)
n = len(p)

def td(ts):
    return datetime.fromtimestamp(ts)

# ===== INDICATORS (same as before) =====
hv_z = np.zeros(n)
ret = np.diff(p) / (p[:-1] + 1e-10)
for i in range(20, len(ret)):
    s = ret[i-20:i].std()
    hv_z[i] = (ret[i-20:i][-1] - ret[i-20:i].mean()) / s if s > 1e-10 else 0.0

zf = np.full(n, np.nan)
for i in range(14, n):
    w = p[i-14:i+1]; r1 = max(w[:8].max()-w[:8].min(), 0.001)
    r2 = max(w[7:].max()-w[7:].min(), 0.001)
    r3 = max(w.max()-w.min(), 0.001)
    D = np.clip((np.log(r1+r2)-np.log(r3))/np.log(2)+1, 1, 2)
    a = np.exp(-4.6*(D-1))
    f = a*p[i] + (1-a)*p[i-1]
    s10 = p[max(0,i-9):i+1].std()
    zf[i] = (p[i]-f)/s10 if s10 > 1e-10 else 0.0

ratio = np.where(h-l>1e-10, (p-l)/(h-l), 0.5)
sd = 2*(ratio-0.5)
rd = sd*v; cvd = np.cumsum(rd)
cvn = np.zeros(n)
for i in range(20, n):
    s = cvd[i-20:i+1].std()
    z = (cvd[i-20:i+1][-1]-cvd[i-20:i+1].mean())/s if s > 1e-10 else 0
    cvn[i] = np.tanh(z/3)
sdm = np.zeros(n)
for i in range(20, n):
    rng = p[i-20:i].max()-p[i-20:i].min()+1e-10
    pos = (p[i]-p[i-20])/rng
    sdm[i] = np.clip((0.5-pos)*2, -1, 1)
ofs = sd+cvn+sdm

# ATR for dynamic stops
atr = np.zeros(n)
for i in range(14, n):
    tr = max(h[i]-l[i], abs(h[i]-p[i-1]), abs(l[i]-p[i-1]))
    atr[i] = (atr[i-1]*13 + tr)/14 if i > 14 else tr

# ===== OPTIMIZED BACKTEST =====
# Use dynamic SL based on ATR + trailing stop
# Sniper trail: 2x ATR, Scalp trail: 1x ATR

def backtest_v2(ofs_th, rr_target, trail_atr_mult=2.0):
    sigs = []
    for i in range(60, n):
        ts = times[i]
        if td(ts).weekday() >= 5: continue
        d = td(ts); mins = d.hour*60 + d.minute
        if not (13*60+30 <= mins < 20*60): continue
        ph = ('OPEN','LUNCH','CLOSE') and 'NORMAL'
        if mins < 14*60: ph='OPEN'
        elif mins < 16*60+30: ph='NORMAL'
        elif mins < 18*60: ph='LUNCH'
        elif mins < 19*60+30: ph='NORMAL'
        elif mins < 20*60: ph='POWER_HOUR'
        else: ph='CLOSE'
        if ph in ('OPEN','LUNCH','CLOSE'): continue
        
        if hv_z[i] < -1.0 or hv_z[i] > 2.0: continue
        if np.isnan(zf[i]) or zf[i] > 0.5: continue
        if abs(ofs[i]) <= 1.0 or abs(ofs[i]) < ofs_th: continue
        
        dirn = 'buy' if sd[i] > 0 else 'sell'
        entry = p[i]
        trail_dist = atr[i] * trail_atr_mult
        sl_init = entry - trail_dist if dirn == 'buy' else entry + trail_dist
        
        best_price = entry
        exit_pl = None; win = None
        max_bars = 24  # 6 hours max hold
        
        for j in range(1, min(max_bars, n-i)):
            ci = i + j
            if dirn == 'buy':
                best_price = max(best_price, h[ci])
                trail_stop = best_price - trail_dist
                if l[ci] <= trail_stop:
                    exit_pl = trail_stop - entry
                    win = exit_pl > 0
                    break
                if (best_price - entry) / (entry * trail_atr_mult * 0.01) >= rr_target:
                    tp = entry + trail_dist * rr_target * 0.1  # scaled TP
                    if h[ci] >= tp:
                        exit_pl = tp - entry
                        win = True
                        break
            else:
                best_price = min(best_price, l[ci])
                trail_stop = best_price + trail_dist
                if h[ci] >= trail_stop:
                    exit_pl = entry - trail_stop
                    win = exit_pl > 0
                    break
                if (entry - best_price) / (entry * trail_atr_mult * 0.01) >= rr_target:
                    tp = entry - trail_dist * rr_target * 0.1
                    if l[ci] <= tp:
                        exit_pl = entry - tp
                        win = True
                        break
        
        if exit_pl is not None:
            sigs.append({'pl': exit_pl, 'win': win, 'entry': entry, 'dir': dirn})
    
    return sigs

# ===== SWEEP =====
print('\n=== BACKTEST V2: TRAILING STOP + ATR DYNAMIC ===')
results = []
for ofs_th in [x*0.1 for x in range(5, 26)]:
    for rr in [1.5, 2.0, 2.5, 3.0]:
        sigs = backtest_v2(ofs_th, rr, trail_atr_mult=2.0)
        nw = sum(1 for s in sigs if s['win'])
        nl = sum(1 for s in sigs if not s['win'])
        nc = nw + nl
        if nc < 15: continue
        wr = nw/nc*100
        aw = np.mean([s['pl'] for s in sigs if s['win']]) if nw>0 else 0
        al = abs(np.mean([s['pl'] for s in sigs if not s['win']])) if nl>0 else 1
        pf = (nw*aw)/(nl*al) if nl>0 and al>0 else 0
        results.append({'ofs':ofs_th,'rr':rr,'sig':nc,'wr':wr,'pf':pf,'aw':aw,'al':al})

results.sort(key=lambda r: -r['wr']*r['pf'])
print(f'{"Rank":>4} {"OFS":>4} {"RR":>4} {"N":>5} {"WR%":>6} {"PF":>6} {"AW":>8} {"AL":>8}')
for i, r in enumerate(results[:20]):
    print(f'{i+1:4} {r["ofs"]:4.1f} {r["rr"]:4.1f} {r["sig"]:5} {r["wr"]:5.1f}% {r["pf"]:6.2f} {r["aw"]:8.2f} {r["al"]:8.2f}')

# Check specific targets
print(f'\n=== TARGET CHECK ===')
for r in results:
    if r['wr'] >= 45 and r['pf'] >= 2.0:
        print(f'  NEAR: OFS={r["ofs"]:.1f} RR={r["rr"]:.1f} WR={r["wr"]:.1f}% PF={r["pf"]:.2f} ({r["sig"]} trades)')

print(f'\nBest WR: {max((r for r in results if r["sig"]>=30), key=lambda r:r["wr"])}')
print(f'Best PF: {max((r for r in results if r["sig"]>=30), key=lambda r:r["pf"])}')
print(f'Best WR*PF: {max(results, key=lambda r:r["wr"]*r["pf"])}')
