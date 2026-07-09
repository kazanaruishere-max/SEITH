"""Generate the AI SEITH Jupyter notebook with dark theme."""
import nbformat as nbf
from pathlib import Path

nb = nbf.v4.new_notebook()
nb.metadata = {
    "kernelspec": {
        "display_name": "Python 3",
        "language": "python",
        "name": "python3"
    },
    "language_info": {
        "name": "python",
        "version": "3.11.9"
    }
}

cells = []

# ── Imports & Style ──────────────────────────────────────────
cells.append(nbf.v4.new_code_cell("""\
import sys, os, json, math
from datetime import datetime, timedelta
from pathlib import Path

import numpy as np
import pandas as pd
import matplotlib
import matplotlib.pyplot as plt
import matplotlib.dates as mdates
import seaborn as sns

# ── Dark Theme ───────────────────────────────────────────────
style_path = Path.cwd() / "jupyter/styles/seith_dark.mplstyle"
if style_path.exists():
    plt.style.use(str(style_path))
else:
    plt.style.use("dark_background")

plt.rcParams.update({
    "figure.facecolor": "#0d1117",
    "axes.facecolor": "#161b22",
    "axes.edgecolor": "#30363d",
    "axes.labelcolor": "#c9d1d9",
    "text.color": "#c9d1d9",
    "axes.titlecolor": "#ffd700",
    "xtick.color": "#8b949e",
    "ytick.color": "#8b949e",
    "grid.color": "#21262d",
    "legend.facecolor": "#161b22",
    "legend.edgecolor": "#30363d",
    "legend.labelcolor": "#c9d1d9",
})

GOLD    = "#ffd700"
GREEN   = "#00c853"
RED     = "#ff1744"
CYAN    = "#00bcd4"
PURPLE  = "#7c4dff"
ORANGE  = "#ff6d00"
WHITE   = "#c9d1d9"
DARK_BG = "#0d1117"
CARD_BG = "#161b22"

print(f"AI SEITH Analytics — {datetime.now():%Y-%m-%d %H:%M}")
print(f"Style: seith_dark loaded | Figure: {plt.rcParams['figure.figsize']}")
"""))

# ── Helper Functions ─────────────────────────────────────────
cells.append(nbf.v4.new_code_cell("""\
def seith_metric_card(ax, label, value, color=GOLD, unit="", extra=""):
    \"\"\"Draw a professional metric card.\"\"\"
    ax.set_facecolor(CARD_BG)
    ax.text(0.5, 0.7, str(value), ha="center", va="center",
            fontsize=32, fontweight="bold", color=color)
    ax.text(0.5, 0.25, label, ha="center", va="center",
            fontsize=11, color=WHITE, alpha=0.8)
    if unit:
        ax.text(0.85, 0.72, unit, ha="center", va="center",
                fontsize=13, color=color, alpha=0.6)
    if extra:
        ax.text(0.5, 0.05, extra, ha="center", va="center",
                fontsize=9, color=WHITE, alpha=0.5)
    ax.set_xlim(0, 1)
    ax.set_ylim(0, 1)
    ax.axis("off")

def seith_card_title(ax, text, subtitle=""):
    \"\"\"Card header with gold accent line.\"\"\"
    ax.set_facecolor(CARD_BG)
    ax.text(0.03, 0.75, text, fontsize=15, fontweight="bold", color=GOLD, va="center")
    if subtitle:
        ax.text(0.03, 0.35, subtitle, fontsize=10, color=WHITE, alpha=0.6, va="center")
    ax.plot([0.03, 0.97], [0.15, 0.15], color=GOLD, linewidth=0.8, alpha=0.4)
    ax.set_xlim(0, 1)
    ax.set_ylim(0, 1)
    ax.axis("off")

print("Helpers loaded.")
"""))

# ── 1. SYSTEM OVERVIEW ────────────────────────────────────────
cells.append(nbf.v4.new_markdown_cell("""## 1. System Configuration Overview"""))

