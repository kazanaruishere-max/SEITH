#!/usr/bin/env python3
"""US100 parameter sweep: find optimal OFS threshold + scorecard."""

import numpy as np
import pandas as pd
import yfinance as yf
from datetime import datetime, timedelta
import warnings
warnings.filterwarnings('ignore')

END = datetime(2026, 7, 10)
START = END - timedelta(days=45)
print(f'Period: {START.date()} to {END.date()}')

# 1. FETCH
qqq = yf.download('QQQ', start=START.strftime('%Y-%m-%d'), end=END.strftime('%Y-%m-%d'), interval='15m', progress=False)
df = qqq.copy()
if isinstance(df.columns, pd.MultiIndex):
    df.columns = df.columns.droplevel(1)
df.index = pd.to_datetime(df.index, utc=True)
df['date'] = df.index.date
df['hour_utc'] = df.index.hour
df['min_utc'] = df.index.minute

def phase(r):
    m = r['hour_utc']*60 + r['min_utc']
    if m < 14*60: return 'OPEN'
    if m < 16*60+30: return 'NORMAL'
    if m < 18*60: return 'LUNCH'
    if m < 19*60+30: return 'NORMAL'
    if m < 20*60: return 'POWER_HOUR'
    return 'CLOSE'
df['phase'] = df.apply(phase, axis=1)
n = len(df)
print(f'Bars: {n}, Days: {df["date"].nunique()}')

# 2. MACRO (skip FOMC/CPI/NFP)
macro_dates = {'FOMC':['2025-07-30','2025-09-17','2025-11-07','2025-12-17','2026-01-28','2026-03-18','2026-05-06','2026-06-17'],
    'CPI':['2025-08-13','2025-09-11','2025-10-15','2025-11-13','2025-12-11','2026-01-14','2026-02-12','2026-03-12','2026-04-10','2026-05-13','2026-06-11'],
    'NFP':['2025-08-01','2025-09-05','2025-10-03','2025-11-07','2025-12-05','2026-01-09','2026-02-06','2026-03-06','2026-04-03','2026-05-08','2026-06-05']}
red = set()
for _, dates in macro_dates.items():
    for x in dates: red.add(x)
df['is_red'] = df['date'].astype(str).isin(red)
print(f'Macro RED: {df["is_red"].sum()} bars')

# 3. INDICATORS
p = df['Close'].values.astype(float)
h = df['High'].values.astype(float)
l = df['Low'].values.astype(float)
v = df['Volume'].values.astype(float)

# HV Z-Score
hv_z = np.zeros(n)
ret = np.diff(p) / (p[:-1] + 1e-10)
for i in range(20, len(ret)):
    s = ret[i-20:i].std()
    hv_z[i] = (ret[i-20:i][-1] - ret[i-20:i].mean()) / s if s > 1e-10 else 0.0
df['hv_z'] = hv_z

# FRAMA
zf = np.full(n, np.nan)
for i in range(14, n):
    w = p[i-14:i+1]
    half = 7
    n1, x1 = w[:8].min(), w[:8].max()
    n2, x2 = w[7:].min(), w[7:].max()
    n3, x3 = w.min(), w.max()
    r1, r2, r3 = max(x1-n1,.001), max(x2-n2,.001), max(x3-n3,.001)
    D = np.clip((np.log(r1+r2)-np.log(r3))/np.log(2)+1, 1, 2)
    alpha = np.exp(-4.6*(D-1))
    prev = p[i-1]
    f = alpha*p[i] + (1-alpha)*prev
    std10 = p[max(0,i-9):i+1].std()
    zf[i] = (p[i]-f)/std10 if std10 > 1e-10 else 0.0

# OFS proxy (volume-weighted delta)
ratio = np.where(h-l>1e-10, (p-l)/(h-l), 0.5)
s_delta = 2*(ratio-0.5)
df['s_delta'] = s_delta
raw_delta = s_delta * v
cvd = np.cumsum(raw_delta)
cvd_n = np.zeros(n)
for i in range(20, n):
    s = cvd[i-20:i+1].std()
    z = (cvd[i-20:i+1][-1]-cvd[i-20:i+1].mean())/s if s>1e-10 else 0
    cvd_n[i] = np.tanh(z/3)

s_dom = np.zeros(n)
for i in range(20, n):
    pos = (p[i]-p[i-20])/(p[i-20:i].max()-p[i-20:i].min()+1e-10)
    s_dom[i] = np.clip((0.5-pos)*2, -1, 1)

df['ofs'] = s_delta + cvd_n + s_dom

print(f'OFS range: {df["ofs"].min():.2f} to {df["ofs"].max():.2f}')
print(f'OFS abs percentiles:')
for ptile in [50, 60, 70, 80, 90, 95, 99]:
    print(f'  P{ptile}: {np.percentile(df["ofs"].abs(), ptile):.2f}')

# 4. PARAMETER SWEEP: find optimal OFS threshold
results = []
ofs_passing = []
print('\n=== PARAMETER SWEEP: OFS threshold ===')
print(f'{"Thresh":>6} {"Sig":>5} {"Comp":>5} {"WR%":>6} {"PF":>6}')
for th in [x*0.1 for x in range(5, 35)]:
    sigs = 0; wins = 0; losses = 0; pl = 0.0
    for i in range(40, n):
        bar = df.iloc[i]; dt = df.index[i]
        if dt.weekday() >= 5: continue
        if bar['phase'] in ('OPEN','LUNCH','CLOSE'): continue
        if bar['is_red']: continue
        hv = bar['hv_z']
        if hv < -1.0 or hv > 2.0: continue
        z = zf[i]; zf_int = float(z) if not np.isnan(z) else 0.0
        if np.isnan(z) or zf_int > 0.5: continue
        if abs(bar['ofs']) <= 1.0 or abs(bar['ofs']) < th: continue

        sigs += 1
        d = 'buy' if bar['s_delta'] > 0 else 'sell'
        entry = bar['Close']
        sl = entry*0.995 if d=='buy' else entry*1.005
        tp = entry*1.012 if d=='buy' else entry*0.988
        for j in range(1, min(20, n-i)):
            b = df.iloc[i+j]
            if d == 'buy':
                if b['High'] >= tp: wins+=1; pl+=(tp-entry); break
                if b['Low'] <= sl: losses+=1; pl+=(sl-entry); break
            else:
                if b['Low'] <= tp: wins+=1; pl+=(entry-tp); break
                if b['High'] >= sl: losses+=1; pl+=(entry-sl); break

    comp = wins + losses
    wr = wins / comp * 100 if comp > 0 else 0
    avg_rr = 1.2 / 0.5  # ~2.4 (from earlier backtest: avg win 8.27 / avg loss 3.42)
    pf_est = (wr/100 * avg_rr) / ((100-wr)/100) if wr < 100 else 999
    results.append({'th': th, 'sigs': sigs, 'comp': comp, 'wr': wr, 'pf': pf_est})

# Best threshold by WR
best = max(results, key=lambda r: r['wr'] if r['comp']>=10 else 0)
print(f'\nBest threshold by WR: {best["th"]} -> WR={best["wr"]:.1f}%, sigs={best["sigs"]}, comp={best["comp"]}')

# Show all
for r in sorted(results, key=lambda x:-x['wr']):
    marker = '*' if abs(r['th']-best['th'])<0.01 else ' '
    print(f'{marker} {r["th"]:5.1f} {r["sigs"]:5} {r["comp"]:5} {r["wr"]:5.1f}%  {r["pf"]:5.2f}')
