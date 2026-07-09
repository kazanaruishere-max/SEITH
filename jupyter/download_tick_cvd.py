"""Download XAUUSD tick data from Dukascopy (free) and compute CVD per M1 bar."""
import os, sys, struct, lzma, csv, time
from pathlib import Path
from datetime import datetime, timedelta
from urllib.request import Request, urlopen

SYMBOL_DUKA = "XAUUSD"
SYMBOL = "XAUUSD.sml"
START_DATE = datetime(2026, 4, 1)
END_DATE = datetime(2026, 7, 9)

def download_ticks(symbol: str, year: int, month: int, day: int, hour: int) -> list:
    """Download 1 hour of tick data from Dukascopy."""
    url = f"https://www.dukascopy.com/datafeed/{symbol}/{year}/{month:02d}/{day:02d}/{hour:02d}h_ticks.bi5"
    req = Request(url, headers={"User-Agent": "Mozilla/5.0"})
    try:
        with urlopen(req, timeout=30) as resp:
            compressed = resp.read()
        if len(compressed) < 100:
            return []
        decompressed = lzma.decompress(compressed)
        tick_size = 20
        num_ticks = len(decompressed) // tick_size
        ticks = []
        base_ts = int(datetime(year, month, day, hour).timestamp() * 1000)
        for i in range(num_ticks):
            offset = i * tick_size
            vals = struct.unpack(">5i", decompressed[offset:offset + tick_size])
            ticks.append({
                "time_ms": base_ts + vals[0],
                "ask": vals[1],
                "bid": vals[2],
                "ask_vol": vals[3],
                "bid_vol": vals[4],
            })
        return ticks
    except Exception:
        return []

def compute_cvd(ticks: list) -> list:
    """Compute cumulative volume delta per minute from ticks.
    CVD = cumulative sum of buy/sell classified ticks.
    """
    if not ticks:
        return []
    
    ticks.sort(key=lambda t: t["time_ms"])
    prev_ask = None
    cvd = 0
    minute_data = {}
    
    for t in ticks:
        ask = t["ask"]
        minute = int((t["time_ms"] / 1000) / 60)
        
        if prev_ask is not None:
            if ask > prev_ask:
                cvd += 1
            elif ask < prev_ask:
                cvd -= 1
        
        prev_ask = ask
        minute_data[minute] = cvd
    
    return [(m, cvd) for m, cvd in sorted(minute_data.items())]

def download_range():
    """Download ticks for entire date range and compute per-minute CVD."""
    out_dir = Path(__file__).parent / "backtest_analysis"
    out_dir.mkdir(parents=True, exist_ok=True)
    csv_path = out_dir / "xauusd_m1_cvd.csv"
    
    date = START_DATE
    total_minutes = 0
    all_cvd = []
    
    while date < END_DATE:
        for hour in range(24):
            ticks = download_ticks(SYMBOL_DUKA, date.year, date.month, date.day, hour)
            if ticks:
                cvd_data = compute_cvd(ticks)
                for minute_ts, cvd_val in cvd_data:
                    abs_minute = int(date.timestamp() // 60) + minute_ts
                    all_cvd.append((abs_minute, cvd_val))
                print(f"  {date.date()} {hour:02d}: {len(ticks)} ticks, {len(cvd_data)} mins CVD", flush=True)
            time.sleep(0.2)
        date += timedelta(days=1)
    
    if not all_cvd:
        print("No tick data downloaded!")
        return
    
    # Sort and deduplicate
    all_cvd.sort(key=lambda x: x[0])
    unique = []
    seen = set()
    for ts, val in all_cvd:
        if ts not in seen:
            seen.add(ts)
            unique.append((ts, val))
    
    # Save CVD data
    with open(csv_path, "w", newline="") as f:
        w = csv.writer(f)
        w.writerow(["minute_ts", "cvd"])
        for ts, val in unique:
            w.writerow([ts, val])
    
    print(f"\nTotal: {len(unique):,} minutes of CVD data")
    print(f"Saved: {csv_path}")
    print(f"Range: CVD from {unique[0][0]} to {unique[-1][0]} (value: {unique[0][1]} to {unique[-1][1]})")

if __name__ == "__main__":
    download_range()
