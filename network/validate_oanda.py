#!/usr/bin/env python3
"""OANDA validation: MT5 connection + OFS from bid/ask + DOM analysis."""

import sys, json, numpy as np
sys.path.insert(0, 'python/python')
from seith_bridge.mt5 import init_mt5, get_tick, get_dom, get_rates_json

print('=== STEP 1: MT5 Connect ===')
if not init_mt5():
    print('FAIL: MT5 init')
    exit(1)
print('OK: MT5 connected')

print('\n=== STEP 2: Fetch US100.cash ===')
rates = get_rates_json('US100.cash', count=100, timeframe=15)
if not rates:
    rates = get_rates_json('US100', count=100, timeframe=15)
    sym = 'US100'
else:
    sym = 'US100.cash'

if not rates:
    print('FAIL: no rates for US100')
    exit(1)

r = json.loads(rates)
print(f'OK: {len(r)} M1 candles for {sym}')
print(f'  Latest: time={r[-1]["time"]} close={r[-1]["close"]} vol={r[-1]["volume"]}')

# Build price array
prices = np.array([x['close'] for x in r])
volumes = np.array([x['volume'] for x in r], dtype=float)
print(f'  Price range: {prices.min():.2f} - {prices.max():.2f}')
print(f'  Vol range: {volumes.min():.0f} - {volumes.max():.0f}')

print('\n=== STEP 3: Tick + DOM ===')
tick = get_tick(sym)
if tick:
    spread = tick['ask'] - tick['bid']
    print(f'Tick: bid={tick["bid"]:.2f} ask={tick["ask"]:.2f} spread={spread:.2f}')
else:
    print('No tick')
    spread = 0.0

dom = get_dom(sym)
if dom:
    bids = dom.get('bids', [])
    asks = dom.get('asks', [])
    bid_vol = sum(b['volume'] for b in bids[:3]) if bids else 0
    ask_vol = sum(a['volume'] for a in asks[:3]) if asks else 0
    dom_imb = (bid_vol - ask_vol) / (bid_vol + ask_vol + 1)
    print(f'DOM: {len(bids)} levels bid, {len(asks)} levels ask')
    print(f'  Top3 bid vol: {bid_vol:.0f}, ask vol: {ask_vol:.0f}')
    print(f'  DOM imbalance: {dom_imb:.3f}')
else:
    print('No DOM')
    dom_imb = 0.0

print('\n=== STEP 4: OFS Validation ===')
# S_Delta from OHLCV
n = len(prices)
s_delta = np.zeros(n)
for i in range(1, n):
    hl = r[i]['high'] - r[i]['low']
    if hl > 1e-8:
        ratio = (r[i]['close'] - r[i]['low']) / hl
    else:
        ratio = 0.5
    s_delta[i] = 2.0 * (ratio - 0.5)

# Volume-weighted CVD
raw_delta = s_delta * volumes
cvd = np.cumsum(raw_delta)
cvd_norm = np.zeros(n)
for i in range(10, n):
    seg = cvd[i-10:i+1]
    s = seg.std()
    z = (seg[-1] - seg.mean()) / s if s > 1e-10 else 0
    cvd_norm[i] = np.tanh(z / 3.0)

# S_DOM from price pos
s_dom = np.zeros(n)
for i in range(10, n):
    pos = (prices[i] - prices[i-10]) / (prices[i-10:i].max() - prices[i-10:i].min() + 1e-10)
    s_dom[i] = np.clip((0.5 - pos) * 2, -1, 1)

ofs = s_delta + cvd_norm + s_dom

# Live OFS adjustment from tick spread + DOM
spread_factor = 1.0
if spread > 0:
    spread_factor = 1.0 + min(spread * 10, 0.5)

dom_factor = 1.0
if dom is not None:
    dom_factor = 1.0 + abs(dom_imb) * 0.5

ofs_live = ofs * min(spread_factor, 1.5) * min(dom_factor, 1.5)

print(f'  Spread adjustment factor: {spread_factor:.3f}')
print(f'  DOM imbalance factor: {dom_factor:.3f}')
print(f'  Live OFS range: {ofs_live.min():.2f} to {ofs_live.max():.2f}')
percentiles = [50, 70, 80, 90, 95, 99]
print('  Percentiles:')
for p in percentiles:
    print(f'    P{p}: {np.percentile(abs(ofs_live), p):.2f}')

print('\n=== COMPARISON: Proxy vs Live ===')
proxy_range = max(abs(ofs_live.min()), abs(ofs_live.max()))
print(f'  OANDA|OFS| P95: {np.percentile(abs(ofs_live), 95):.2f}')
print(f'  OANDA|OFS| P99: {np.percentile(abs(ofs_live), 99):.2f}')
print(f'  OANDA OFS range: {ofs_live.min():.2f} to {ofs_live.max():.2f}')

if np.percentile(abs(ofs_live), 95) > 2.5:
    print('\n  === VERDICT ===')
    print('  ✅ OFS OANDA distribusi lebih lebar dari proxy.')
    print('  OFS threshold 2.0-2.5 mungkin realistis -> WR 55% achievable.')
elif np.percentile(abs(ofs_live), 95) > 1.8:
    print('\n  === VERDICT ===')
    print('  ⚠️  OFS OANDA distribusi mirip proxy.')
    print('  Threshold optimal ~1.5-1.8. WR 40-50% realistic.')
else:
    print('\n  === VERDICT ===')
    print('  ❌ OFS OANDA distribusi lebih sempit dari proxy.')
    print('  Orderflow tidak cukup informatif. Target 55% WR tidak realistis.')
