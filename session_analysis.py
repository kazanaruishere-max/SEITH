"""Session analysis for AI SEITH best trading hours."""
import csv
from datetime import datetime, timezone

with open('jupyter/backtest_analysis/xauusd_m1_14m.csv') as f:
    rows = list(csv.DictReader(f))

m15 = []
for i in range(0, len(rows), 15):
    chunk = rows[i:i+15]
    if len(chunk) < 15: continue
    m15.append({'time': int(chunk[-1]['time']), 'close': float(chunk[-1]['close'])})

hv = {}
for i in range(20, len(m15)):
    prices = [m15[j]['close'] for j in range(i-20, i)]
    returns = [(prices[k]/prices[k-1]-1) for k in range(1, len(prices))]
    mean = sum(returns)/len(returns)
    var = sum((r-mean)**2 for r in returns)/len(returns)
    std = var**0.5 if var > 0 else 1e-10
    z = (returns[-1] if returns else 0) / std
    hv[i] = z

sl, tp = 1.5, 1.5

print("REVERSAL BACKTEST by SESSION (SL=$1.50 TP=$1.50 HV>0.5)")
print("=" * 70)

sessions = [
    ("Asia Night",   [0,1,2,3],   "00:00-04:00 UTC"),
    ("Asia Prime",   [4,5,6,7],   "04:00-08:00 UTC"),
    ("London Open",  [8,9,10,11], "08:00-12:00 UTC"),
    ("London/NY",    [12,13,14,15], "12:00-16:00 UTC"),
    ("NY Session",   [16,17,18,19],"16:00-20:00 UTC"),
    ("NY Late",      [20,21,22,23],"20:00-00:00 UTC"),
]

for name, hours, label in sessions:
    hits = total = 0
    for i in range(20, len(m15)-1):
        if i not in hv or hv[i] <= 0.5: continue
        dt = datetime.fromtimestamp(m15[i]['time'], tz=timezone.utc)
        if dt.hour not in hours: continue
        curr = m15[i]; entry = curr['close']
        curr_up = curr['close'] > m15[i-1]['close']
        direction = 'SELL' if curr_up else 'BUY'
        result = 'PENDING'
        for j in range(i*15, min((i+8)*15, len(rows))):
            bh = float(rows[j]['high']); bl = float(rows[j]['low'])
            if direction == 'BUY':
                if bl <= entry - sl: result = 'LOSS'; break
                if bh >= entry + tp: result = 'WIN'; break
            else:
                if bh >= entry + sl: result = 'LOSS'; break
                if bl <= entry - tp: result = 'WIN'; break
        total += 1
        if result == 'WIN': hits += 1
    wr = hits/total*100 if total else 0
    freq = total / 100
    print(f"  {name:<15s} {label:<20s} {total:4d} trades  WR={wr:5.1f}%  {freq:.1f}/day")

print()
print("BEST HOURS (HV>0.5, sorted by WR)")
print("=" * 70)

hour_data = []
for hour in range(24):
    hits = total = 0
    for i in range(20, len(m15)-1):
        if i not in hv or hv[i] <= 0.5: continue
        dt = datetime.fromtimestamp(m15[i]['time'], tz=timezone.utc)
        if dt.hour != hour: continue
        curr = m15[i]; entry = curr['close']
        curr_up = curr['close'] > m15[i-1]['close']
        direction = 'SELL' if curr_up else 'BUY'
        result = 'PENDING'
        for j in range(i*15, min((i+8)*15, len(rows))):
            bh = float(rows[j]['high']); bl = float(rows[j]['low'])
            if direction == 'BUY':
                if bl <= entry - sl: result = 'LOSS'; break
                if bh >= entry + tp: result = 'WIN'; break
            else:
                if bh >= entry + sl: result = 'LOSS'; break
                if bl <= entry - tp: result = 'WIN'; break
        total += 1
        if result == 'WIN': hits += 1
    if total >= 15:
        hour_data.append((hits/total*100, hour, total))

hour_data.sort(reverse=True)
for wr, hour, total in hour_data[:10]:
    print(f"  Hour {hour:02d} UTC: {total:3d} trades, WR={wr:.1f}%")

print()
print("RECOMMENDED CONFIG")
print("=" * 70)
best_sessions = []
for name, hours, label in sessions:
    hits = total = 0
    for i in range(20, len(m15)-1):
        if i not in hv or hv[i] <= 0.5: continue
        dt = datetime.fromtimestamp(m15[i]['time'], tz=timezone.utc)
        if dt.hour not in hours: continue
        curr = m15[i]; entry = curr['close']
        curr_up = curr['close'] > m15[i-1]['close']
        direction = 'SELL' if curr_up else 'BUY'
        result = 'PENDING'
        for j in range(i*15, min((i+8)*15, len(rows))):
            bh = float(rows[j]['high']); bl = float(rows[j]['low'])
            if direction == 'BUY':
                if bl <= entry - sl: result = 'LOSS'; break
                if bh >= entry + tp: result = 'WIN'; break
            else:
                if bh >= entry + sl: result = 'LOSS'; break
                if bl <= entry - tp: result = 'WIN'; break
        total += 1
        if result == 'WIN': hits += 1
    if total >= 20:
        best_sessions.append((hits/total*100, name, hours, total))

best_sessions.sort(reverse=True)
best_all = []
for wr, name, hours, total in best_sessions:
    print(f"  {name:<15s}: WR={wr:.1f}% ({total} trades)")
    best_all.extend(hours)

# Final combined
print()
print("COMBINED: Best sessions only")
hits = total = 0
for i in range(20, len(m15)-1):
    if i not in hv or hv[i] <= 0.5: continue
    dt = datetime.fromtimestamp(m15[i]['time'], tz=timezone.utc)
    if dt.hour not in best_all: continue
    curr = m15[i]; entry = curr['close']
    curr_up = curr['close'] > m15[i-1]['close']
    direction = 'SELL' if curr_up else 'BUY'
    result = 'PENDING'
    for j in range(i*15, min((i+8)*15, len(rows))):
        bh = float(rows[j]['high']); bl = float(rows[j]['low'])
        if direction == 'BUY':
            if bl <= entry - sl: result = 'LOSS'; break
            if bh >= entry + tp: result = 'WIN'; break
        else:
            if bh >= entry + sl: result = 'LOSS'; break
            if bl <= entry - tp: result = 'WIN'; break
    total += 1
    if result == 'WIN': hits += 1
wr = hits/total*100 if total else 0
freq = total / 100
print(f"  Hours: {sorted(set(best_all))}")
print(f"  Trades: {total} ({freq:.1f}/day)")
print(f"  WR: {wr:.1f}%")
print(f"  Target PF 4.0 needs WR >= 73%")
print(f"  Status: {'ACHIEVED' if wr >= 73 else 'NEAR MISS'}")
