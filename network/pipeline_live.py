#!/usr/bin/env python3
"""AI SEITH US100 — Single-pass pipeline validation on OANDA live data."""

import sys, json, numpy as np
sys.path.insert(0, 'python/python')
from seith_bridge.mt5 import init_mt5, get_rates_json, get_tick, shutdown
import MetaTrader5 as mt5

SYMBOL = 'US100'
HV_W = 20; FRAMA_P = 14; CVD_W = 20; OFS_TH = 2.0

def compute(rates):
    p = np.array([x['close'] for x in rates], dtype=float)
    h = np.array([x['high'] for x in rates], dtype=float)
    l = np.array([x['low'] for x in rates], dtype=float)
    v = np.array([x['volume'] for x in rates], dtype=float)
    n = len(p)

    hv_z = np.zeros(n)
    ret = np.diff(p) / (p[:-1] + 1e-10)
    for i in range(HV_W, len(ret)):
        seg = ret[i-HV_W:i]; s = seg.std()
        hv_z[i] = (seg[-1]-seg.mean())/s if s>1e-10 else 0

    zf = np.full(n, np.nan)
    for i in range(FRAMA_P, n):
        w = p[i-FRAMA_P:i+1]; hlf = FRAMA_P//2
        r1 = max(w[:hlf+1].max()-w[:hlf+1].min(), .001)
        r2 = max(w[hlf:].max()-w[hlf:].min(), .001)
        r3 = max(w.max()-w.min(), .001)
        D = np.clip((np.log(r1+r2)-np.log(r3))/np.log(2)+1, 1, 2)
        alpha = np.exp(-4.6*(D-1))
        prev = p[i-1]
        f = alpha*p[i] + (1-alpha)*prev
        s10 = p[max(0,i-9):i+1].std()
        zf[i] = (p[i]-f)/s10 if s10>1e-10 else 0

    ratio = np.where(h-l>1e-10, (p-l)/(h-l), 0.5)
    s_delta = 2*(ratio-0.5)
    rd = s_delta * v
    cvd = np.cumsum(rd)
    cvn = np.zeros(n)
    for i in range(CVD_W, n):
        seg = cvd[i-CVD_W:i+1]; s = seg.std()
        z = (seg[-1]-seg.mean())/s if s>1e-10 else 0
        cvn[i] = np.tanh(z/3)

    sdm = np.zeros(n)
    for i in range(20, n):
        pos = (p[i]-p[i-20])/(p[i-20:i].max()-p[i-20:i].min()+1e-10)
        sdm[i] = np.clip((0.5-pos)*2, -1, 1)

    ofs = s_delta + cvn + sdm
    return {'p':p,'h':h,'l':l,'v':v,'hv_z':hv_z,'zf':zf,'sd':s_delta,'ofs':ofs,'times':[x['time'] for x in rates]}

print('='*55)
print('  AI SEITH US100 — Pipeline Validation (OANDA Live)')
print('='*55)

if not init_mt5():
    print('FAIL: MT5'); exit(1)
print('MT5 OK')

# Load data (try M15 first, fallback M1)
for tf_name, tf in [('M15', mt5.TIMEFRAME_M15), ('M1', mt5.TIMEFRAME_M1)]:
    rj = get_rates_json(SYMBOL, 500, tf)
    if rj: break
rates = json.loads(rj)
print(f'Loaded {len(rates)} {tf_name} candles for {SYMBOL}')

tick = get_tick(SYMBOL)
if tick:
    sp = tick['ask']-tick['bid']
    print(f'Current: bid={tick["bid"]:.1f} ask={tick["ask"]:.1f} spread={sp:.1f}')

ind = compute(rates)
n = len(ind['p'])
print(f'\n=== LIVE PIPELINE RESULTS ===')

# Run pipeline on all bars
total_signals = 0
signals = []
gate_counts = {'G0':0,'G1_hv':0,'G2_frama':0,'G3_ofs':0,'G4_vwap':0}
passed = {'G0':0,'G1':0,'G2':0,'G3':0,'G4':0}

for i in range(60, n):
    # G1 HV
    hv = ind['hv_z'][i]
    if hv < -1.0 or hv > 2.0:
        gate_counts['G1_hv'] += 1; continue
    passed['G1'] += 1

    # G2 FRAMA
    zf_val = ind['zf'][i]
    if np.isnan(zf_val) or zf_val > 0.5:
        gate_counts['G2_frama'] += 1; continue
    passed['G2'] += 1

    # G3 OFS
    ofs = ind['ofs'][i]
    if abs(ofs) <= 1.0 or abs(ofs) < OFS_TH:
        gate_counts['G3_ofs'] += 1; continue
    passed['G3'] += 1

    # G4 VWAP (session SMA)
    p20 = ind['p'][max(0,i-20):i+1]
    vwap = p20.mean()
    band = p20.std() * 2.5
    c = ind['p'][i]
    if c > vwap+band or c < vwap-band:
        gate_counts['G4_vwap'] += 1; continue
    passed['G4'] += 1

    total_signals += 1
    sig = {
        'time': ind['times'][i],
        'dir': 'buy' if ind['sd'][i] > 0 else 'sell',
        'price': c, 'ofs': ofs, 'hv': hv, 'zf': zf_val,
    }
    signals.append(sig)

# Print summary
print(f'Total bars scanned: {n}')
print(f'Gate pass rates:')
print(f'  G1 HV pass: {passed["G1"]}/{n-60} = {passed["G1"]/(n-60)*100:.1f}%')
print(f'  G2 FRAMA pass: {passed["G2"]}/{passed["G1"]} = {passed["G2"]/passed["G1"]*100:.1f}%' if passed['G1']>0 else '')
print(f'  G3 OFS pass: {passed["G3"]}/{passed["G2"]} = {passed["G3"]/passed["G2"]*100:.1f}%' if passed['G2']>0 else '')
print(f'  G4 VWAP pass: {passed["G4"]}/{passed["G3"]} = {passed["G4"]/passed["G3"]*100:.1f}%' if passed['G3']>0 else '')
print(f'\nTotal 5/5 signals: {total_signals}')
print(f'Signal density: {total_signals/(n-60)*100:.1f}% of bars')
print(f'\nFirst 5 signals:')
for s in signals[:5]:
    print(f'  {s["time"]} {s["dir"]} @ {s["price"]:.1f} OFS={s["ofs"]:.2f} HV={s["hv"]:.2f}')

print(f'\nGate block distribution:')
for g, c in sorted(gate_counts.items(), key=lambda x:-x[1]):
    print(f'  {g}: {c} blocks ({c/(n-60)*100:.1f}%)')

print(f'\n=== VERDICT ===')
if total_signals >= 10:
    print(f'  ✅ Pipeline running: {total_signals} signals from {n} bars')
    print(f'  OFS threshold {OFS_TH} optimal — ~{passed["G3"]/passed["G2"]*100:.0f}% of bars pass G3')
elif total_signals >= 3:
    print(f'  ⚠️  Pipeline running: {total_signals} signals, but low count')
    print(f'  Consider lowering OFS_TH to 1.5-1.8')
else:
    print(f'  ❌ Pipeline barely producing signals')
    print(f'  OFS distribution needs recalibration')

shutdown()
