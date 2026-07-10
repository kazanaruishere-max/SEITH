"""AI SEITH Signal Chart Generator — Candlestick chart with entry/SL/TP."""

import os
import pandas as pd
import matplotlib
matplotlib.use("Agg")  # non-interactive backend
import matplotlib.pyplot as plt
import matplotlib.dates as mdates

# Dark theme matching Jupyter seith_dark style
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
    "grid.alpha": 0.3,
    "font.size": 9,
})

GOLD = "#ffd700"
GREEN = "#00c853"
RED = "#ff1744"
CYAN = "#00bcd4"


def generate_chart(
    prices: list,          # list of (time_seconds, bid, ask)
    entry_price: float,
    sl_price: float,
    tp1_price: float,
    tp2_price: float = None,
    direction: str = "BUY",
    output_path: str = None,
) -> str:
    """Generate candlestick chart with entry/SL/TP lines.

    Returns path to saved PNG image. Saves to output_path or auto-generates.
    """
    if output_path is None:
        output_path = os.path.join(
            os.environ.get("TEMP", "/tmp"),
            f"seith_signal_{abs(hash(str(prices[-5:]))) % 100000}.png",
        )

    if len(prices) < 10:
        return _generate_fallback_chart(entry_price, sl_price, tp1_price, direction, output_path)

    # Build OHLCV from tick data (aggregate into 1-min candles)
    df = _build_candles(prices, 60)  # 60-second candles

    if df is None or df.empty:
        return _generate_fallback_chart(entry_price, sl_price, tp1_price, direction, output_path)

    # Take last N candles
    n_candles = min(60, len(df))
    df = df.iloc[-n_candles:]

    fig, ax = plt.subplots(figsize=(10, 6))

    # ── Candlesticks ──
    width = 0.6 * (df.index[1] - df.index[0]).total_seconds() / 86400 if len(df) > 1 else 0.0001
    up = df[df["close"] >= df["open"]]
    down = df[df["close"] < df["open"]]

    for subset, color in [(up, GREEN), (down, RED)]:
        if subset.empty:
            continue
        ax.bar(
            subset.index, subset["close"] - subset["open"], width, bottom=subset["open"],
            color=color, edgecolor=color, linewidth=0.5, alpha=0.8,
        )
        ax.vlines(
            subset.index, subset["low"], subset["high"],
            color=color, linewidth=0.5, alpha=0.6,
        )

    # ── Entry Zone (shaded area) ──
    zone_width = abs(entry_price * 0.0003)  # ~0.03% of price
    zone_top = entry_price + zone_width
    zone_bottom = entry_price - zone_width
    ax.axhspan(zone_bottom, zone_top, color=CYAN, alpha=0.12, zorder=2)
    ax.axhline(entry_price, color=CYAN, linewidth=1.5, linestyle="-", alpha=0.7, label=f"Entry {entry_price:.3f}")
    ax.plot([df.index[-1], df.index[-1] + pd.Timedelta(seconds=30)],
            [entry_price, entry_price], color=CYAN, linewidth=1.5, alpha=0.7)

    # ── Stop Loss ──
    ax.axhline(sl_price, color=RED, linewidth=1.2, linestyle="--", alpha=0.6, label=f"SL {sl_price:.3f}")

    # ── Take Profit ──
    tp_color = GOLD
    ax.axhline(tp1_price, color=tp_color, linewidth=1.2, linestyle="--", alpha=0.6, label=f"TP1 {tp1_price:.3f}")
    if tp2_price:
        ax.axhline(tp2_price, color=GREEN, linewidth=1, linestyle=":", alpha=0.4, label=f"TP2 {tp2_price:.3f}")

    # ── Styling ──
    ax.set_title(f"{direction} SIGNAL — XAUUSD.sml", color=GOLD, fontsize=13, fontweight="bold", pad=10)
    ax.set_xlabel("Time (UTC)", color=WHITE)
    ax.set_ylabel("Price", color=WHITE)
    ax.legend(loc="best", fontsize=8, facecolor="#161b22", edgecolor="#30363d", labelcolor=WHITE)
    ax.xaxis.set_major_formatter(mdates.DateFormatter("%H:%M"))
    ax.tick_params(colors=WHITE)
    ax.grid(True, alpha=0.15)
    for spine in ax.spines.values():
        spine.set_color("#30363d")

    plt.tight_layout()
    plt.savefig(output_path, dpi=150, bbox_inches="tight", facecolor="#0d1117")
    plt.close(fig)
    return output_path


def _build_candles(prices: list, seconds: int = 60):
    """Aggregate tick data into OHLCV candles."""
    if not prices:
        return None
    df = pd.DataFrame(prices, columns=["time", "bid", "ask"])
    df["time"] = pd.to_datetime(df["time"], unit="s")
    df["price"] = (df["bid"] + df["ask"]) / 2
    df.set_index("time", inplace=True)
    ohlc = df["price"].resample(f"{seconds}S").ohlc()
    ohlc.columns = ["open", "high", "low", "close"]
    ohlc.dropna(inplace=True)
    return ohlc


def _generate_fallback_chart(entry, sl, tp, direction, output_path):
    """Generate minimal chart when price data is limited."""
    fig, ax = plt.subplots(figsize=(10, 6))
    ax.set_facecolor("#161b22")
    ax.text(0.5, 0.5, f"{direction} SIGNAL\nEntry: {entry:.3f}\nSL: {sl:.3f}\nTP: {tp:.3f}",
            ha="center", va="center", fontsize=16, color=GOLD,
            transform=ax.transAxes, fontweight="bold")
    ax.set_title("XAUUSD.sml Signal (No Chart Data)", color=GOLD)
    ax.axis("off")
    plt.savefig(output_path, dpi=150, bbox_inches="tight", facecolor="#0d1117")
    plt.close(fig)
    return output_path


# Fix missing WHITE reference
WHITE = "#c9d1d9"
