#!/usr/bin/env python3
"""Debug OANDA DOM access — try different symbols + methods."""

import sys, time, json
sys.path.insert(0, 'python/python')
from seith_bridge.mt5 import init_mt5
import MetaTrader5 as mt5

init_mt5()

symbols = ['US100', 'US100.cash', 'US100Cash', 'USA100', 'XAUUSD', 'SP500']
print('=== Testing DOM access ===')
for sym in symbols:
    info = mt5.symbol_info(sym)
    if info:
        print(f'\n{sym}: trade_mode={info.trade_mode}, market_book={info.book_depth}')
        
        # Try DOM
        selected = mt5.symbol_select(sym, True)
        print(f'  symbol_select: {selected}')
        
        subscribed = mt5.market_book_add(sym)
        print(f'  market_book_add: {subscribed}')
        
        if subscribed:
            time.sleep(1.0)
            raw = mt5.market_book_get(sym)
            if raw and len(raw) > 0:
                print(f'  DOM levels: {len(raw)}')
                for b in raw[:5]:
                    print(f'    type={b.type} price={b.price:.2f} volume={b.volume}')
            else:
                print(f'  No DOM data (type={type(raw).__name__}, len={len(raw) if raw else 0})')
            
            mt5.market_book_release(sym)
    else:
        print(f'\n{sym}: not found')

print('\n=== Try direct access US100 ===')
selected = mt5.symbol_select('US100', True)
print(f'Selected: {selected}')
info = mt5.symbol_info('US100')
if info:
    print(f'  spread={info.spread}, digits={info.digits}, book_depth={info.book_depth}')

sub = mt5.market_book_add('US100')
print(f'Book added: {sub}')
if sub:
    for t in [0.5, 1.0, 2.0]:
        time.sleep(t)
        raw = mt5.market_book_get('US100')
        if raw and len(raw) > 0:
            print(f'After {t}s: {len(raw)} levels')
            for b in raw[:3]:
                print(f'  type={b.type} price={b.price:.2f} vol={b.volume}')
            break
        else:
            print(f'After {t}s: no data')
    mt5.market_book_release('US100')

mt5.shutdown()
