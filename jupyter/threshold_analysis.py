"""Threshold sensitivity analysis — OFS, direction & tier optimization."""
import json, csv
from collections import defaultdict

trades = []
with open("jupyter/backtest_analysis/trades_backtest.csv") as f:
    for row in csv.DictReader(f):
        row["pips"] = float(row["pips"])
        trades.append(row)

total = len(trades)
print(f"Total: {total} trades | Net: {sum(t['pips'] for t in trades):+.1f}")
print()

# ── Tier & Direction ──
print("=" * 60)
print("  TIER x DIRECTION BREAKDOWN")
print("=" * 60)
for tier in ["Tier1Institutional", "Tier2Tactical"]:
    for d in ["BUY", "SELL"]:
        tt = [t for t in trades if t["tier"] == tier and t["direction"] == d]
        if not tt: continue
        w = [t for t in tt if t["result"] == "WIN"]
        net = sum(t["pips"] for t in tt)
        print(f"  {tier:22s} {d:5s}: {len(tt):4d} trades  WR={len(w)/len(tt)*100:5.1f}%  net={net:+.1f}")

# ── Optimal OFS proxy from volume imbalance ──
# Backtest uses compute_ofs which is 1/-1/0 based on volume imbalance
# We can see what OFS values (s_delta x3) actually occur
print()
print("=" * 60)
print("  SIMULATED OFS VALUE PERFORMANCE")
print("=" * 60)
# The backtest uses s,s,s for ofs calc, so OFS = 3*s
# s=1 => OFS=3 (strong bullish), s=-1 => OFS=-3 (strong bearish), s=0 => OFS=0 (blocked)
# But we have no OFS in the CSV, so let's check if they'd be blocked or not
for label, filter_trades in [("ALL FILTERED", trades)]:
    w = [t for t in filter_trades if t["result"] == "WIN"]
    l = [t for t in filter_trades if t["result"] == "LOSS"]
    print(f"  Current (OFS≥2 pass): {len(filter_trades)} trades, WR={len(w)/len(filter_trades)*100:.1f}%")
    break

# ── Simulate stricter OFS ──
print()
print("=" * 60)
print("  OFS THRESHOLD SWEEP (SIMULATED)")
print("=" * 60)
# Our OFS uses volume imbalance: bullish/bearish volume ratio
# OFS∈{-3,0,3}. OFS≥2 = pass (always passes since min non-zero is 3)
# OFS≥4 doesn't apply. So with current impl, all trades would pass OFS.
# This tells us OFS filter isn't blocking anything currently.
print("  Note: current OFS impl uses s_delta=s_cvd=s_dom")
print("  Result: OFS is always ∈{-3,0,3}, so |OFS|≥2 always passes")
print("  → OFS filter is NOT blocking any trades currently")
print("  → Need real S_Delta/S_CVD/S_DOM data for meaningful tuning")

# ── Practical: update spread tolerance from normalized data ──
print()
print("=" * 60)
print("  ACTIONABLE RECOMMENDATIONS")
print("=" * 60)
avg_win = sum(t["pips"] for t in trades if t["result"] == "WIN") / max(sum(1 for t in trades if t["result"] == "WIN"), 1)
avg_loss = abs(sum(t["pips"] for t in trades if t["result"] == "LOSS")) / max(sum(1 for t in trades if t["result"] == "LOSS"), 1)
rr = avg_win / max(avg_loss, 0.001)

print(f"  Avg Win: {avg_win:.3f} | Avg Loss: {avg_loss:.3f} | RR: {rr:.2f}")
print(f"  Current spread_tolerance_pips: 3.5")
print(f"  Suggestion: keep 3.5 (need real spread data)")
print(f"  Current OFS_MIN_VALID: 2")
print(f"  Suggestion: keep 2 (OFS impl needs real data)")
print(f"  Tier1 Institutional RR: {rr:.1f} (target 2.0-2.5)")
print(f"  Tier2 Tactical RR:     {rr:.1f} (target 1.0-1.2)")
print()
print(f"  KEY INSIGHT: Backtest engine functional but")
print(f"  signal source (Bayesian) is random placeholder.")
print(f"  Threshold tuning will be meaningful once")
print(f"  real OANDA sentiment + GVZ + spread are wired.")
