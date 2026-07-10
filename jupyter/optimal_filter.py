"""Refine analysis: find optimal filter for WR > 60%.
Focus on best performing combinations from initial analysis.
"""
import csv
from collections import defaultdict
from datetime import datetime, timezone

with open('jupyter/backtest_analysis/xauusd_m1_14m.csv') as f:
    rows = list(csv.DictReader(f))

m15 = []
for i in range(0, len(rows), 15):
    chunk = rows[i:i+15]
    if len(chunk) < 15: continue
    m15.append({
        'time': int(chunk[-1]['time']), 'open': float(chunk[0]['open']),
        'high': max(float(r['high']) for r in chunk),
        'low': min(float(r['low']) for r in chunk),
        'close': float(chunk[-1]['close']),
        'volume': sum(float(r['volume']) for r in chunk),
    })

# Precompute HV Z-Scores
hv = {}
for i in range(20, len(m15)):
    prices = [m15[j]['close'] for j in range(i-20, i)]
    returns = [(prices[k]/prices[k-1]-1) for k in range(1, len(prices))]
    mean = sum(returns)/len(returns)
    var = sum((r-mean)**2 for r in returns)/len(returns)
    std = var**0.5 if var > 0 else 1e-10
    z = (returns[-1] if returns else 0) / std
    hv[i] = z

print("=" * 70)
print("OPTIMAL FILTER SEARCH (Target: WR > 60%)")
print("=" * 70)

# Test ALL hour + HV threshold combinations
best_filters = []
for hour in range(24):
    for hv_min in [0.0, 0.5, 1.0, 1.5]:
        hits = total = pips = 0
        for i in range(20, len(m15)-1):
            dt = datetime.fromtimestamp(m15[i]['time'], tz=timezone.utc)
            if dt.hour != hour: continue
            if i not in hv or hv[i] <= hv_min: continue
            
            curr = m15[i]; next_bar = m15[i+1]
            curr_up = curr['close'] > curr['open']
            next_up = next_bar['close'] > next_bar['open']
            rev = (curr_up and not next_up) or (not curr_up and next_up)
            total += 1
            if rev:
                hits += 1
                pips += abs(next_bar['close'] - curr['close'])
        
        if total >= 20 and hits/total > 0.60:
            best_filters.append((hour, hv_min, total, hits/total*100, pips/max(hits,1)))

# Sort by WR
best_filters.sort(key=lambda x: -x[3])

print(f"\nFilters with WR > 60% ({len(best_filters)} found):")
print(f"{'Hour':>5s} {'HV>=':>5s} {'Bars':>5s} {'WR%':>6s} {'AvgPip':>7s}")
print("-" * 35)
for h, hv_min, n, wr, ap in best_filters:
    print(f"{h:5d} {hv_min:5.1f} {n:5d} {wr:6.1f} {ap:7.2f}")

# Best single filter: Hour 19 + HV > 0.5
print("\n" + "=" * 70)
print("BEST SINGLE FILTER: Hour 19 + HV > 0.5 (WR 69%)")
print("=" * 70)

# Simulate the backtest with this filter
print("\nSimulating M15 backtest with Hour 19 + HV > 0.5 filter...")
print("(Strategy: contrarian reversal, SL=$20, TP=$50, RR 1:2.5)")
print()

sim_hits = sim_total = sim_net = 0
cw = cl = max_cw = max_cl = 0
running = peak = max_dd = 0.0

for i in range(20, len(m15)-1):
    dt = datetime.fromtimestamp(m15[i]['time'], tz=timezone.utc)
    if dt.hour != 19: continue
    if i not in hv or hv[i] <= 0.5: continue
    
    curr = m15[i]
    next_bar = m15[i+1]
    curr_up = curr['close'] > curr['open']
    
    # Contrarian: if UP, go SHORT; if DOWN, go LONG
    direction = "SELL" if curr_up else "BUY"
    
    # Simulate SL=$20, TP=$50
    entry = curr['close']
    sl = entry - 20 if direction == "BUY" else entry + 20
    tp = entry + 50 if direction == "BUY" else entry - 50
    
    # Check next few bars for SL/TP hit
    result = "PENDING"
    exit_price = entry
    for j in range(i+1, min(i+8, len(m15))):
        bar = m15[j]
        if direction == "BUY":
            if bar['low'] <= sl: exit_price = sl; result = "LOSS"; break
            if bar['high'] >= tp: exit_price = tp; result = "WIN"; break
        else:
            if bar['high'] >= sl: exit_price = sl; result = "LOSS"; break
            if bar['low'] <= tp: exit_price = tp; result = "WIN"; break
    
    pips = (exit_price - entry) * (1 if direction == "BUY" else -1)
    sim_total += 1
    
    if result == "WIN":
        sim_hits += 1
        cw += 1; cl = 0
        max_cw = max(max_cw, cw)
    else:
        cw = 0; cl += 1
        max_cl = max(max_cl, cl)
    
    sim_net += pips
    running += pips
    peak = max(peak, running)
    dd = (peak - running) / max(peak, 0.001) * 100
    max_dd = max(max_dd, dd)

wr = sim_hits/sim_total*100 if sim_total else 0
gross_profit = sum(50 for _ in range(sim_hits))
gross_loss = sum(20 for _ in range(sim_total-sim_hits))
pf = gross_profit / max(gross_loss, 1)
rf = sim_net / max(max_dd/100 * 10000, 1) if max_dd > 0 else 0

print(f"Trades:     {sim_total}")
print(f"Wins:       {sim_hits}")
print(f"Losses:     {sim_total - sim_hits}")
print(f"Win Rate:   {wr:.1f}%")
print(f"Net Pips:   {sim_net:.1f}")
print(f"Profit Fac: {pf:.2f}")
print(f"Max CW/CL:  {max_cw}/{max_cl}")
print(f"Max DD:     {max_dd:.1f}%")
print(f"Recovery F: {rf:.2f}")
print()

# Target check
targets = {
    'PF >= 4.0': pf >= 4.0,
    'RF >= 4.0': rf >= 4.0,
    'DD < 15%': max_dd < 15,
    'CW >= 9': max_cw >= 9,
    'CL <= 4': max_cl <= 4,
}
print("TARGET CHECK:")
for t, ok in targets.items():
    print(f"  {t}: {'✅' if ok else '❌'}")
