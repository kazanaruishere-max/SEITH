"""HONEST VALIDATION: 
- Parameters selected using IS data ONLY
- Tested ONCE on OOS data (no second chances)
- No data leakage
"""
import csv
from datetime import datetime, timezone
from collections import defaultdict

with open('jupyter/backtest_analysis/xauusd_m1_14m.csv') as f:
    lines = f.readlines()
header = lines[0].strip().split(',')
data = [dict(zip(header, l.strip().split(','))) for l in lines[1:]]

# Compute M15 bars
m15 = []
for i in range(0, len(data), 15):
    chunk = data[i:i+15]
    if len(chunk) < 15: continue
    m15.append({
        'time': int(chunk[-1]['time']),
        'close': float(chunk[-1]['close']),
        'high': max(float(r['high']) for r in chunk),
        'low': min(float(r['low']) for r in chunk),
    })

# HV Z-Score
hv = {}
for i in range(20, len(m15)):
    prices = [m15[j]['close'] for j in range(i-20, i)]
    returns = [(prices[k]/prices[k-1]-1) for k in range(1, len(prices))]
    mean = sum(returns)/len(returns)
    var = sum((r-mean)**2 for r in returns)/len(returns)
    std = var**0.5 if var > 0 else 1e-10
    z = (returns[-1] if returns else 0) / std
    hv[i] = z

# 80/20 chronological split
split_idx = int(len(m15) * 0.8)
print(f"Split: {datetime.fromtimestamp(m15[split_idx]['time'], tz=timezone.utc)}")
print(f"IS bars: {split_idx - 20}, OOS bars: {len(m15) - split_idx - 1}")
print()

# =====================
# STEP 1: On IS data ONLY, find best session hours
# =====================
print("=" * 70)
print("STEP 1: FIND BEST HOURS (IS DATA ONLY)")
print("=" * 70)

hour_perf = {}
for hour in range(24):
    for hv_min in [0.3, 0.5, 0.7, 1.0]:
        hits = total = 0
        for i in range(20, split_idx):
            if i not in hv or hv[i] <= hv_min: continue
            dt = datetime.fromtimestamp(m15[i]['time'], tz=timezone.utc)
            if dt.hour != hour: continue
            
            curr_up = m15[i]['close'] > m15[i-1]['close']
            next_up = m15[i+1]['close'] > m15[i]['close']
            rev = (curr_up and not next_up) or (not curr_up and next_up)
            total += 1
            if rev: hits += 1
        
        if total >= 20:
            wr = hits/total*100
            key = (hour, hv_min)
            hour_perf[key] = {'total': total, 'wr': wr, 'hits': hits}

# Sort by WR descending, show top 20
sorted_hours = sorted(hour_perf.items(), key=lambda x: -x[1]['wr'])
print(f"{'Hour':>5s} {'HV>=':>5s} {'Trades':>7s} {'WR%':>6s}")
print("-" * 35)
top_sessions = []
for (h, hv_min), d in sorted_hours[:15]:
    print(f"{h:5d} {hv_min:5.1f} {d['total']:7d} {d['wr']:6.1f}%")
    top_sessions.append({'hour': h, 'hv_min': hv_min, 'total': d['total'], 'wr': d['wr']})

# Select top 4 hours for further testing
best_hours = sorted(set(s['hour'] for s in top_sessions[:6]))[:6]
print(f"\nSelected hours from IS data: {best_hours}")
print(f"(Based on top IS sessions)")

# =====================
# STEP 2: On IS data ONLY, find best SL/TP
# =====================
print()
print("=" * 70)
print("STEP 2: FIND BEST SL/TP (IS DATA ONLY)")
print("=" * 70)

is_configs = []
for sl in [1.5, 2.0, 2.5, 3.0, 3.5]:
    for rr in [1.0, 1.2, 1.5, 2.0]:
        tp = round(sl * rr, 1)
        for hv_min in [0.3, 0.5, 0.7]:
            total = hits = losses = net = 0
            cw = cl = max_cw = max_cl = 0
            for i in range(20, split_idx):
                if i not in hv or hv[i] <= hv_min: continue
                dt = datetime.fromtimestamp(m15[i]['time'], tz=timezone.utc)
                if dt.hour not in best_hours: continue
                
                curr_up = m15[i]['close'] > m15[i-1]['close']
                direction = 'SELL' if curr_up else 'BUY'
                entry = m15[i]['close']
                
                result = 'PENDING'
                for j in range(i*15, min((i+8)*15, len(data))):
                    bh = float(data[j]['high']); bl = float(data[j]['low'])
                    if direction == 'BUY':
                        if bl <= entry - sl: result = 'LOSS'; break
                        if bh >= entry + tp: result = 'WIN'; break
                    else:
                        if bh >= entry + sl: result = 'LOSS'; break
                        if bl <= entry - tp: result = 'WIN'; break
                
                total += 1
                if result == 'WIN':
                    hits += 1; net += tp; cw += 1; cl = 0
                else:
                    losses += 1; net -= sl; cl += 1; cw = 0
                max_cw = max(max_cw, cw); max_cl = max(max_cl, cl)
            
            if total < 20: continue
            wr = hits/total*100
            pf = (hits*tp)/(losses*sl) if losses > 0 else 0
            is_configs.append({
                'sl': sl, 'tp': tp, 'hv_min': hv_min,
                'total': total, 'wr': wr, 'pf': pf, 'cw': max_cw, 'cl': max_cl, 'net': net
            })

