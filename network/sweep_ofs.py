#!/usr/bin/env python3
"""OFX threshold sweep on live US100 M15 data."""

import sys, json, numpy as np
sys.path.insert(0, 'python/python')
from seith_bridge.mt5 import init_mt5, get_rates_json, shutdown
import MetaTrader5 as mt5

init_mt5()
rj = get_rates_json('US100', 500, mt5.TIMEFRAME_M15)
if not rj:
    print('No data')
    exit()
rates = json.loads(rj)

p = np.array([x['close'] for x in rates], dtype=float)
h = np.array([x['high'] for x in rates], dtype=float)
l = np.array([x['low'] for x in rates], dtype=float)
v = np.array([x['volume'] for x in rates], dtype=float)
n = len(p)

# HV
hv_z = np.zeros(n)
ret = np.diff(p)/(p[:-1]+1e-10)
for i in range(20, len(ret)):
    seg = ret[i-20:i]; s = seg.std()
    hv_z[i] = (seg[-1]-seg.mean())/s if s>1e-10 else 0

# FRAMA
zf = np.full(n, np.nan)
for i in range(14, n):
    w = p[i-14:i+1]
    r1 = max(w[:8].max()-w[:8].min(), .001)
    r2 = max(w[7:].max()-w[7:].min(), .001)
    r3 = max(w.max()-w.min(), .001)
    D = np.clip((np.log(r1+r2)-np.log(r3))/np.log(2)+1, 1, 2)
    alpha = np.exp(-4.6*(D-1))
    f = alpha*p[i]+(1-alpha)*p[i-1]
    s10 = p[max(0,i-9):i+1].std()
    zf[i] = (p[i]-f)/s10 if s10>1e-10 else 0

# OFS
ratio = np.where(h-l>1e-10, (p-l)/(h-l), 0.5)
sd = 2*(ratio-0.5)
rd = sd*v; cvd = np.cumsum(rd)
cvn = np.zeros(n)
for i in range(20, n):
    seg = cvd[i-20:i+1]; s = seg.std()
    z = (seg[-1]-seg.mean())/s if s>1e-10 else 0
    cvn[i] = np.tanh(z/3)
sdm = np.zeros(n)
for i in range(20, n):
    pos = (p[i]-p[i-20])/(p[i-20:i].max()-p[i-20:i].min()+1e-10)
    sdm[i] = np.clip((0.5-pos)*2, -1, 1)
ofs = sd+cvn+sdm

# Pre-filter: bars that pass HV + FRAMA (Gate 1 + 2)
valid = []
for i in range(60, n):
    if hv_z[i] >= -1.0 and hv_z[i] <= 2.0:
        if not np.isnan(zf[i]) and zf[i] <= 0.5:
            valid.append(abs(ofs[i]))

valid = np.array(valid)
print(f'Bars passing G1+G2: {len(valid)} of {n-60}')
print(f'\nOFS |abs| distribution on LIVE M15:')
for pt in [50, 60, 70, 80, 85, 90, 95, 99]:
    print(f'  P{pt}: {np.percentile(valid, pt):.2f}')

# How many signals per threshold
header = f'{"Thresh":>6} {"Signals":>8} {"Density":>9}'
print(f'\n{header}')
for th in [x*0.1 for x in range(5, 35)]:
    hits = int(sum(1 for o in valid if o > 1.0 and o >= th))
    d = hits/len(valid)*100 if len(valid)>0 else 0
    print(f'  {th:4.1f} {hits:8} {d:8.2f}%')

shutdown()