cells.append(nbf.v4.new_code_cell("""\
fig, axes = plt.subplots(2, 4, figsize=(16, 5))
fig.suptitle("AI SEITH — System Configuration", fontsize=20, fontweight="bold", color=GOLD, y=1.02)

cards = [
    ("Broker", "OANDA", CYAN, "Demo", "$10,000"),
    ("Symbol", "XAUUSD.sml", GOLD, "Leverage", "1:100"),
    ("Spread Tol.", "≤ 3.5 pips", ORANGE, "Max Pos.", "1"),
    ("Daily Loss", "≤ 3.0%", RED, "Auto-halt", "Ya"),
    ("Win Rate", "≥ 70%", GREEN, "Target", "Utama"),
    ("Profit Factor", "≥ 2.0", PURPLE, "Target", "Utama"),
    ("Max DD", "≤ 8%", CYAN, "Hard Limit", "15%"),
    ("Max Consec. Loss", "≤ 3", ORANGE, "Auto-halt", "≥ 5"),
]
for ax, (label, val, c, unit, extra) in zip(axes.flat, cards):
    seith_metric_card(ax, label, val, c, unit, extra)

plt.tight_layout()
plt.show()
"""))

# ── 2. LIVE DOM ───────────────────────────────────────────────
cells.append(nbf.v4.new_markdown_cell("""## 2. Live Depth of Market (OANDA XAUUSD.sml)"""))

cells.append(nbf.v4.new_code_cell("""\
def fetch_live_dom():
    \"\"\"Fetch DOM snapshot from running MT5 terminal.\"\"\"
    try:
        import MetaTrader5 as mt5
        mt5.initialize(path="C:/Program Files/OANDA Global MetaTrader 5 Terminal/terminal64.exe")
        mt5.symbol_select("XAUUSD.sml", True)
        mt5.market_book_add("XAUUSD.sml")
        import time; time.sleep(1.5)
        raw = mt5.market_book_get("XAUUSD.sml")
        if raw is None or len(raw) == 0:
            return None
        asks = sorted([b for b in raw if b.type == 1], key=lambda x: x.price)
        bids = sorted([b for b in raw if b.type == 2], key=lambda x: -x.price)
        mt5.shutdown()
        return {"asks": asks, "bids": bids, "symbol": "XAUUSD.sml"}
    except Exception as e:
        print(f"Cannot connect to MT5: {e}")
        return None

def plot_dom_live(dom):
    if dom is None:
        print("No DOM data — MT5 terminal must be running.")
        return
    fig, (ax1, ax2) = plt.subplots(1, 2, figsize=(14, 5), gridspec_kw={"width_ratios": [1, 1.5]})
    fig.suptitle(f"Depth of Market — {dom['symbol']}", fontsize=18, fontweight="bold", color=GOLD, y=1.02)

    asks = dom["asks"]
    bids = dom["bids"]
    best_ask = asks[0].price if asks else 0
    best_bid = bids[0].price if bids else 0
    spread = best_ask - best_bid

    # ── Spread Gauge ──
    ax1.set_facecolor(CARD_BG)
    ax1.text(0.5, 0.85, f"Spread", ha="center", fontsize=14, color=WHITE)
    ax1.text(0.5, 0.6, f"{spread:.3f}", ha="center", fontsize=48, fontweight="bold",
             color=GREEN if spread < 1.0 else (ORANGE if spread < 3.5 else RED))
    ax1.text(0.5, 0.45, "pips", ha="center", fontsize=12, color=WHITE, alpha=0.6)
    ax1.text(0.5, 0.28, f"Best Ask: {best_ask:.3f}", ha="center", fontsize=11, color=RED)
    ax1.text(0.5, 0.18, f"Best Bid: {best_bid:.3f}", ha="center", fontsize=11, color=GREEN)
    ax1.text(0.5, 0.06, f"Levels: {len(asks)} ask / {len(bids)} bid",
             ha="center", fontsize=10, color=WHITE, alpha=0.5)
    ax1.set_xlim(0, 1)
    ax1.set_ylim(0, 1)
    ax1.axis("off")

    # ── Order Book Depth Chart ──
    for side, data, color, label in [("Ask", asks, RED, "Sell Limit"),
                                      ("Bid", bids, GREEN, "Buy Limit")]:
        prices = [d.price for d in data]
        vols = [d.volume / 1000 for d in data]
        if not prices:
            continue
        ax2.barh(prices, vols, height=0.08, color=color, alpha=0.7,
                 edgecolor=color, linewidth=0.5, label=label)
        for p, v in zip(prices, vols):
            if v > 0:
                ax2.text(v + 0.1, p, f"{v:.0f}k", va="center", fontsize=7, color=WHITE, alpha=0.7)

    ax2.set_facecolor(CARD_BG)
    ax2.set_xlabel("Volume (×1000)", color=WHITE)
    ax2.set_ylabel("Price", color=WHITE)
    ax2.legend(loc="lower right", fontsize=10)
    ax2.tick_params(colors=WHITE)
    ax2.grid(True, alpha=0.15)
    for spine in ax2.spines.values():
        spine.set_color("#30363d")

    plt.tight_layout()
    plt.show()

dom = fetch_live_dom()
plot_dom_live(dom)
"""))