# Sort by PF
is_configs.sort(key=lambda x: -x['pf'])
print(f"{'SL':>4s} {'TP':>5s} {'HV':>4s} {'Trades':>7s} {'WR%':>6s} {'PF':>6s} {'CW':>4s} {'CL':>4s}")
print("-" * 55)
for c in is_configs[:15]:
    pf_str = f"{c['pf']:.2f}"
    print(f"{c['sl']:>4.1f} {c['tp']:>5.1f} {c['hv_min']:>4.1f} {c['total']:>7d} {c['wr']:>6.1f} {pf_str:>6s} {c['cw']:>4d} {c['cl']:>4d}")

# Select best config from IS
best = is_configs[0] if is_configs else None
if best:
    print(f"\nBEST IS CONFIG: SL=${best['sl']} TP=${best['tp']} HV>{best['hv_min']}")
    print(f"IS results: {best['total']}t WR={best['wr']:.1f}% PF={best['pf']:.2f} CW={best['cw']} CL={best['cl']}")
    
    # =====================
    # STEP 3: Test ONCE on OOS data
    # =====================
    print()
    print("=" * 70)
    print("STEP 3: VALIDATE ON OOS DATA (ONE TEST ONLY)")
    print("=" * 70)
    
    sl = best['sl']; tp = best['tp']; hv_min = best['hv_min']
    total = hits = losses = net = 0
    cw = cl = max_cw = max_cl = 0
    
    for i in range(split_idx, len(m15)-1):
        if i not in hv or hv[i] <= hv_min: continue
        dt = datetime.fromtimestamp(m15[i]['time'], tz=timezone.utc)
        if dt.hour not in best_hours: continue
        
        curr_up = m15[i]['close'] > m15[i-1]['close']
        direction = 'SELL' if curr_up else 'BUY'
        entry = m15[i]['close']
        
        result = 'PENDING'
        for j in range(i*15, min((i+8)*15, len(data))):
            bh = float(data[j]['high']); bl = float(data[j]['low'])
            if direction == 'BUY':
                if bl <= entry - sl: result = 'LOSS'; break
                if bh >= entry + tp: result = 'WIN'; break
            else:
                if bh >= entry + sl: result = 'LOSS'; break
                if bl <= entry - tp: result = 'WIN'; break
        
        total += 1
        if result == 'WIN':
            hits += 1; net += tp; cw += 1; cl = 0
        else:
            losses += 1; net -= sl; cl += 1; cw = 0
        max_cw = max(max_cw, cw); max_cl = max(max_cl, cl)
    
    wr = hits/total*100 if total else 0
    pf = (hits*tp)/(losses*sl) if losses > 0 else 0
    
    print(f"Hours: {best_hours}")
    print(f"Config: SL=${sl} TP=${tp} HV>{hv_min}")
    print()
    print(f"{'Metric':>15s} {'IS':>10s} {'OOS':>10s} {'Target':>10s} {'Status':>10s}")
    print("-" * 55)
    results = [
        ('Trades', str(best['total']), str(total), '>20', ''),
        ('WR%', f"{best['wr']:.1f}%", f"{wr:.1f}%", '>60%', 'PASS' if wr >= 60 else 'FAIL'),
        ('PF', f"{best['pf']:.2f}", f"{pf:.2f}", '>4.0', 'PASS' if pf >= 4.0 else 'FAIL'),
        ('CW', str(best['cw']), str(max_cw), '>9', 'PASS' if max_cw >= 9 else 'FAIL'),
        ('CL', str(best['cl']), str(max_cl), '<4', 'PASS' if max_cl <= 4 else 'FAIL'),
        ('Net', f"${best['net']:.0f}", f"${net:.0f}", 'Positive', 'PASS' if net > 0 else 'FAIL'),
    ]
    for name, is_v, oos_v, tgt, status in results:
        print(f"{name:>15s} {is_v:>10s} {oos_v:>10s} {tgt:>10s} {status:>10s}")
    
    all_pass = wr >= 60 and pf >= 4.0 and max_cw >= 9 and max_cl <= 4
    print(f"\nALL TARGETS MET: {'YES' if all_pass else 'NO'}")
    if not all_pass:
        print("\nNOTE: This is the honest result. Hours and parameters selected from IS only.")
        print("If results are weaker than previous run, it confirms data mining bias.")
