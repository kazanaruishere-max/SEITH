"""AI SEITH Signal Chart Generator — Full-size OHLCV candlestick chart.
Clear bars, bold wicks, dark theme, real-time data from MT5.
"""

import os
import pandas as pd
import matplotlib
matplotlib.use("Agg")
import matplotlib.pyplot as plt
import mplfinance as mpf
from matplotlib import rcParams

# Dark theme globals
rcParams.update({
    "figure.facecolor": "#0d1117",
    "axes.facecolor": "#161b22",
    "axes.edgecolor": "#30363d",
    "axes.labelcolor": "#c9d1d9",
    "text.color": "#c9d1d9",
    "xtick.color": "#8b949e",
    "ytick.color": "#8b949e",
    "grid.color": "#21262d",
    "grid.alpha": 0.3,
    "font.size": 10,
})

WHITE = "#c9d1d9"
GOLD = "#ffd700"
GREEN = "#00c853"
RED = "#ff1744"
CYAN = "#00bcd4"

mpf_style = mpf.make_mpf_style(
    base_mpf_style="charles",
    marketcolors=mpf.make_marketcolors(
        up=GREEN, down=RED,
        edge={"up": GREEN, "down": RED},
        wick={"up": "#69f0ae", "down": "#ff5252"},
        volume={"up": GREEN, "down": RED},
    ),
    facecolor="#0d1117",
    figcolor="#0d1117",
    edgecolor="#30363d",
    gridcolor="#30363d",
    gridstyle="--",
    gridaxis="both",
    y_on_right=False,
)


def generate_chart(
    prices: list,
    entry_price: float,
    sl_price: float,
    tp1_price: float,
    tp2_price: float = None,
    direction: str = "BUY",
    output_path: str = None,
) -> str:
    if output_path is None:
        output_path = os.path.join(
            os.environ.get("TEMP", "/tmp"),
            f"seith_signal_{abs(hash(str(prices[-5:]))) % 100000}.png",
        )

    if len(prices) < 3:
        return _fallback_chart(entry_price, sl_price, tp1_price, direction, output_path)

    # Build DataFrame from OHLCV data
    df = pd.DataFrame(prices, columns=["time", "open", "high", "low", "close"])
    df["time"] = pd.to_datetime(df["time"], unit="s")
    df.set_index("time", inplace=True)
    df["volume"] = 100

    n = min(25, len(df))
    df = df.iloc[-n:]

    # Plot with mplfinance — large size, thick candles
    fig, axes = mpf.plot(
        df,
        type="candle",
        style=mpf_style,
        figsize=(14, 8),
        returnfig=True,
        title=("BUY SIGNAL" if direction == "BUY" else "SELL SIGNAL") + " — XAUUSD.sml M15",
        ylabel="Price (USD)",
        xrotation=0,
        datetime_format="%H:%M",
        tight_layout=True,
        scale_width_adjustment=dict(volume=0.8, candle=0.9),  # thicker candles
    )

    ax = axes[0]
    zone_w = max(abs(entry_price * 0.0003), 0.1)
    ax.axhspan(entry_price - zone_w, entry_price + zone_w, color=CYAN, alpha=0.12, zorder=2)
    ax.axhline(entry_price, color=CYAN, linewidth=2.5, linestyle="-", alpha=0.9, label=f"Entry {entry_price:.3f}")
    ax.axhline(sl_price, color=RED, linewidth=2.0, linestyle="--", alpha=0.8, label=f"SL {sl_price:.3f}")
    ax.axhline(tp1_price, color=GOLD, linewidth=2.0, linestyle="--", alpha=0.8, label=f"TP1 {tp1_price:.3f}")
    if tp2_price:
        ax.axhline(tp2_price, color=GREEN, linewidth=1.5, linestyle=":", alpha=0.6, label=f"TP2 {tp2_price:.3f}")

    ax.legend(loc="best", fontsize=10, facecolor="#161b22", edgecolor="#30363d", labelcolor=WHITE, framealpha=0.95)
    ax.tick_params(colors=WHITE, labelsize=11)
    ax.grid(True, alpha=0.2, linewidth=0.5)
    ax.set_ylabel("Price (USD)", fontsize=12, color=WHITE)

    fig.savefig(output_path, dpi=300, bbox_inches="tight", facecolor="#0d1117", edgecolor="none")
    plt.close(fig)
    return output_path


def _fallback_chart(entry, sl, tp, direction, output_path):
    fig, ax = plt.subplots(figsize=(14, 8))
    ax.set_facecolor("#161b22")
    ax.text(0.5, 0.5, f"{direction} SIGNAL\nEntry: {entry:.3f}\nSL: {sl:.3f}\nTP: {tp:.3f}\n\n(Not enough price data)",
            ha="center", va="center", fontsize=18, color=GOLD, transform=ax.transAxes, fontweight="bold")
    ax.set_title("XAUUSD.sml — Waiting for data", color=GOLD, fontsize=16)
    ax.axis("off")
    fig.savefig(output_path, dpi=200, bbox_inches="tight", facecolor="#0d1117")
    plt.close(fig)
    return output_path
