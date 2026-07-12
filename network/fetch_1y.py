#!/usr/bin/env python3
"""Test MT5 data range access for 1 year of US100 M15."""

import sys; sys.path.insert(0, 'python/python')
from seith_bridge.mt5 import init_mt5
import MetaTrader5 as mt5
from datetime import datetime, timezone
import numpy as np

init_mt5()

start = datetime(2025, 7, 14, tzinfo=timezone.utc)
end = datetime(2026, 7, 10, tzinfo=timezone.utc)
print(f'Requesting: {start.date()} to {end.date()}', flush=True)

rates = mt5.copy_rates_range('US100', mt5.TIMEFRAME_M15, start, end)
if rates is None:
    print('Range failed, trying from_pos 10000...', flush=True)
    rates = mt5.copy_rates_from_pos('US100', mt5.TIMEFRAME_M15, 0, 10000)

if rates is not None:
    dates = [datetime.fromtimestamp(r['time']) for r in rates]
    p = np.array([r['close'] for r in rates], dtype=float)
    print(f'Got {len(rates)} bars', flush=True)
    print(f'Period: {dates[0].strftime("%Y-%m-%d")} to {dates[-1].strftime("%Y-%m-%d")}', flush=True)
    print(f'Trading days: {len(set(d.date() for d in dates))}', flush=True)
    print(f'Price range: {p.min():.1f} - {p.max():.1f}', flush=True)
    
    # Save to file for backtest
    import json
    data = [{'time': int(r['time']), 'open': float(r['open']), 'high': float(r['high']),
             'low': float(r['low']), 'close': float(r['close']), 'volume': int(r['tick_volume'])} for r in rates]
    with open('network/us100_m15_1y.json', 'w') as f:
        json.dump(data, f)
    print(f'Saved to network/us100_m15_1y.json', flush=True)
else:
    print('No data at all', flush=True)

mt5.shutdown()
