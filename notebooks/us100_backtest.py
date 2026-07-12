#!/usr/bin/env python3
"""AI SEITH US100 - Backtest with M15 data (60-day window)."""

import numpy as np
import pandas as pd
import yfinance as yf
from datetime import datetime, timedelta
import warnings
warnings.filterwarnings('ignore')

# === CONFIG ===
END = datetime(2026, 7, 10)  # Friday (last trading day)
START = END - timedelta(days=45)
START_STR = START.strftime('%Y-%m-%d')
END_STR = END.strftime('%Y-%m-%d')

HV_W = 20
FRAMA_P = 14
CVD_W = 20
VWAP_BAND = 2.5

print(f'Period: {START_STR} to {END_STR}')

# 1. FETCH DATA
print('Fetching QQQ M15...')
qqq = yf.download('QQQ', start=START_STR, end=END_STR, interval='15m', progress=False)
print(f'Rows: {len(qqq)}')

vix = yf.download('^VIX', start=START_STR, end=END_STR, progress=False)
tnx = yf.download('^TNX', start=START_STR, end=END_STR, progress=False)
fvx = yf.download('^FVX', start=START_STR, end=END_STR, progress=False)

# 2. PREPARE
df = qqq.copy()
if isinstance(df.columns, pd.MultiIndex):
    df.columns = df.columns.droplevel(1)
df.index = pd.to_datetime(df.index, utc=True)
df['date'] = df.index.date
df['hour_utc'] = df.index.hour
df['min_utc'] = df.index.minute

# Session phases for M15 bars
# Market 13:30-20:00 UTC. OPEN 13:30-14:00, NORMAL 14:00-16:30,
# LUNCH 16:30-18:00, NORMAL 18:00-19:30, POWER_HOUR 19:30-20:00, CLOSE 20:00-20:30
def detect_phase(row):
    h, m = row['hour_utc'], row['min_utc']
    mins = h * 60 + m
    if mins < 14*60:     return 'OPEN'
    if mins < 16*60+30:  return 'NORMAL'
    if mins < 18*60:     return 'LUNCH'
    if mins < 19*60+30:  return 'NORMAL'
    if mins < 20*60:     return 'POWER_HOUR'
    return 'CLOSE'

df['phase'] = df.apply(detect_phase, axis=1)
n = len(df)

print(f'Bars: {n}, Days: {df["date"].nunique()}')
print(f'Period: {df.index.min()} -> {df.index.max()}')
print(df['phase'].value_counts())

# 3. MACRO CALENDAR
macro_events = [
    ('FOMC', ['2025-07-30', '2025-09-17', '2025-11-07', '2025-12-17',
              '2026-01-28', '2026-03-18', '2026-05-06', '2026-06-17']),
    ('CPI',  ['2025-08-13', '2025-09-11', '2025-10-15', '2025-11-13', '2025-12-11',
              '2026-01-14', '2026-02-12', '2026-03-12', '2026-04-10', '2026-05-13', '2026-06-11']),
    ('NFP',  ['2025-08-01', '2025-09-05', '2025-10-03', '2025-11-07', '2025-12-05',
              '2026-01-09', '2026-02-06', '2026-03-06', '2026-04-03', '2026-05-08', '2026-06-05']),
]
red_dates = set()
for _, dates in macro_events:
    for d in dates:
        red_dates.add(d)
df['is_red'] = df['date'].astype(str).isin(red_dates)
print(f'Red dates: {len(red_dates)}, red bars: {df["is_red"].sum()}')

# 4. INDICATORS
p = df['Close'].values.astype(float)
h = df['High'].values.astype(float)
l = df['Low'].values.astype(float)
v = df['Volume'].values.astype(float)

# 4a. HV Z-Score
hv_z = np.zeros(n)
ret = np.diff(p) / (p[:-1] + 1e-10)
for i in range(HV_W, len(ret)):
    seg = ret[i-HV_W:i]
    s = seg.std()
    hv_z[i] = (seg[-1] - seg.mean()) / s if s > 1e-10 else 0.0
df['hv_z'] = hv_z

