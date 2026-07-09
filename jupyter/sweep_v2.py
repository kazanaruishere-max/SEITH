"""Parameter sweep with strict IS/OOS split.
Sweeps on TRAIN (in-sample), validates best config on TEST (out-of-sample).
"""
import subprocess, json, csv, os
from pathlib import Path

BASE_CMD = ["cargo", "run", "-p", "xauusd", "--bin", "seith-backtest", "--"]
REPORT_DIR = Path("jupyter/backtest_analysis")
os.chdir(Path(__file__).parent.parent)

def run_segment(tag: str, args: list = None, segment: str = "train") -> dict:
    cmd = BASE_CMD + [f"--segment={segment}"] + (args or [])
    result = subprocess.run(cmd, capture_output=True, text=True, timeout=180)
    rpath = REPORT_DIR / f"backtest_report_{segment}.json"
    if rpath.exists():
        with open(rpath) as f:
            return json.load(f)
    print(f"  ERROR [{tag}]: no report")
    return None

def extract(r: dict) -> list:
    if not r:
        return ["-"] * 8
    return [r.get("total_trades", 0), f"{r.get('win_rate', 0):.1f}",
            f"{r.get('net_pips', 0):.1f}", f"{r.get('profit_factor', 0):.2f}",
            f"{r.get('max_drawdown_pct', 0):.1f}", r.get("max_consecutive_losses", 0),
            f"{r.get('sortino_ratio', 0):.2f}", f"{r.get('recovery_factor', 0):.2f}"]

# ── IS Sweep ──
print("=" * 70)
print("  IN-SAMPLE SWEEP (TRAINING)")
print("=" * 70)
results = []

# Baseline
r = run_segment("baseline")
results.append(("Baseline", extract(r)))

# Tier thresholds
for t1, t2 in [(70, 55), (80, 65), (65, 50)]:
    r = run_segment(f"T1={t1}", [f"--tier1={t1}", f"--tier2={t2}"])
    results.append((f"T1={t1}", extract(r)))

# OFS
for ofs in [1, 3]:
    r = run_segment(f"OFS={ofs}", [f"--ofs={ofs}"])
    results.append((f"OFS={ofs}", extract(r)))

# GVZ
for gvz in [0.5, 1.5]:
    r = run_segment(f"GVZ={gvz}", [f"--gvz={gvz}"])
    results.append((f"GVZ={gvz}", extract(r)))

header = ["Config", "Trades", "WR%", "Net$", "PF", "DD%", "MaxCL", "Sortino", "RecFact"]
print(f"\n  {header[0]:>10s}  {'Trades':>6s}  {'WR%':>5s}  {'Net$':>6s}  {'PF':>5s}  {'DD%':>6s}  {'CL':>4s}  {'Sort':>6s}  {'RF':>6s}")
print("  " + "-" * 66)
for name, vals in results:
    vals_str = [str(v) for v in vals]
    print(f"  {name:>10s}  {vals_str[0]:>6s}  {vals_str[1]:>5s}  {vals_str[2]:>6s}  {vals_str[3]:>5s}  {vals_str[4]:>6s}  {vals_str[5]:>4s}  {vals_str[6]:>6s}  {vals_str[7]:>6s}")

# Find best config by PF
best_idx = max(range(len(results)), key=lambda i: float(results[i][1][3]) if results[i][1][3] != "-" else -1)
best_name = results[best_idx][0]
print(f"\n  Best IS config: {best_name} (PF={results[best_idx][1][3]})")

# ── OOS Validation ──
print()
print("=" * 70)
print(f"  OUT-OF-SAMPLE VALIDATION: {best_name}")
print("=" * 70)
r_oos = run_segment("oos-best", segment="test")
if r_oos:
    v = extract(r_oos)
    print(f"  Trades: {v[0]}  WR: {v[1]}%  Net: {v[2]}  PF: {v[3]}  DD: {v[4]}%  CL: {v[5]}  Sortino: {v[6]}")
    print()
    # Compare
    print("  IS vs OOS:")
    print(f"    WR:   {results[best_idx][1][1]}%  ->  {v[1]}%")
    print(f"    PF:   {results[best_idx][1][3]}   ->  {v[3]}")
    print(f"    DD:   {results[best_idx][1][4]}%  ->  {v[4]}%")

# Save
csv_path = REPORT_DIR / "sweep_is_oos.csv"
with open(csv_path, "w", newline="") as f:
    w = csv.writer(f)
    w.writerow(header)
    for name, vals in results:
        w.writerow([name] + [str(v) for v in vals])
print(f"\n  Saved: {csv_path}")
