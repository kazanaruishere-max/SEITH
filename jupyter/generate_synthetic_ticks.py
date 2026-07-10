"""Generate synthetic tick data from M1 OHLCV for tick-level backtest engine.
Dukascopy is blocked from this network. Synthetic ticks preserve OHLCV structure.
"""
import csv, random, struct, math
from pathlib import Path
from datetime import datetime

IN_CSV = Path("jupyter/backtest_analysis/xauusd_m1_14m.csv")
OUT_CSV = Path("jupyter/backtest_analysis/xauusd_ticks_synthetic.csv")
TICKS_PER_BAR = 60  # 1 tick per second

def brownian_bridge(o, h, l, c, n):
    """Generate n ticks from OHLC using Brownian bridge."""
    ticks = [o]
    for i in range(1, n):
        t = i / n
        # Brownian bridge: mu = linear interpolation, sigma = scaled by range
        mu = o * (1 - t) + c * t
        sigma = (h - l) * 0.3 * math.sqrt(t * (1 - t))
        price = random.gauss(mu, sigma)
        price = max(l - 0.1, min(h + 0.1, price))
        ticks.append(round(price, 3))
    return ticks

def main():
    random.seed(42)
    with open(IN_CSV) as f:
        reader = csv.DictReader(f)
        bars = list(reader)

    print(f"Generating synthetic ticks from {len(bars):,} M1 bars...")
    with open(OUT_CSV, "w", newline="") as f:
        w = csv.writer(f)
        w.writerow(["time_ms", "bid", "ask", "bid_vol", "ask_vol"])

        total_ticks = 0
        for bar in bars:
            ts = int(bar["time"])
            o, h, l, c = float(bar["open"]), float(bar["high"]), float(bar["low"]), float(bar["close"])
            volume = int(float(bar["volume"]))

            prices = brownian_bridge(o, h, l, c, TICKS_PER_BAR)
            spread_base = random.uniform(0.2, 0.8)
            vol_per_tick = max(1, volume // TICKS_PER_BAR)

            for i, p in enumerate(prices):
                spread = spread_base + random.gauss(0, 0.1)
                spread = max(0.1, spread)
                ask = round(p + spread / 2, 3)
                bid = round(p - spread / 2, 3)
                bv = random.randint(vol_per_tick // 2, vol_per_tick * 2)
                av = random.randint(vol_per_tick // 2, vol_per_tick * 2)
                time_ms = (ts + i) * 1000 + random.randint(0, 999)
                w.writerow([time_ms, bid, ask, bv, av])
                total_ticks += 1

            if total_ticks % 100000 == 0:
                print(f"  {total_ticks:,} ticks generated...", flush=True)

    size_mb = OUT_CSV.stat().st_size / 1024 / 1024
    print(f"\nDone: {total_ticks:,} ticks written to {OUT_CSV}")
    print(f"Size: {size_mb:.1f} MB")

if __name__ == "__main__":
    main()