# 4b. FRAMA
frama = np.full(n, np.nan)
zf = np.full(n, np.nan)
for i in range(FRAMA_P, n):
    w = p[i-FRAMA_P:i+1]
    half = FRAMA_P // 2
    n1, x1 = w[:half+1].min(), w[:half+1].max()
    n2, x2 = w[half:].min(), w[half:].max()
    n3, x3 = w.min(), w.max()
    r1 = max(x1 - n1, 0.001)
    r2 = max(x2 - n2, 0.001)
    r3 = max(x3 - n3, 0.001)
    D = (np.log(r1+r2) - np.log(r3)) / np.log(2) + 1.0
    D = np.clip(D, 1.0, 2.0)
    alpha = np.exp(-4.6 * (D - 1))
    prev = frama[i-1] if not np.isnan(frama[i-1]) else p[i]
    frama[i] = alpha * p[i] + (1 - alpha) * prev
    w10 = p[max(0, i-9):i+1]
    s = w10.std()
    zf[i] = (p[i] - frama[i]) / s if s > 1e-10 else 0.0
df['z_frama'] = zf

# 4c. OFS Proxy
ratio = np.where(h - l > 1e-10, (p - l) / (h - l), 0.5)
s_delta = 2.0 * (ratio - 0.5)
raw_delta = s_delta * v
cvd = np.cumsum(raw_delta)
cvd_norm = np.zeros(n)
for i in range(CVD_W, n):
    seg = cvd[i-CVD_W:i+1]
    s = seg.std()
    z = (seg[-1] - seg.mean()) / s if s > 1e-10 else 0.0
    cvd_norm[i] = np.tanh(z / 3.0)

s_dom = np.zeros(n)
for i in range(20, n):
    seg_p = p[i-20:i]
    seg_v = v[i-20:i]
    if seg_v.sum() < 1e-10: continue
    pos = (p[i] - seg_p.min()) / (seg_p.max() - seg_p.min() + 1e-10)
    s_dom[i] = np.clip((0.5 - pos) * 2, -1, 1)

df['s_delta'] = s_delta
df['cvd_norm'] = cvd_norm
df['s_dom'] = s_dom
df['ofs'] = s_delta + cvd_norm + s_dom

print(f'HV Z: {hv_z.min():.2f} to {hv_z.max():.2f}')
print(f'OFS: {df["ofs"].min():.2f} to {df["ofs"].max():.2f}')
print(f'Z_FRAMA: {np.nanmin(zf):.2f} to {np.nanmax(zf):.2f}')

# 4d. VWAP
vwap = np.full(n, np.nan)
vwap_u = np.full(n, np.nan)
vwap_l = np.full(n, np.nan)
cpv, cv = 0.0, 0.0
for i in range(n):
    cpv += p[i] * v[i]
    cv += v[i]
    if cv > 0:
        w = cpv / cv
        vwap[i] = w
        s = np.sqrt(max(np.mean((p[max(0, i-20):i+1] - w)**2), 1e-10))
        vwap_u[i] = w + VWAP_BAND * s
        vwap_l[i] = w - VWAP_BAND * s
df['vwap_overext'] = (p > vwap_u) | (p < vwap_l)

# 4e. Yield
tnx_c = tnx['Close'].squeeze()
fvx_c = fvx['Close'].squeeze()
if isinstance(tnx_c, pd.Series):
    tnx_c.index = pd.to_datetime(tnx_c.index, utc=True)
    fvx_c.index = pd.to_datetime(fvx_c.index, utc=True)
    tnx_d = tnx_c.reindex(df.index, method='ffill')
    fvx_d = fvx_c.reindex(df.index, method='ffill')
    yz = (tnx_d - tnx_d.rolling(20).mean()) / tnx_d.rolling(20).std().clip(lower=1e-10)
    df['yield_z'] = yz.fillna(0.0).values
    df['curve'] = (tnx_d - fvx_d).values
else:
    df['yield_z'] = 0.0
    df['curve'] = 0.0
print('Indicators done.')

