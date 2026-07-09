"""Update Jupyter notebook cells to read real backtest CSV instead of demo data."""
import nbformat as nbf
from pathlib import Path

nb_path = Path("C:/Users/Lenovo/PROJECT/AI SEITH/jupyter/notebooks/xauusd_overview.ipynb")
nb = nbf.read(nb_path, as_version=4)

# Find cells to replace
for i, cell in enumerate(nb.cells):
    # Cell 2: Replace generate_demo_trades with real data loader
    if "generate_demo_trades" in cell.source:
        nb.cells[i] = nbf.v4.new_code_cell("""\
BACKTEST_CSV = Path.cwd() / "backtest_analysis" / "trades_backtest.csv"
BACKTEST_REPORT = Path.cwd() / "backtest_analysis" / "backtest_report.json"

if BACKTEST_CSV.exists():
    trades = pd.read_csv(BACKTEST_CSV)
    trades["time"] = pd.to_datetime(trades["time"])
    trades["result_bool"] = trades["result"] == "WIN"
    print(f"Loaded {len(trades)} backtest trades from CSV")
    if BACKTEST_REPORT.exists():
        import json
        with open(BACKTEST_REPORT) as f:
            r = json.load(f)
        print(f"Report: WR={r['win_rate']:.1f}% PF={r['profit_factor']:.2f} DD={r['max_drawdown_pct']:.1f}%")
else:
    print("No backtest data found. Run: cargo run -p xauusd --bin seith-backtest")
    # Fallback to demo
    trades = generate_demo_trades()

trades.tail(3)
""")

    # Cell 3: Update equity curve section header
    if "equity" in cell.source and "fig.suptitle" in cell.source and "Performance" in cell.source:
        if "cumulative_pips" in cell.source:
            nb.cells[i] = nbf.v4.new_code_cell("""\
total = len(trades)
wins = trades["result_bool"].sum() if "result_bool" in trades.columns else 0
losses = total - wins
wr = wins / total * 100 if total > 0 else 0
avg_win = trades.loc[trades["result_bool"], "pips"].mean() if wins > 0 else 0
avg_loss = trades.loc[~trades["result_bool"], "pips"].abs().mean() if losses > 0 else 0
pf = abs(avg_win * wins / (avg_loss * losses)) if losses > 0 else float("inf")
net_pips = trades["pips"].sum() if "pips" in trades.columns else 0
max_dd_pct = trades["drawdown"].max() if "drawdown" in trades.columns else 0
max_cl = trades.loc[~trades["result_bool"], "pips"].value_counts().shape[0] if losses > 0 else 0

# Equity curve
if "cumulative_pips" not in trades.columns:
    trades["cumulative_pips"] = trades["pips"].cumsum()
    trades["equity"] = 10000 + trades["cumulative_pips"] * 0.1
    trades["drawdown_pct"] = (trades["equity"].cummax() - trades["equity"]) / trades["equity"].cummax() * 100
    trades["drawdown"] = trades["drawdown_pct"]

# Cards
fig, axes = plt.subplots(1, 6, figsize=(16, 3.5))
fig.suptitle("Performance Metrics", fontsize=18, fontweight="bold", color=GOLD, y=1.05)

metrics = [
    ("Win Rate", f"{wr:.1f}%", GREEN, f"Target ≥ 70%"),
    ("Profit Factor", f"{pf:.2f}", PURPLE, f"Target ≥ 2.0"),
    ("Max DD", f"{max_dd_pct:.1f}%", RED if max_dd_pct > 8 else ORANGE, "Limit ≤ 8%"),
    ("Net Pips", f"{net_pips:+.0f}", GREEN if net_pips > 0 else RED, f"{total} trades"),
    ("Recovery Factor", f"{0.0:.2f}", CYAN, "Target ≥ 4.0"),
    ("Consec Loss", f"{max_cl}", ORANGE if max_cl >= 3 else GREEN, f"Max Win: {0}"),
]
for ax, (label, val, c, extra) in zip(axes.flat, metrics):
    seith_metric_card(ax, label, val, c, extra=extra)
plt.tight_layout()
plt.show()
""")

nbf.write(nb, nb_path)
print(f"Updated: {nb_path}")
""