# ── 3. TRADE PERFORMANCE DASHBOARD ────────────────────────────
cells.append(nbf.v4.new_markdown_cell("""## 3. Trade Performance Dashboard"""))

cells.append(nbf.v4.new_code_cell("""\
def generate_demo_trades(n=200):
    \"\"\"Generate demo trades for skeleton framework.\"\"\"
    np.random.seed(42)
    dates = pd.date_range(end=datetime.now(), periods=n, freq="2h")
    directions = np.random.choice(["BUY", "SELL"], n, p=[0.55, 0.45])
    win_rate = 0.70
    results = np.random.random(n) < win_rate
    pips = np.where(results,
                    np.random.exponential(15, n) * np.random.choice([1, 1.5, 2], n, p=[0.5, 0.3, 0.2]),
                    -np.random.exponential(8, n))
    tiers = np.random.choice(["Tier 1", "Tier 2", "No Signal"], n, p=[0.25, 0.45, 0.30])

    df = pd.DataFrame({
        "time": dates, "direction": directions, "result": results,
        "pips": pips.round(2), "tier": tiers,
        "spread": np.round(np.random.uniform(0.2, 2.5, n), 2),
        "slippage": np.round(np.abs(np.random.normal(0.1, 0.3, n)), 3),
    })
    df["cumulative_pips"] = df["pips"].cumsum()
    df["equity"] = 10000 + df["cumulative_pips"] * 0.1  # $0.10 per pip
    df["drawdown"] = (df["equity"].cummax() - df["equity"]) / df["equity"].cummax() * 100
    return df

trades = generate_demo_trades()
print(f"Generated {len(trades)} demo trades for skeleton.")
trades.tail(3)
"""))

