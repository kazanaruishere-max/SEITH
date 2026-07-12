#!/usr/bin/env python3
"""Backtest v4: hybrid trailing — enter trail only after profit buffer."""

import json, numpy as np
from datetime import datetime

with open('network/us100_m15_1y.json') as f:
    raw = json.load(f)
print(f'Loaded {len(raw)} bars')

p = np.array([x['close'] for x in raw]); h = np.array([x['high'] for x in raw])
l = np.array([x['low'] for x in raw]); v = np.array([x['volume'] for x in raw], dtype=float)
times = np.array([x['time'] for x in raw], dtype=int); n = len(p)

# Indicators (same as v3, shortened)
hv_z = np.zeros(n)
ret = np.diff(p)/(p[:-1]+1e-10)
for i in range(20, len(ret)):
    s = ret[i-20:i].std()
    hv_z[i] = (ret[i-20:i][-1]-ret[i-20:i].mean())/s if s>1e-10 else 0

zf = np.full(n, np.nan)
for i in range(14, n):
    w = p[i-14:i+1]; r1 = max(w[:8].max()-w[:8].min(),.001)
    r2 = max(w[7:].max()-w[7:].min(),.001); r3 = max(w.max()-w.min(),.001)
    D = np.clip((np.log(r1+r2)-np.log(r3))/np.log(2)+1,1,2)
    a = np.exp(-4.6*(D-1)); f = a*p[i]+(1-a)*p[i-1]
    s10 = p[max(0,i-9):i+1].std()
    zf[i] = (p[i]-f)/s10 if s10>1e-10 else 0

ratio = np.where(h-l>1e-10, (p-l)/(h-l), 0.5)
sd = 2*(ratio-0.5); rd = sd*v; cvd = np.cumsum(rd)
cvn = np.zeros(n)
for i in range(20, n):
    s = cvd[i-20:i+1].std()
    z = (cvd[i-20:i+1][-1]-cvd[i-20:i+1].mean())/s if s>1e-10 else 0
    cvn[i] = np.tanh(z/3)
sdm = np.zeros(n)
for i in range(20, n):
    pos = (p[i]-p[i-20])/(p[i-20:i].max()-p[i-20:i].min()+1e-10)
    sdm[i] = np.clip((0.5-pos)*2,-1,1)
ofs = sd+cvn+sdm

atr = np.zeros(n)
for i in range(14, n):
    tr = max(h[i]-l[i], abs(h[i]-p[i-1]), abs(l[i]-p[i-1]))
    atr[i] = (atr[i-1]*13+tr)/14 if i>14 else tr

def backtest(ofs_th, init_sl_mult=1.5, activate_trail_mult=0.5, trail_mult=1.0):
    """Hybrid trailing approach."""
    sigs = []
    for i in range(60, n):
        ts = times[i]; d = datetime.fromtimestamp(ts)
        if d.weekday() >= 5: continue
        mins = d.hour*60+d.minute
        if not (13*60+30 <= mins < 20*60): continue
        if mins < 14*60 or (16*60+30 <= mins < 18*60) or mins >= 20*60: continue
        if hv_z[i] < -1.0 or hv_z[i] > 2.0: continue
        if np.isnan(zf[i]) or zf[i] > 0.5: continue
        if abs(ofs[i]) <= 1.0 or abs(ofs[i]) < ofs_th: continue
        
        dirn = 'buy' if sd[i] > 0 else 'sell'
        entry = p[i]
        init_sl = atr[i] * init_sl_mult
        trail = atr[i] * trail_mult
        activate_dist = atr[i] * activate_trail_mult
        trailing_active = False
        best = entry
        
        for j in range(1, min(48, n-i)):
            ci = i+j
            if dirn == 'buy':
                best = max(best, h[ci])
                if not trailing_active:
                    if h[ci] >= entry + activate_dist:
                        trailing_active = True
                        best = h[ci]
                    elif l[ci] <= entry - init_sl:
                        sigs.append({'pl': (entry - init_sl) - entry, 'win': False, 'entry': entry})
                        break
                else:
                    if l[ci] <= best - trail:
                        sigs.append({'pl': (best - trail) - entry, 'win': True})
                        break
            else:
                best = min(best, l[ci])
                if not trailing_active:
                    if l[ci] <= entry - activate_dist:
                        trailing_active = True
                        best = l[ci]
                    elif h[ci] >= entry + init_sl:
                        sigs.append({'pl': entry - (entry + init_sl), 'win': False, 'entry': entry})
                        break
                else:
                    if h[ci] >= best + trail:
                        sigs.append({'pl': entry - (best + trail), 'win': True})
                        break
    return sigs

print('\n=== HYBRID TRAILING: INIT SL + ACTIVATE THRESHOLD + TRAIL ===')
results = []
for ofs_th in [x*0.1 for x in range(12, 28)]:
    for init_mult in [0.8, 1.0, 1.5, 2.0]:
        for act_mult in [0.3, 0.5, 0.75]:
            sigs = backtest(ofs_th, init_mult, act_mult, trail_mult=1.0)
            nw = sum(1 for s in sigs if s['win']); nl = len(sigs)-nw
            if nw+nl < 20: continue
            wr = nw/(nw+nl)*100
            aw = np.mean([s['pl'] for s in sigs if s['win']]) if nw>0 else 0
            al = abs(np.mean([s['pl'] for s in sigs if not s['win']])) if nl>0 else 1
            pf = (nw*aw)/(nl*al) if nl>0 and al>0 else 0
            results.append({'ofs':ofs_th,'init':init_mult,'act':act_mult,'n':nw+nl,'wr':wr,'pf':pf,'aw':aw,'al':al,'rr':aw/al if al>0 else 0})

results.sort(key=lambda r: -r['wr']*r['pf'])
print(f'{"OFS":>4} {"Init":>4} {"Act":>4} {"N":>5} {"WR%":>6} {"PF":>6} {"AW":>7} {"AL":>7} {"RR":>5}')
for r in results[:30]:
    print(f'{r["ofs"]:4.1f} {r["init"]:4.1f} {r["act"]:4.2f} {r["n"]:5} {r["wr"]:5.1f}% {r["pf"]:6.2f} {r["aw"]:7.1f} {r["al"]:7.1f} {r["rr"]:5.2f}')

print(f'\n=== TARGET: WR>=50% + PF>=3.0 ===')
hits = [r for r in results if r['wr']>=50 and r['pf']>=3.0 and r['n']>=20]
if hits:
    for r in sorted(hits, key=lambda x:-x['wr']*x['pf'])[:5]:
        print(f'  OFS={r["ofs"]:.1f} Init={r["init"]:.1f} Act={r["act"]:.2f} n={r["n"]} WR={r["wr"]:.1f}% PF={r["pf"]:.2f} RR={r["rr"]:.2f}')
else:
    print('None found.')
    best = max(results, key=lambda r: r['wr']*r['pf'])
    print(f'Best: OFS={best["ofs"]:.1f} Init={best["init"]:.1f} Act={best["act"]:.2f} n={best["n"]} WR={best["wr"]:.1f}% PF={best["pf"]:.2f} RR={best["rr"]:.2f}')