# 5. 5-GATE PIPELINE
signals = []
gate_blocks = {'G0_macro': 0, 'G1_hv': 0, 'G2_frama': 0, 'G3_ofs': 0, 'G4_vwap': 0}
daily = {}

for i in range(max(FRAMA_P, CVD_W, 20) + 2, n):
    bar = df.iloc[i]
    dt_idx = df.index[i]
    day = dt_idx.date()

    if dt_idx.weekday() >= 5: continue
    if bar['phase'] in ('OPEN', 'LUNCH', 'CLOSE'): continue
    if bar['is_red']:
        gate_blocks['G0_macro'] += 1
        continue

    if day not in daily:
        daily[day] = {'signals': 0, 'blocks': {k: 0 for k in gate_blocks}}

    # G1: HV Z-Score
    hv = bar['hv_z']
    if hv < -1.0 or hv > 2.0:
        gate_blocks['G1_hv'] += 1; daily[day]['blocks']['G1_hv'] += 1; continue

    # G2: FRAMA
    z = bar['z_frama']
    if np.isnan(z) or z > 0.5:
        gate_blocks['G2_frama'] += 1; daily[day]['blocks']['G2_frama'] += 1; continue

    # [3-GATE MODE] G3 OFS + G4 VWAP dilewati — hanya Macro + HV + FRAMA

    # 3/3 -> SIGNAL
    daily[day]['signals'] += 1
    is_power_hour = bar['phase'] == 'POWER_HOUR'
    signals.append({
        'time': dt_idx, 'phase': bar['phase'],
        'dir': 'buy' if bar['s_delta'] > 0 else 'sell',
        'entry': bar['Close'], 'ofs': 0.0, 'hv': hv, 'z_frama': z,
        'yield_z': bar['yield_z'], 'is_power_hour': is_power_hour,
    })

dsig = pd.DataFrame(signals)
print(f'\nTotal signals (3/3): {len(dsig)}')
if len(dsig) > 0:
    print(f'Buy: {(dsig["dir"]=="buy").sum()}, Sell: {(dsig["dir"]=="sell").sum()}')
else:
    print('No signals generated.')

print('\nGate blocks:')
for g, c in sorted(gate_blocks.items(), key=lambda x: -x[1]):
    print(f'  {g}: {c}')

# 6. EXIT SIMULATION
def simulate(sig, df):
    i = df.index.get_indexer([sig['time']], method='bfill')[0]
    if i < 0: return None
    entry = sig['entry']; d = sig['dir']
    is_ph = sig['is_power_hour']
    # Sniper: RR 1:1.5, Scalp: RR 1:0.5
    if is_ph:
        sl_pct, tp_pct = 0.003, 0.005  # scalping
    else:
        sl_pct, tp_pct = 0.005, 0.012  # sniper
    sl = entry * (1 - sl_pct) if d == 'buy' else entry * (1 + sl_pct)
    tp = entry * (1 + tp_pct) if d == 'buy' else entry * (1 - tp_pct)
    for j in range(1, min(20, len(df) - i)):
        b = df.iloc[i + j]
        if d == 'buy':
            if b['High'] >= tp: return {'exit': tp, 'pl': tp - entry, 'win': True}
            if b['Low'] <= sl: return {'exit': sl, 'pl': sl - entry, 'win': False}
        else:
            if b['Low'] <= tp: return {'exit': tp, 'pl': entry - tp, 'win': True}
            if b['High'] >= sl: return {'exit': sl, 'pl': entry - sl, 'win': False}
    return None

ex = []
for _, s in dsig.iterrows():
    r = simulate(s, df)
    ex.append(r if r else {'exit': None, 'pl': None, 'win': None})
dsig['exit'] = [e['exit'] for e in ex]
dsig['pl'] = [e['pl'] for e in ex]
dsig['win'] = [e['win'] for e in ex]

