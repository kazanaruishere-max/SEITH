"""AI SEITH Signal Chart Generator — Candlestick chart with entry/SL/TP.
Uses mplfinance for professional candlestick rendering.
"""

import os, io
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
        up=GREEN, down=RED, edge={"up": GREEN, "down": RED},
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
    prices: list,          # list of (time_seconds, bid, ask)
    entry_price: float,
    sl_price: float,
    tp1_price: float,
    tp2_price: float = None,
    direction: str = "BUY",
    output_path: str = None,
) -> str:
    """Generate professional candlestick chart with entry/SL/TP lines."""
    if output_path is None:
        output_path = os.path.join(
            os.environ.get("TEMP", "/tmp"),
            f"seith_signal_{abs(hash(str(prices[-5:]))) % 100000}.png",
        )

    if len(prices) < 10:
        return _fallback_chart(entry_price, sl_price, tp1_price, direction, output_path)

    # Build OHLCV candles from tick data
    df = _build_candles(prices, 60)
    if df is None or df.empty:
        return _fallback_chart(entry_price, sl_price, tp1_price, direction, output_path)

    n = min(30, len(df))
    df = df.iloc[-n:]

    # ── Generate candlestick chart with mplfinance ──
    fig, axes = mpf.plot(
        df,
        type="candle",
        style=mpf_style,
        figsize=(12, 7),
        returnfig=True,
        title=f"{direction} SIGNAL — XAUUSD.sml",
        ylabel="Price",
        xrotation=0,
        datetime_format="%H:%M",
        tight_layout=True,
        savefig=dict(fname=output_path, dpi=200, bbox_inches="tight", pad_inches=0.3),
    )

    # ── Add entry/SL/TP lines to the main axis ──
    ax = axes[0]

    # Entry zone
    zone_w = abs(entry_price * 0.0003)
    ax.axhspan(entry_price - zone_w, entry_price + zone_w, color=CYAN, alpha=0.12, zorder=2)
    ax.axhline(entry_price, color=CYAN, linewidth=2, linestyle="-", alpha=0.9, label=f"Entry {entry_price:.3f}")

    # Stop Loss
    ax.axhline(sl_price, color=RED, linewidth=1.5, linestyle="--", alpha=0.8, label=f"SL {sl_price:.3f}")

    # Take Profit
    ax.axhline(tp1_price, color=GOLD, linewidth=1.5, linestyle="--", alpha=0.8, label=f"TP1 {tp1_price:.3f}")
    if tp2_price:
        ax.axhline(tp2_price, color=GREEN, linewidth=1.2, linestyle=":", alpha=0.6, label=f"TP2 {tp2_price:.3f}")

    ax.legend(loc="best", fontsize=9, facecolor=CARD_BG, edgecolor="#30363d", labelcolor=WHITE)

    # Re-save with annotations
    fig.savefig(output_path, dpi=200, bbox_inches="tight", facecolor=DARK_BG)
    plt.close(fig)
    return output_path


def _build_candles(prices: list, seconds: int = 60):
    """Aggregate tick data into OHLCV candles with proper volume."""
    if not prices:
        return None
    df = pd.DataFrame(prices, columns=["time", "bid", "ask"])
    df["time"] = pd.to_datetime(df["time"], unit="s")
    df["price"] = (df["bid"] + df["ask"]) / 2
    df.set_index("time", inplace=True)
    ohlc = df["price"].resample(f"{seconds}S").ohlc()
    ohlc.columns = ["open", "high", "low", "close"]
    ohlc.dropna(inplace=True)
    # Add fake volume for compat
    ohlc["volume"] = 100
    return ohlc


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
