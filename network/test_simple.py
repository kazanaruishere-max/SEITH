#!/usr/bin/env python3
import sys, json, numpy as np
sys.path.insert(0, 'python/python')
from seith_bridge.mt5 import init_mt5
import MetaTrader5 as mt5

init_mt5()
print("Fetching data...", flush=True)
rates = mt5.copy_rates_from_pos('US100', mt5.TIMEFRAME_M15, 0, 500)
if rates is None:
    print("No data")
    mt5.shutdown()
    exit()
print(f"Got {len(rates)} bars", flush=True)

# Quick analysis
p = np.array([r['close'] for r in rates], dtype=float)
print(f"Price range: {p.min():.2f} - {p.max():.2f}", flush=True)

# OFS
h = np.array([r['high'] for r in rates], dtype=float)
l = np.array([r['low'] for r in rates], dtype=float)
v = np.array([r['tick_volume'] for r in rates], dtype=float)
n = len(p)

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

print(f"OFS range: {ofs.min():.2f} to {ofs.max():.2f}", flush=True)
for pt in [50,70,80,90,95,99]:
    print(f"  P{pt}: {np.percentile(abs(ofs)[60:], pt):.2f}", flush=True)

print("Done.", flush=True)
mt5.shutdown()