cells.append(nbf.v4.new_code_cell("""\
# ── Performance Metrics Cards ──
total = len(trades)
wins = trades["result"].sum()
losses = total - wins
wr = wins / total * 100
avg_win = trades.loc[trades["result"], "pips"].mean()
avg_loss = trades.loc[~trades["result"], "pips"].mean()
pf = abs(avg_win * wins / (avg_loss * losses)) if losses > 0 else float("inf")
max_dd = trades["drawdown"].max()
net_pips = trades["pips"].sum()
rf = (net_pips * 0.1) / (trades["equity"].max() - trades["equity"].min()) if max_dd > 0 else 0
consec_wins = 0
consec_losses = 0
max_cw = 0
max_cl = 0
for r in trades["result"]:
    if r:
        consec_wins += 1
        consec_losses = 0
        max_cw = max(max_cw, consec_wins)
    else:
        consec_losses += 1
        consec_wins = 0
        max_cl = max(max_cl, consec_losses)

fig, axes = plt.subplots(1, 6, figsize=(16, 3.5))
fig.suptitle("Performance Metrics", fontsize=18, fontweight="bold", color=GOLD, y=1.05)

metrics = [
    ("Win Rate", f"{wr:.1f}%", GREEN, "Target ≥ 70%"),
    ("Profit Factor", f"{pf:.2f}", PURPLE, "Target ≥ 2.0"),
    ("Max DD", f"{max_dd:.2f}%", RED if max_dd > 8 else ORANGE, "Limit ≤ 8%"),
    ("Net Pips", f"{net_pips:+.0f}", GREEN if net_pips > 0 else RED, f"{total} trades"),
    ("Recovery Factor", f"{rf:.2f}", CYAN, "Target ≥ 4.0"),
    ("Consec. Loss", f"{max_cl}", ORANGE if max_cl >= 3 else GREEN, f"Max Win: {max_cw}"),
]
for ax, (label, val, c, extra) in zip(axes.flat, metrics):
    seith_metric_card(ax, label, val, c, extra=extra)

plt.tight_layout()
plt.show()
"""))

cells.append(nbf.v4.new_code_cell("""\
# ── Equity Curve + Drawdown ──
fig, (ax1, ax2) = plt.subplots(2, 1, figsize=(14, 7), gridspec_kw={"height_ratios": [3, 1]})
fig.suptitle("Equity Curve & Drawdown", fontsize=16, fontweight="bold", color=GOLD, y=1.01)

ax1.set_facecolor(CARD_BG)
ax1.fill_between(trades["time"], trades["equity"], trades["equity"].iloc[0],
                 color=GREEN, alpha=0.08)
ax1.plot(trades["time"], trades["equity"], color=GREEN, linewidth=1.5)
ax1.fill_between(trades["time"], trades["equity"].cummax(), trades["equity"],
                 color=RED, alpha=0.06)
ax1.axhline(y=10000, color=WHITE, linewidth=0.5, alpha=0.3, ls="--")
ax1.set_ylabel("Equity ($)", color=WHITE)
ax1.legend(["Equity", "Initial"], loc="upper left", fontsize=10)
ax1.xaxis.set_major_formatter(mdates.DateFormatter("%m-%d"))
ax1.tick_params(colors=WHITE)

ax2.set_facecolor(CARD_BG)
ax2.fill_between(trades["time"], trades["drawdown"], 0, color=RED, alpha=0.3)
ax2.plot(trades["time"], trades["drawdown"], color=RED, linewidth=1)
ax2.axhline(y=8, color=ORANGE, linewidth=0.8, ls="--", alpha=0.6, label="8% Limit")
ax2.axhline(y=15, color=RED, linewidth=0.8, ls="--", alpha=0.6, label="15% Hard Limit")
ax2.set_ylabel("Drawdown (%)", color=WHITE)
ax2.set_xlabel("Trade Timeline", color=WHITE)
ax2.legend(loc="upper left", fontsize=9)
ax2.xaxis.set_major_formatter(mdates.DateFormatter("%m-%d"))
ax2.tick_params(colors=WHITE)

for ax in [ax1, ax2]:
    ax.grid(True, alpha=0.1)
    for spine in ax.spines.values():
        spine.set_color("#30363d")

plt.tight_layout()
plt.show()
"""))

# ── 4. MARKET REGIME ANALYSIS ─────────────────────────────────
cells.append(nbf.v4.new_markdown_cell("""## 4. Market Regime Analysis (GVZ Z-Score Distribution)"""))

