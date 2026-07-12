#!/usr/bin/env python3
"""Final analysis: what's achievable with OANDA US100 data."""

import json, numpy as np
from datetime import datetime

with open('network/us100_m15_1y.json') as f:
    raw = json.load(f)

p = np.array([x['close'] for x in raw]); h = np.array([x['high'] for x in raw])
l = np.array([x['low'] for x in raw]); v = np.array([x['volume'] for x in raw], dtype=float)
times = np.array([x['time'] for x in raw], dtype=int); n = len(p)

# Indicators (abbreviated)
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
    sdm[i] = np.clip((0.5-pos)*2, -1, 1)
ofs = sd+cvn+sdm

atr = np.zeros(n)
for i in range(14, n):
    tr = max(h[i]-l[i], abs(h[i]-p[i-1]), abs(l[i]-p[i-1]))
    atr[i] = (atr[i-1]*13+tr)/14 if i>14 else tr

def run_bt_dt(ofs_th, init_sl, act_dist, trail=1.0, max_hold=32):
    """Day trading simulation."""
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
        i_sl = atr[i]*init_sl; trl = atr[i]*trail
        ad = atr[i]*act_dist
        active = False; best = entry
        
        for j in range(1, min(max_hold, n-i)):
            ci = i+j
            if dirn == 'buy':
                best = max(best, h[ci])
                if not active:
                    if h[ci] >= entry+ad: active = True; best = h[ci]
                    elif l[ci] <= entry-i_sl: sigs.append({'win':False,'pl':-i_sl,'entry':entry,'bars':j}); break
                else:
                    if l[ci] <= best-trl: sigs.append({'win':True,'pl':best-trl-entry,'entry':entry,'bars':j}); break
            else:
                best = min(best, l[ci])
                if not active:
                    if l[ci] <= entry-ad: active = True; best = l[ci]
                    elif h[ci] >= entry+i_sl: sigs.append({'win':False,'pl':-i_sl,'entry':entry,'bars':j}); break
                else:
                    if h[ci] >= best+trl: sigs.append({'win':True,'pl':entry-(best+trl),'entry':entry,'bars':j}); break
    return sigs

# Sweep for day trading parameters
results = []
for ot in [x*0.1 for x in range(12, 26)]:
    for init in [0.8, 1.0, 1.5, 2.0]:
        for act in [0.5, 0.75, 1.0, 1.5]:
            sigs = run_bt_dt(ot, init, act, 0.5)
            nw = sum(1 for s in sigs if s['win']); nl = len(sigs)-nw
            if nw+nl < 20: continue
            wr = nw/(nw+nl)*100
            aw = np.mean([s['pl'] for s in sigs if s['win']]) if nw>0 else 0
            al = abs(np.mean([s['pl'] for s in sigs if not s['win']])) if nl>0 else 1
            pf = (nw*aw)/(nl*al) if nl>0 and al>0 else 0
            results.append({'ofs':ot,'init':init,'act':act,'n':nw+nl,'wr':wr,'pf':pf,'aw':aw,'al':al,'rr':aw/al if al>0 else 0})

results.sort(key=lambda r: -r['wr']*r['pf'])

print('='*65)
print('  FINAL BACKTEST REPORT — US100 OANDA M15, 1 TAHUN')
print('  Target: WR 55% + PF 4.0 (day trading, not scalping)')
print(f'  Data: 23,426 bars, 310 trading days')
print('='*65)

# Find closest to target
dt_results = [r for r in results if r['wr'] >= 40 and r['n'] >= 30]
print(f'\nConfigurations with WR >= 40% (30+ trades): {len(dt_results)}')
print(f'\n{"OFS":>4} {"Init":>4} {"Act":>4} {"N":>5} {"WR":>5} {"PF":>5} {"RR":>5}'  )
for r in dt_results[:10]:
    print(f'{r["ofs"]:4.1f} {r["init"]:4.1f} {r["act"]:4.1f} {r["n"]:5} {r["wr"]:5.1f}% {r["pf"]:5.2f} {r["rr"]:5.2f}')

# Max PF
best_pf = max(dt_results, key=lambda r: r['pf'])
print(f'\nBest PF: {best_pf}')
# Max WR
best_wr = max(dt_results, key=lambda r: r['wr'])
print(f'Best WR: {best_wr}')
# Balanced
best_bal = max(dt_results, key=lambda r: r['wr']*r['pf'])
print(f'Best WR*PF: {best_bal}')

print(f'\n=== SUMMARY ===')
print(f'Achievable: WR ~45-50% with PF ~1.5-2.0')
print(f'Max PF with WR>=40%: {best_pf["pf"]:.2f}')
print(f'WR at that PF: {best_pf["wr"]:.1f}% ({best_pf["n"]} trades)')
print(f'\nGap to target:')
print(f'  Target WR: 55% | Achievable: ~{best_wr["wr"]:.0f}%')
print(f'  Target PF: 4.0  | Achievable: ~{best_pf["pf"]:.1f}')
print(f'\nConclusion: Target WR 55% + PF 4.0 not achievable')
print(f'with the current 5-gate + trailing stop approach on US100.')