comp = dsig[dsig['win'].notna()]
completed_trades = len(comp)
if completed_trades > 0:
    wins = comp[comp['win'] == True]
    losses = comp[comp['win'] == False]
    wr = len(wins) / completed_trades * 100
    gp = wins['pl'].sum() if len(wins) > 0 else 0
    gl = abs(losses['pl'].sum()) if len(losses) > 0 else 1
    pf = gp / gl if gl > 0 else float('inf')
    total_pl = comp['pl'].sum()
    avg_w = wins['pl'].mean() if len(wins) > 0 else 0
    avg_l = losses['pl'].mean() if len(losses) > 0 else 0
else:
    wins = 0; losses = 0; wr = 0; pf = 0; total_pl = 0; avg_w = 0; avg_l = 0

print(f'\nCompleted trades: {completed_trades}')
print(f'Wins: {len(wins) if isinstance(wins, pd.DataFrame) else wins}, '
      f'Losses: {len(losses) if isinstance(losses, pd.DataFrame) else losses}')
print(f'Win Rate: {wr:.1f}%')
print(f'Profit Factor: {pf:.2f}')
print(f'Total P&L: {total_pl:.2f} pts')
if completed_trades > 0:
    print(f'Avg Win: {avg_w:.2f}, Avg Loss: {avg_l:.2f}')

# 7. DAILY
dstats = []
for day, info in sorted(daily.items()):
    dc = dsig[dsig['time'].dt.date == day] if len(dsig) > 0 else pd.DataFrame()
    dw = dc[dc['win'] == True] if len(dc) > 0 else pd.DataFrame()
    dstats.append({
        'date': day, 'signals': info['signals'],
        'completed': len(dc) if len(dc) > 0 else 0,
        'wins': len(dw) if len(dw) > 0 else 0,
        'pl': dc['pl'].sum() if len(dc) > 0 else 0,
    })
dd = pd.DataFrame(dstats)

print(f'\n--- Daily ---')
print(f'Days with data: {len(dd)}')
print(f'Avg signals/day: {dd["signals"].mean():.1f}')
print(f'Max signals/day: {dd["signals"].max()}')
print(f'Days >=1 signal: {(dd["signals"]>=1).sum()}')

# 8. SCORECARD vs TARGET
wins_a = comp['win'].values if completed_trades > 0 else []
mcw, mcl, cw, cl = 0, 0, 0, 0
for w in wins_a:
    if w: cw += 1; cl = 0; mcw = max(mcw, cw)
    else: cl += 1; cw = 0; mcl = max(mcl, cl)

print('\n' + '=' * 55)
print(' SCORECARD - US100 M15 Backtest (60-day)')
print('=' * 55)
checks = [
    ('Win Rate >= 70%', wr >= 70, f'{wr:.1f}%'),
    ('PF >= 2.0', pf >= 2.0, f'{pf:.2f}'),
    ('Max CW >= 6', mcw >= 6, str(mcw)),
    ('Max CL <= 4', mcl <= 4, str(mcl)),
    ('Total trades >= 10', completed_trades >= 10, str(completed_trades)),
]
passed = 0
for name, ok, val in checks:
    m = 'OK' if ok else 'XX'
    print(f'  [{m}] {name}: {val}')
    if ok: passed += 1
print(f'\n Passed: {passed}/{len(checks)}')
print('=' * 55)

# 9. SUMMARY
print('\n' + '=' * 55)
print(f' BACKTEST SUMMARY - US100')
print(f' Period: {START_STR} to {END_STR}')
print(f' Data: QQQ M15 proxy')
print(f' Pipeline: 3-Gate (Macro + HV + FRAMA)')
print('=' * 55)
print(f' Total bars: {n}')
print(f' Trading days: {df["date"].nunique()}')
print(f' Signals (5/5): {len(dsig)}')
print(f' Completed trades: {completed_trades}')
print(f' Win Rate: {wr:.1f}%')
print(f' Profit Factor: {pf:.2f}')
print(f' Total P&L: {total_pl:.2f}')
print(f' Avg signals/day: {dd["signals"].mean():.1f}' if len(dd) > 0 else '')
print(f' Max CW: {mcw}, Max CL: {mcl}')
print(f' Gate most blocked: {max(gate_blocks, key=gate_blocks.get)} ({max(gate_blocks.values())})')
print('=' * 55)