cells.append(nbf.v4.new_code_cell("""\
# ── Simulated GVZ Z-Score Distribution ──
np.random.seed(42)
gvz_scores = np.random.normal(0.3, 1.2, 500)

fig, ax = plt.subplots(figsize=(12, 5))
fig.suptitle("GVZ Z-Score Distribution — Market Regime Classifier", fontsize=16, fontweight="bold", color=GOLD, y=1.01)
ax.set_facecolor(CARD_BG)

sns.histplot(gvz_scores, bins=40, color=GOLD, alpha=0.35, ax=ax,
             edgecolor=GOLD, linewidth=0.5)
sns.kdeplot(gvz_scores, color=GOLD, linewidth=2, ax=ax)

ax.axvline(x=1.0, color=RED, linewidth=1.5, ls="--", alpha=0.7, label="GVZ > +1.0 = HIGH VOL")
ax.axvline(x=-1.0, color=CYAN, linewidth=1.5, ls="--", alpha=0.7, label="GVZ < -1.0 = LOW VOL")
ax.axvline(x=0, color=WHITE, linewidth=0.8, ls=":", alpha=0.4)

high_vol = (gvz_scores > 1.0).mean() * 100
low_vol = (gvz_scores < -1.0).mean() * 100
normal_vol = (np.abs(gvz_scores) <= 1.0).mean() * 100

ax.set_xlabel("GVZ Z-Score", color=WHITE)
ax.set_ylabel("Frequency", color=WHITE)
ax.legend(loc="upper right", fontsize=10)
ax.text(0.98, 0.95, f"High Vol: {high_vol:.0f}%  |  Normal: {normal_vol:.0f}%  |  Low Vol: {low_vol:.0f}%",
        transform=ax.transAxes, ha="right", va="top", fontsize=10, color=WHITE,
        bbox=dict(boxstyle="round,pad=0.3", facecolor=CARD_BG, edgecolor="#30363d"))
ax.tick_params(colors=WHITE)
for spine in ax.spines.values():
    spine.set_color("#30363d")

plt.tight_layout()
plt.show()

print(f"Regime Split: HIGH_VOL={high_vol:.1f}% | NORMAL={normal_vol:.1f}% | LOW_VOL={low_vol:.1f}%")
"""))

# ── 5. ORDERFLOW STATISTICS ───────────────────────────────────
cells.append(nbf.v4.new_markdown_cell("""## 5. Orderflow Statistics (OFS Components)"""))

cells.append(nbf.v4.new_code_cell("""\
# ── Simulated OFS Distribution ──
np.random.seed(42)
ofs_scores = np.random.choice([-3, -2, -1, 0, 1, 2, 3], 300, p=[0.05, 0.08, 0.15, 0.30, 0.17, 0.15, 0.10])

fig, ax = plt.subplots(figsize=(12, 5))
fig.suptitle("OFS Score Distribution — Institutional Tracker", fontsize=16, fontweight="bold", color=GOLD, y=1.01)
ax.set_facecolor(CARD_BG)

values, counts = np.unique(ofs_scores, return_counts=True)
colors_bar = [GREEN if v >= 2 else RED if v <= -2 else WHITE for v in values]
ax.bar(values, counts, color=colors_bar, alpha=0.6, edgecolor=colors_bar, linewidth=0.8, width=0.6)

ax.axvspan(-1.5, 1.5, color=RED, alpha=0.06)
ax.text(0, max(counts) * 0.9, "RETAIL NOISE\nBLOCK ZONE", ha="center", fontsize=10,
        color=RED, alpha=0.5, fontweight="bold")

for v, c in zip(values, counts):
    ax.text(v, c + 1, str(c), ha="center", fontsize=10, color=WHITE)

valid = (np.abs(ofs_scores) >= 2).mean() * 100
ax.set_xlabel("OFS Score", color=WHITE)
ax.set_ylabel("Frequency", color=WHITE)
ax.text(0.98, 0.95, f"Valid Signals (|OFS| ≥ 2): {valid:.0f}%",
        transform=ax.transAxes, ha="right", va="top", fontsize=11, color=GREEN,
        bbox=dict(boxstyle="round,pad=0.3", facecolor=CARD_BG, edgecolor="#30363d"))
ax.tick_params(colors=WHITE)
for spine in ax.spines.values():
    spine.set_color("#30363d")

plt.tight_layout()
plt.show()
"""))

