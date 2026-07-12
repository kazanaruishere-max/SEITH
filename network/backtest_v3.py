#!/usr/bin/env python3
"""Backtest v3: optimize trailing stop + ATR multiplier."""

import json, numpy as np
from datetime import datetime

with open('network/us100_m15_1y.json') as f:
    raw = json.load(f)
print(f'Loaded {len(raw)} bars')

p = np.array([x['close'] for x in raw], dtype=float)
h = np.array([x['high'] for x in raw], dtype=float)
l = np.array([x['low'] for x in raw], dtype=float)
v = np.array([x['volume'] for x in raw], dtype=float)
times = np.array([x['time'] for x in raw], dtype=int)
n = len(p)

# Indicators
hv_z = np.zeros(n)
ret = np.diff(p)/(p[:-1]+1e-10)
for i in range(20, len(ret)):
    s = ret[i-20:i].std()
    hv_z[i] = (ret[i-20:i][-1]-ret[i-20:i].mean())/s if s>1e-10 else 0

zf = np.full(n, np.nan)
for i in range(14, n):
    w = p[i-14:i+1]; r1 = max(w[:8].max()-w[:8].min(),.001)
    r2 = max(w[7:].max()-w[7:].min(),.001)
    r3 = max(w.max()-w.min(),.001)
    D = np.clip((np.log(r1+r2)-np.log(r3))/np.log(2)+1,1,2)
    a = np.exp(-4.6*(D-1))
    f = a*p[i]+(1-a)*p[i-1]
    s10 = p[max(0,i-9):i+1].std()
    zf[i] = (p[i]-f)/s10 if s10>1e-10 else 0

ratio = np.where(h-l>1e-10, (p-l)/(h-l), 0.5)
sd = 2*(ratio-0.5)
rd = sd*v; cvd = np.cumsum(rd)
cvn = np.zeros(n)
for i in range(20, n):
    s = cvd[i-20:i+1].std()
    z = (cvd[i-20:i+1][-1]-cvd[i-20:i+1].mean())/s if s>1e-10 else 0
    cvn[i] = np.tanh(z/3)
sdm = np.zeros(n)
for i in range(20, n):
    pos = (p[i]-p[i-20])/(p[i-20:i].max()-p[i-20:i].min()+1e-10)
    sdm[i] = np.clip((0.5-pos)*2, -1, 1)
ofs = sd+cvn+sdm

atr = np.zeros(n)
for i in range(14, n):
    tr = max(h[i]-l[i], abs(h[i]-p[i-1]), abs(l[i]-p[i-1]))
    atr[i] = (atr[i-1]*13 + tr)/14 if i > 14 else tr

def backtest(ofs_th, trail_mult, min_rr=2.0):
    sigs = []
    for i in range(60, n):
        ts = times[i]; d = datetime.fromtimestamp(ts)
        if d.weekday() >= 5: continue
        mins = d.hour*60 + d.minute
        if not (13*60+30 <= mins < 20*60): continue
        if mins < 14*60 or (16*60+30 <= mins < 18*60) or mins >= 20*60: continue
        if hv_z[i] < -1.0 or hv_z[i] > 2.0: continue
        if np.isnan(zf[i]) or zf[i] > 0.5: continue
        if abs(ofs[i]) <= 1.0 or abs(ofs[i]) < ofs_th: continue
        
        dirn = 'buy' if sd[i] > 0 else 'sell'
        entry = p[i]
        trail = atr[i] * trail_mult
        best = entry
        
        for j in range(1, min(32, n-i)):
            ci = i+j
            if dirn == 'buy':
                best = max(best, h[ci])
                if l[ci] <= best - trail:
                    pl = (best - trail) - entry
                    sigs.append({'pl': pl, 'win': pl > 0, 'entry': entry, 'dir': dirn})
                    break
            else:
                best = min(best, l[ci])
                if h[ci] >= best + trail:
                    pl = entry - (best + trail)
                    sigs.append({'pl': pl, 'win': pl > 0, 'entry': entry, 'dir': dirn})
                    break
    return sigs

print('\n=== TRAILING STOP OPTIMIZATION ===')
print('TrailMult × OFS threshold sweep\n')
results = []
for tm in [0.5, 0.75, 1.0, 1.25, 1.5, 2.0, 2.5, 3.0]:
    for ot in [x*0.1 for x in range(12, 28)]:
        sigs = backtest(ot, tm)
        nw = sum(1 for s in sigs if s['win']); nl = len(sigs)-nw
        if nw+nl < 20: continue
        wr = nw/(nw+nl)*100
        aw = np.mean([s['pl'] for s in sigs if s['win']]) if nw>0 else 0
        al = abs(np.mean([s['pl'] for s in sigs if not s['win']])) if nl>0 else 1
        pf = (nw*aw)/(nl*al) if nl>0 and al>0 else 0
        results.append({'tm':tm,'ofs':ot,'n':nw+nl,'wr':wr,'pf':pf,'aw':aw,'al':al})

results.sort(key=lambda r: -r['wr']*r['pf'])
print(f'{"TM":>5} {"OFS":>4} {"N":>5} {"WR%":>6} {"PF":>6} {"AW":>7} {"AL":>7}')
for r in results[:25]:
    print(f'{r["tm"]:5.1f} {r["ofs"]:4.1f} {r["n"]:5} {r["wr"]:5.1f}% {r["pf"]:6.2f} {r["aw"]:7.1f} {r["al"]:7.1f}')

# Target analysis
print(f'\n=== TARGET: WR>=50% + PF>=3.0 ===')
hits = [r for r in results if r['wr']>=50 and r['pf']>=3.0 and r['n']>=20]
if hits:
    for r in sorted(hits, key=lambda x:-x['wr']*x['pf'])[:5]:
        print(f'  TM={r["tm"]:.1f} OFS={r["ofs"]:.1f} n={r["n"]} WR={r["wr"]:.1f}% PF={r["pf"]:.2f}')
else:
    print('No configuration reaches target')
    # Show closest
    best = max(results, key=lambda r: (r['wr']/50)*(r['pf']/3.0) if r['wr']>=30 else 0)
    print(f'Closest: TM={best["tm"]:.1f} OFS={best["ofs"]:.1f} WR={best["wr"]:.1f}% PF={best["pf"]:.2f} n={best["n"]}')
