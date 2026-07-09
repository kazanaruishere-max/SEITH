"""Parameter sweep — find optimal thresholds for AI SEITH."""
import subprocess, json, csv, sys, os
from pathlib import Path

BASE_CMD = ["cargo", "run", "-p", "xauusd", "--bin", "seith-backtest", "--"]
REPORT = Path("jupyter/backtest_analysis/backtest_report.json")
os.chdir(Path(__file__).parent.parent)

def run_backtest(tag: str, args: list = None) -> dict:
    cmd = BASE_CMD + (args or [])
    print(f"\n  [{tag}] {' '.join(cmd[3:])}")
    result = subprocess.run(cmd, capture_output=True, text=True, timeout=180)
    if REPORT.exists():
        with open(REPORT) as f:
            return json.load(f)
    print(f"  ERROR: {result.stderr.strip()[-200:]}")
    return None

def report_row(tag: str, r: dict) -> list:
    return [tag, r["total_trades"], f"{r['win_rate']:.1f}",
            f"{r['net_pips']:.1f}", f"{r['profit_factor']:.2f}",
            f"{r['max_drawdown_pct']:.1f}", f"{r['max_consecutive_losses']}",
            f"{r['avg_spread']:.2f}"]

# ── Baseline (default thresholds) ──
print("=" * 70)
print("  AI SEITH PARAMETER SWEEP")
print("=" * 70)

results = [report_row("Baseline (OFS=2)", run_backtest("Baseline"))]

# ── Sweep 1: OFS Min ──
print("\n" + "-" * 70)
print("  SWEEP 1: OFS_MIN_VALID")
print("-" * 70)
for val in [1, 3]:
    r = run_backtest(f"OFS={val}", [f"--ofs={val}"])
    if r: results.append(report_row(f"OFS={val}", r))

# ── Sweep 2: Spread Tolerance ──
print("\n" + "-" * 70)
print("  SWEEP 2: SPREAD TOLERANCE")
print("-" * 70)
for val in [2.0, 5.0]:
    r = run_backtest(f"Spread={val}", [f"--spread={val}"])
    if r: results.append(report_row(f"Sp={val}", r))

# ── Sweep 3: GVZ Z-Score threshold ──
print("\n" + "-" * 70)
print("  SWEEP 3: GVZ Z-SCORE THRESHOLD")
print("-" * 70)
for val in [0.5, 1.5]:
    r = run_backtest(f"GVZ={val}", [f"--gvz={val}"])
    if r: results.append(report_row(f"GVZ={val}", r))

# ── Sweep 4: Tier thresholds ──
print("\n" + "-" * 70)
print("  SWEEP 4: TIER THRESHOLDS")
print("-" * 70)
for t1, t2 in [(70, 55), (80, 65)]:
    r = run_backtest(f"T1={t1}T2={t2}", [f"--tier1={t1}", f"--tier2={t2}"])
    if r: results.append(report_row(f"T1={t1}", r))

# ── Output sweep report ──
print("\n" + "=" * 70)
print("  SWEEP RESULTS")
print("=" * 70)
header = ["Config", "Trades", "WR%", "NetPips", "PF", "DD%", "MaxCL", "Spread"]
print(f"  {header[0]:>14s}  {header[1]:>6s}  {header[2]:>5s}  {header[3]:>7s}  {header[4]:>5s}  {header[5]:>6s}  {header[6]:>5s}  {header[7]:>6s}")
print("  " + "-" * 68)
for row in results:
    print(f"  {row[0]:>14s}  {row[1]:>6d}  {row[2]:>5s}  {row[3]:>7s}  {row[4]:>5s}  {row[5]:>6s}  {row[6]:>5s}  {row[7]:>6s}")

# Save to CSV
csv_path = "jupyter/backtest_analysis/sweep_results.csv"
with open(csv_path, "w", newline="") as f:
    w = csv.writer(f)
    w.writerow(header)
    w.writerows(results)
print(f"\n  Saved: {csv_path}")