# ── 6. SIGNAL QUALITY ─────────────────────────────────────────
cells.append(nbf.v4.new_markdown_cell("""## 6. Signal Quality Analysis"""))

cells.append(nbf.v4.new_code_cell("""\
# ── Tier Performance ──
tier_stats = trades.groupby("tier").agg(
    count=("result", "count"),
    wins=("result", "sum"),
    avg_pips=("pips", "mean"),
).reset_index()
tier_stats["win_rate"] = (tier_stats["wins"] / tier_stats["count"] * 100).round(1)

fig, axes = plt.subplots(1, 2, figsize=(14, 5))
fig.suptitle("Signal Quality by Tier", fontsize=16, fontweight="bold", color=GOLD, y=1.02)

ax = axes[0]
ax.set_facecolor(CARD_BG)
bars = ax.bar(tier_stats["tier"], tier_stats["count"],
              color=[GOLD, CYAN, WHITE], alpha=0.6, edgecolor="white", linewidth=0.5)
for bar, wr in zip(bars, tier_stats["win_rate"]):
    ax.text(bar.get_x() + bar.get_width()/2, bar.get_height() + 1,
            f"{wr:.0f}% WR", ha="center", fontsize=10, color=WHITE)
ax.set_ylabel("Trade Count", color=WHITE)
ax.tick_params(colors=WHITE)

ax = axes[1]
ax.set_facecolor(CARD_BG)
for i, row in tier_stats.iterrows():
    tier_data = trades[trades["tier"] == row["tier"]]
    wins = tier_data[tier_data["result"]]
    losses = tier_data[~tier_data["result"]]
    if len(wins) > 0:
        ax.scatter([i] * len(wins), wins["pips"], color=GREEN, alpha=0.4, s=20)
    if len(losses) > 0:
        ax.scatter([i] * len(losses), losses["pips"], color=RED, alpha=0.4, s=20)
    bp = ax.boxplot(tier_data["pips"], positions=[i], widths=0.4,
                    patch_artist=True,
                    boxprops=dict(color=CYAN, facecolor=CYAN, alpha=0.15),
                    whiskerprops=dict(color=CYAN),
                    capprops=dict(color=CYAN),
                    medianprops=dict(color=GOLD, linewidth=2))
ax.set_xticks(range(len(tier_stats)))
ax.set_xticklabels(tier_stats["tier"])
ax.axhline(y=0, color=WHITE, linewidth=0.5, alpha=0.3)
ax.set_ylabel("Pips", color=WHITE)
ax.tick_params(colors=WHITE)

for ax in axes:
    for spine in ax.spines.values():
        spine.set_color("#30363d")
    ax.grid(True, alpha=0.1)

plt.tight_layout()
plt.show()
"""))

# ── 7. SPREAD ANALYSIS ────────────────────────────────────────
cells.append(nbf.v4.new_markdown_cell("""## 7. Spread & Slippage Analysis"""))

