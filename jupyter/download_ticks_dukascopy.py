"""Download XAUUSD tick data from Dukascopy (bypassing ISP DNS block via Cloudflare DNS).
Uses requests library with custom DNS resolution.
"""
import os, sys, struct, lzma, csv, time
from pathlib import Path
from datetime import datetime, timedelta
import socket

# DNS override: bypass trustpositif block by resolving via Cloudflare DNS
DUKASCOPY_IP = '104.20.28.213'
_original_getaddrinfo = socket.getaddrinfo
def _duka_getaddrinfo(host, port, family=0, type=0, proto=0, flags=0):
    if 'dukascopy' in str(host).lower():
        host = DUKASCOPY_IP
        family = socket.AF_INET  # force IPv4
    return _original_getaddrinfo(host, port, family, type, proto, flags)
socket.getaddrinfo = _duka_getaddrinfo

import requests

SYMBOL = "XAUUSD"
START = datetime(2026, 4, 1)
END = datetime(2026, 7, 9)
OUT_DIR = Path(__file__).parent / "backtest_analysis"
OUT_CSV = OUT_DIR / "xauusd_ticks_dukascopy.csv"

def download_hour(dt: datetime) -> list:
    """Download 1 hour of tick data."""
    url = f"https://www.dukascopy.com/datafeed/{SYMBOL}/{dt.year}/{dt.month:02d}/{dt.day:02d}/{dt.hour:02d}h_ticks.bi5"
    try:
        resp = requests.get(url, headers={"User-Agent": "Mozilla/5.0"}, timeout=30)
        if resp.status_code != 200 or len(resp.content) < 100:
            return []
        decompressed = lzma.decompress(resp.content)
        tick_size = 20
        num_ticks = len(decompressed) // tick_size
        base_ts = int(dt.timestamp() * 1000)
        ticks = []
        for i in range(num_ticks):
            offset = i * tick_size
            vals = struct.unpack(">5i", decompressed[offset:offset + tick_size])
            ticks.append({
                "time_ms": base_ts + vals[0],
                "ask": vals[1] / 100000,
                "bid": vals[2] / 100000,
                "ask_vol": vals[3],
                "bid_vol": vals[4],
            })
        return ticks
    except Exception as e:
        return []

def main():
    OUT_DIR.mkdir(parents=True, exist_ok=True)
    
    total_ticks = 0
    date = START
    with open(OUT_CSV, "w", newline="") as f:
        w = csv.writer(f)
        w.writerow(["time_ms", "bid", "ask", "bid_vol", "ask_vol"])
        
        while date < END:
            for hour in range(24):
                ticks = download_hour(date)
                if ticks:
                    for t in ticks:
                        w.writerow([t["time_ms"], t["bid"], t["ask"], t["bid_vol"], t["ask_vol"]])
                    total_ticks += len(ticks)
                    if len(ticks) > 500:
                        print(f"  {date.date()} {hour:02d}: {len(ticks)} ticks", flush=True)
                time.sleep(0.15)
            date += timedelta(days=1)
    
    size_mb = OUT_CSV.stat().st_size / 1024 / 1024
    print(f"\nTotal: {total_ticks:,} ticks")
    print(f"Saved: {OUT_CSV} ({size_mb:.1f} MB)")

if __name__ == "__main__":
    main()
