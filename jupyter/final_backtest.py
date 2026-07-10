"""Final validation with strict IS/OOS split for best config(s)."""
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

# 80/20 chronological split
split_idx = int(len(m15) * 0.8)
split_time = m15[split_idx]['time']

target_hours = [19, 13, 12, 20]

def run_backtest(hours, hv_min, sl, tp):
    """Returns {trades, wr, pf, cw, cl, net} for both IS and OOS."""
    results = {}
    for label, start, end in [("IS", 20, split_idx), ("OOS", split_idx, len(m15)-1)]:
        total = hits = losses = net = 0
        cw = cl = max_cw = max_cl = 0
        for i in range(start, end):
            dt = datetime.fromtimestamp(m15[i]['time'], tz=timezone.utc)
            if dt.hour not in hours: continue
            if i not in hv or hv[i] <= hv_min: continue
            
            curr_up = m15[i]['close'] > m15[i-1]['close']
            direction = 'SELL' if curr_up else 'BUY'
            entry = m15[i]['close']
            
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
            if result == 'WIN':
                hits += 1; net += tp; cw += 1; cl = 0
            else:
                losses += 1; net -= sl; cl += 1; cw = 0
            max_cw = max(max_cw, cw); max_cl = max(max_cl, cl)
        
        wr = hits/total*100 if total else 0
        pf = (hits*tp)/(losses*sl) if losses > 0 else 0
        results[label] = {'trades': total, 'wr': wr, 'pf': pf, 'cw': max_cw, 'cl': max_cl, 'net': net}
    return results

print("IS/OOS VALIDATION (80/20 chronological split)")
print("=" * 60)
print(f"Split point: {datetime.fromtimestamp(split_time, tz=timezone.utc)}")
print()

configs = [
    ("SL=$2 TP=$2.40 HV>0.5", target_hours, 0.5, 2.0, 2.4),
    ("SL=$2 TP=$2.40 HV>0.7", target_hours, 0.7, 2.0, 2.4),
    ("SL=$2 TP=$3.00 HV>0.5", target_hours, 0.5, 2.0, 3.0),
    ("SL=$3 TP=$3.60 HV>0.5", target_hours, 0.5, 3.0, 3.6),
]

for name, hours, hv_min, sl, tp in configs:
    r = run_backtest(hours, hv_min, sl, tp)
    is_r, oos_r = r['IS'], r['OOS']
    alpha_wr = is_r['wr'] - oos_r['wr']
    alpha_pf = is_r['pf'] - oos_r['pf']
    print(f"--- {name} ---")
    print(f"  IS:  {is_r['trades']:3d}t WR={is_r['wr']:5.1f}% PF={is_r['pf']:5.2f} CW={is_r['cw']:2d} CL={is_r['cl']:2d} Net=\${is_r['net']:+.0f}")
    print(f"  OOS: {oos_r['trades']:3d}t WR={oos_r['wr']:5.1f}% PF={oos_r['pf']:5.2f} CW={oos_r['cw']:2d} CL={oos_r['cl']:2d} Net=\${oos_r['net']:+.0f}")
    print(f"  Alpha decay: WR={alpha_wr:+.1f}pp PF={alpha_pf:+.2f}")
    
    all_pass = all([
        r['IS']['pf'] >= 4.0 and r['OOS']['pf'] >= 4.0,
        r['IS']['wr'] >= 60 and r['OOS']['wr'] >= 60,
        r['IS']['cw'] >= 9 and r['OOS']['cw'] >= 9,
        r['IS']['cl'] <= 4 and r['OOS']['cl'] <= 4,
    ])
    print(f"  ALL TARGETS (IS+OOS): {'PASS' if all_pass else 'FAIL'}")
    print()