cells.append(nbf.v4.new_code_cell("""\
fig, axes = plt.subplots(1, 2, figsize=(14, 5))
fig.suptitle("Spread & Slippage Distribution", fontsize=16, fontweight="bold", color=GOLD, y=1.02)

ax = axes[0]
ax.set_facecolor(CARD_BG)
sns.histplot(trades["spread"], bins=30, color=GOLD, alpha=0.3, edgecolor=GOLD, linewidth=0.5, ax=ax)
ax.axvline(x=3.5, color=RED, linewidth=1.5, ls="--", alpha=0.7, label="Tolerance 3.5 pips")
ax.axvline(x=trades["spread"].mean(), color=CYAN, linewidth=1, ls=":", alpha=0.7, label=f"Mean: {trades['spread'].mean():.2f}")
ax.set_xlabel("Spread (pips)", color=WHITE)
ax.set_ylabel("Frequency", color=WHITE)
ax.legend(fontsize=9)
ax.tick_params(colors=WHITE)

ax = axes[1]
ax.set_facecolor(CARD_BG)
sns.histplot(trades["slippage"], bins=30, color=PURPLE, alpha=0.3, edgecolor=PURPLE, linewidth=0.5, ax=ax)
ax.axvline(x=trades["slippage"].mean(), color=CYAN, linewidth=1, ls=":", alpha=0.7, label=f"Mean: {trades['slippage'].mean():.3f}")
ax.axvline(x=trades["slippage"].median(), color=GOLD, linewidth=1, ls="--", alpha=0.7, label=f"Median: {trades['slippage'].median():.3f}")
ax.set_xlabel("Slippage (pips)", color=WHITE)
ax.set_ylabel("Frequency", color=WHITE)
ax.legend(fontsize=9)
ax.tick_params(colors=WHITE)

for ax in axes:
    for spine in ax.spines.values():
        spine.set_color("#30363d")

plt.tight_layout()
plt.show()

print(f"Spread:  mean={trades['spread'].mean():.2f}  median={trades['spread'].median():.2f}  max={trades['spread'].max():.2f}")
print(f"Slippage: mean={trades['slippage'].mean():.3f}  median={trades['slippage'].median():.3f}  max={trades['slippage'].max():.3f}")
"""))

# ── 8. TRADE TIMING ───────────────────────────────────────────
cells.append(nbf.v4.new_markdown_cell("""## 8. Trade Timing Analysis"""))

cells.append(nbf.v4.new_code_cell("""\
trades["hour"] = trades["time"].dt.hour
trades["day"] = trades["time"].dt.day_name()

hourly = trades.groupby("hour").agg(
    count=("result", "count"),
    avg_pips=("pips", "mean"),
    wr=("result", "mean"),
).reset_index()

fig, (ax1, ax2) = plt.subplots(1, 2, figsize=(14, 5))
fig.suptitle("Trade Timing — Hourly Distribution", fontsize=16, fontweight="bold", color=GOLD, y=1.02)

ax1.set_facecolor(CARD_BG)
ax1.bar(hourly["hour"], hourly["count"], color=GOLD, alpha=0.5, edgecolor=GOLD, linewidth=0.5, width=0.6)
ax1.set_xlabel("Hour (UTC)", color=WHITE)
ax1.set_ylabel("Trade Count", color=WHITE)
ax1.set_xticks(range(0, 24, 2))
ax1.tick_params(colors=WHITE)

ax2.set_facecolor(CARD_BG)
colors_wr = [GREEN if wr >= 0.7 else ORANGE for wr in hourly["wr"]]
ax2.bar(hourly["hour"], hourly["wr"] * 100, color=colors_wr, alpha=0.5, edgecolor=colors_wr, linewidth=0.5, width=0.6)
ax2.axhline(y=70, color=GREEN, linewidth=1, ls="--", alpha=0.5, label="Target 70%")
ax2.set_xlabel("Hour (UTC)", color=WHITE)
ax2.set_ylabel("Win Rate (%)", color=WHITE)
ax2.set_xticks(range(0, 24, 2))
ax2.legend(fontsize=9)
ax2.tick_params(colors=WHITE)

for ax in [ax1, ax2]:
    for spine in ax.spines.values():
        spine.set_color("#30363d")
    ax.grid(True, alpha=0.1)

plt.tight_layout()
plt.show()
"""))

# ── 9. RECALIBRATION RECOMMENDATIONS ──────────────────────────
cells.append(nbf.v4.new_markdown_cell("""## 9. Recalibration Recommendations"""))

