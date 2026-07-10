"""Improved synthetic ticks with realistic market microstructure.
- Bid-ask bounce (alternating between bid and ask)
- Volume clustering (bursts of activity)
- Spread widening during volatility
- Micro-trends within M1 bar (not random walk)
"""
import csv, math, random
from pathlib import Path

IN_CSV = Path("jupyter/backtest_analysis/xauusd_m1_14m.csv")
OUT_CSV = Path("jupyter/backtest_analysis/xauusd_ticks_v2.csv")
TICKS_PER_BAR = 60

def generate_ticks(o, h, l, c, v, spread_base):
    """Generate ticks for one M1 bar with realistic microstructure."""
    ticks = []
    center = (h + l) / 2
    half_range = (h - l) / 2
    spread = spread_base * (1 + 0.5 * random.random())  # variable spread
    
    # 1. Price path: open → (random walk within range) → close
    price = o
    direction = 1 if c >= o else -1
    progress = 0
    
    for i in range(TICKS_PER_BAR):
        t = (i + 1) / TICKS_PER_BAR
        
        # Brownian bridge with micro-trend: drift toward close with noise
        target = o * (1 - t) + c * t
        noise = random.gauss(0, half_range * 0.15 * math.sqrt(t * (1 - t)))
        price = target + noise
        price = max(l - 0.1, min(h + 0.1, price))
        
        # 2. Spread: wider during fast moves, tighter in calm
        local_spread = spread * (1 + abs(price - center) / half_range * 0.5)
        local_spread = max(0.1, local_spread)
        
        # 3. Bid-ask with bounce
        if i % 2 == 0:
            bid = round(price - local_spread / 2, 3)
            ask = round(price + local_spread / 2, 3)
        else:
            bid = round(price - local_spread / 3, 3)
            ask = round(price + local_spread * 2 / 3, 3)
        
        # 4. Volume clustering: burst near middle of bar
        vol_base = max(1, v // TICKS_PER_BAR)
        if 0.3 < t < 0.7 and random.random() < 0.3:
            vol_mult = 2 + random.random() * 3  # burst 2-5x
        else:
            vol_mult = 0.5 + random.random()
        
        bv = int(vol_base * vol_mult * (0.5 + random.random()))
        av = int(vol_base * vol_mult * (0.5 + random.random()))
        
        ticks.append([bid, ask, bv, av])
    
    return ticks

def main():
    random.seed(42)
    with open(IN_CSV) as f:
        reader = csv.DictReader(f)
        bars = list(reader)
    
    print(f"Generating {len(bars):,} M1 bars with microstructure ticks...")
    with open(OUT_CSV, "w", newline="") as f:
        w = csv.writer(f)
        w.writerow(["time_ms", "bid", "ask", "bid_vol", "ask_vol"])
        
        total_ticks = 0
        for bar in bars:
            ts = int(bar["time"])
            o, h, l, c = float(bar["open"]), float(bar["high"]), float(bar["low"]), float(bar["close"])
            volume = int(float(bar["volume"]))
            spread_base = (h - l) * 0.02  # ~2% of range as spread
            
            ticks = generate_ticks(o, h, l, c, volume, spread_base)
            for i, (bid, ask, bv, av) in enumerate(ticks):
                time_ms = (ts + i) * 1000 + random.randint(0, 999)
                w.writerow([time_ms, bid, ask, bv, av])
                total_ticks += 1
            
            if total_ticks % 500000 == 0:
                print(f"  {total_ticks:,} ticks...", flush=True)
    
    size_mb = OUT_CSV.stat().st_size / 1024 / 1024
    print(f"\nDone: {total_ticks:,} ticks -> {OUT_CSV} ({size_mb:.1f} MB)")

if __name__ == "__main__":
    main()
