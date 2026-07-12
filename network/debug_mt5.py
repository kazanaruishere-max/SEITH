#!/usr/bin/env python3
"""Debug MT5 data access."""

import sys, json, time
sys.path.insert(0, 'python/python')
from seith_bridge.mt5 import init_mt5, get_rates_json, get_tick, get_dom
import MetaTrader5 as mt5

init_mt5()

print('=== Testing symbol access ===')
symbols_to_try = ['US100.cash', 'US100', 'US100Cash', 'USA100', 'US100.IDX']
for sym in symbols_to_try:
    info = mt5.symbol_info(sym)
    if info:
        print(f'{sym}: name={info.name}, trade={info.trade_mode}, time={info.time}')
    else:
        print(f'{sym}: not found')

print('\n=== Available symbols (first 20) ===')
all_syms = mt5.symbols_get()
if all_syms:
    for s in all_syms[:20]:
        if '100' in s.name or 'NAS' in s.name or 'USA' in s.name:
            print(f'  {s.name}')
else:
    print('None available')

print('\n=== Try rates ===')
for sym in ['US100.cash', 'US100']:
    r = get_rates_json(sym, 50, mt5.TIMEFRAME_M1)
    if r:
        d = json.loads(r)
        print(f'{sym} M1: {len(d)} candles, last_close={d[-1]["close"]}')
        break
    else:
        print(f'{sym}: no rates')