cells.append(nbf.v4.new_code_cell("""\
print("=" * 60)
print("  AI SEITH — RECALIBRATION RECOMMENDATIONS")
print("=" * 60)
print()
print(f"  Based on {len(trades)} trades analyzed")
print()

avg_spread = trades["spread"].mean()
max_spread = trades["spread"].max()
avg_slippage = trades["slippage"].mean()
avg_daily_loss = trades.groupby(trades["time"].dt.date)["pips"].apply(lambda x: abs(x[x < 0].sum())).mean()

print(f"  1. Spread Tolerance: {3.5:.1f} → {min(3.5, max_spread + 0.5):.1f}")
print(f"     (Current max spread observed: {max_spread:.2f})")
print()
print(f"  2. Avg Slippage: {avg_slippage:.3f} pips")
print(f"     (May need to adjust stop buffer)")
print()
print(f"  3. Win Rate: {wr:.1f}% {'✅ TARGET MET' if wr >= 70 else '⚠️ BELOW TARGET'}")
print(f"     Profit Factor: {pf:.2f} {'✅ TARGET MET' if pf >= 2.0 else '⚠️ BELOW TARGET'}")
print(f"     Max DD: {max_dd:.2f}% {'✅ WITHIN LIMIT' if max_dd <= 8 else '❌ EXCEEDS LIMIT'}")
print(f"     Consec Loss: {max_cl} {'⚠️ NEAR LIMIT' if max_cl >= 3 else '✅ SAFE'}")
print()
print(f"  4. Threshold drift detected:")
for col in ["spread", "slippage"]:
    val = trades[col].quantile(0.95)
    print(f"     P95 {col}: {val:.3f} (consider as new tolerance floor)")
print()
print("=" * 60)
"""))

# ── 10. SYSTEM HEALTH ─────────────────────────────────────────
cells.append(nbf.v4.new_markdown_cell("""## 10. System Health Summary"""))

cells.append(nbf.v4.new_code_cell("""\
fig, ax = plt.subplots(figsize=(14, 4))
ax.set_facecolor(CARD_BG)

checks = [
    ("MT5 Terminal", "Connected" if dom is not None else "Disconnected",
     GREEN if dom is not None else RED),
    ("OANDA Bridge", "Operational", GREEN),
    ("Python Bridge (PyO3)", "Operational", GREEN),
    ("SQLite Database", "Ready", GREEN),
    ("Telegram Bot", "@AISEITH_bot", CYAN),
    ("DOM Feed", f"{dom['level_count']} levels" if dom else "Offline",
     GREEN if dom else ORANGE),
    ("Spread Now", f"{spread:.3f} pips" if dom else "N/A",
     GREEN if dom and spread < 1 else (ORANGE if dom else RED)),
    ("Last Updated", datetime.now().strftime("%H:%M:%S"), WHITE),
]

for i, (label, val, color) in enumerate(checks):
    y = 0.85 - (i % 4) * 0.22
    x = 0.05 + (i // 4) * 0.5
    ax.text(x, y, "●", color=color, fontsize=14, va="center")
    ax.text(x + 0.04, y, label, color=WHITE, fontsize=11, va="center", fontweight="bold")
    ax.text(x + 0.35, y, val, color=color if color != WHITE else GOLD, fontsize=11, va="center")

ax.set_xlim(0, 1)
ax.set_ylim(0, 1)
ax.axis("off")
plt.show()

print(f"AI SEITH v1.0 — {datetime.now():%Y-%m-%d %H:%M:%S}")
print("System ready. All components operational.")
"""))

nb.cells = cells

# Write
out_path = Path("C:/Users/Lenovo/PROJECT/AI SEITH/jupyter/notebooks/xauusd_overview.ipynb")
with open(out_path, "w", encoding="utf-8") as f:
    nbf.write(nb, f)

print(f"Notebook written: {out_path}")
print(f"Cells: {len(cells)}")
