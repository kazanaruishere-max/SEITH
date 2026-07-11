"""AI SEITH Signal Chart Generator — OHLCV candlestick chart with entry/SL/TP.
Receives pre-built OHLCV data (no resampling needed).
Uses mplfinance for professional candlestick rendering.
"""

import os
import pandas as pd
import matplotlib
matplotlib.use("Agg")
import matplotlib.pyplot as plt
import mplfinance as mpf

DARK_BG = "#0d1117"
CARD_BG = "#161b22"
GOLD = "#ffd700"
GREEN = "#00c853"
RED = "#ff1744"
CYAN = "#00bcd4"
WHITE = "#c9d1d9"

mpf_style = mpf.make_mpf_style(
    base_mpf_style="charles",
    marketcolors=mpf.make_marketcolors(
        up=GREEN, down=RED,
        edge={"up": GREEN, "down": RED},
        wick={"up": "#00e676", "down": "#ff5252"},
        volume={"up": GREEN, "down": RED},
    ),
    facecolor=DARK_BG,
    figcolor=DARK_BG,
    edgecolor="#30363d",
    gridcolor="#21262d",
    gridstyle="--",
    gridaxis="both",
    y_on_right=False,
)


def generate_chart(
    prices: list,         # each entry: [timestamp_seconds, open, high, low, close]
    entry_price: float,
    sl_price: float,
    tp1_price: float,
    tp2_price: float = None,
    direction: str = "BUY",
    output_path: str = None,
) -> str:
    """Generate candlestick chart from OHLCV data with entry/SL/TP lines."""
    if output_path is None:
        output_path = os.path.join(
            os.environ.get("TEMP", "/tmp"),
            f"seith_signal_{abs(hash(str(prices[-5:]))) % 100000}.png",
        )

    if len(prices) < 3:
        return _fallback_chart(entry_price, sl_price, tp1_price, direction, output_path)

    # Build DataFrame directly from OHLCV data
    df = pd.DataFrame(prices, columns=["time", "open", "high", "low", "close"])
    df["time"] = pd.to_datetime(df["time"], unit="s")
    df.set_index("time", inplace=True)
    df["volume"] = 100  # fake volume for mplfinance compat

    # Last 20 candles max
    n = min(20, len(df))
    df = df.iloc[-n:]

    # Generate chart with mplfinance
    fig, axes = mpf.plot(
        df,
        type="candle",
        style=mpf_style,
        figsize=(12, 7),
        returnfig=True,
        title=f"{direction} SIGNAL \u2014 XAUUSD.sml",
        ylabel="Price",
        xrotation=0,
        datetime_format="%H:%M",
        tight_layout=True,
    )

    ax = axes[0]

    # Entry zone
    zone_w = abs(entry_price * 0.0003)
    ax.axhspan(entry_price - zone_w, entry_price + zone_w, color=CYAN, alpha=0.15, zorder=2)
    ax.axhline(entry_price, color=CYAN, linewidth=2, linestyle="-", alpha=0.9, label=f"Entry {entry_price:.3f}")

    # Stop Loss
    ax.axhline(sl_price, color=RED, linewidth=1.5, linestyle="--", alpha=0.8, label=f"SL {sl_price:.3f}")

    # Take Profit
    ax.axhline(tp1_price, color=GOLD, linewidth=1.5, linestyle="--", alpha=0.8, label=f"TP1 {tp1_price:.3f}")
    if tp2_price:
        ax.axhline(tp2_price, color=GREEN, linewidth=1.2, linestyle=":", alpha=0.6, label=f"TP2 {tp2_price:.3f}")

    ax.legend(loc="best", fontsize=9, facecolor=CARD_BG, edgecolor="#30363d", labelcolor=WHITE)
    ax.tick_params(colors=WHITE, labelsize=10)

    fig.savefig(output_path, dpi=200, bbox_inches="tight", facecolor=DARK_BG)
    plt.close(fig)
    return output_path


def _fallback_chart(entry, sl, tp, direction, output_path):
    """Generate minimal text chart when price data is limited."""
    fig, ax = plt.subplots(figsize=(10, 6))
    ax.set_facecolor(CARD_BG)
    ax.text(0.5, 0.5, f"{direction} SIGNAL\nEntry: {entry:.3f}\nSL: {sl:.3f}\nTP: {tp:.3f}",
            ha="center", va="center", fontsize=16, color=GOLD,
            transform=ax.transAxes, fontweight="bold")
    ax.set_title("XAUUSD.sml Signal (No Chart Data)", color=GOLD)
    ax.axis("off")
    plt.savefig(output_path, dpi=200, bbox_inches="tight", facecolor=DARK_BG)
    plt.close(fig)
    return output_path
