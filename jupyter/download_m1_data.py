"""Download XAUUSD.sml M1 OHLCV in monthly chunks."""
import sys, os, time, csv
from pathlib import Path
from datetime import datetime, timedelta

sys.path.insert(0, str(Path(__file__).parent.parent / "python"))
import MetaTrader5 as mt5
import numpy as np

SYMBOL = "XAUUSD.sml"
MONTHS = 3

def download_month(symbol, year, month):
    start = datetime(year, month, 1)
    if month == 12:
        end = datetime(year + 1, 1, 1) - timedelta(seconds=1)
    else:
        end = datetime(year, month + 1, 1) - timedelta(seconds=1)
    rates = mt5.copy_rates_range(symbol, mt5.TIMEFRAME_M1, start, end)
    if rates is None or len(rates) == 0:
        return np.array([])
    return rates

def main():
    mt5.initialize(path="C:\\Program Files\\OANDA Global MetaTrader 5 Terminal\\terminal64.exe")
    mt5.login(1715547260, "Jakarta0910.", "OANDA_Global-Demo-1")
    time.sleep(2)

    now = datetime.now()
    all_rates = np.array([])
    for i in range(MONTHS):
        m = now.month - i
        y = now.year
        if m <= 0: m += 12; y -= 1
        rates = download_month(SYMBOL, y, m)
        if len(rates) > 0:
            if len(all_rates) == 0:
                all_rates = rates
            else:
                all_rates = np.concatenate([all_rates, rates])
            print(f"{y}-{m:02d}: {len(rates):,} bars")
        time.sleep(0.5)

    if len(all_rates) == 0:
        print("No data downloaded!")
        mt5.shutdown()
        return

    # Sort by time and deduplicate
    all_rates = np.sort(all_rates, order="time")
    _, idx = np.unique(all_rates["time"], return_index=True)
    unique = all_rates[idx]

    out_dir = Path(__file__).parent / "backtest_analysis"
    out_dir.mkdir(parents=True, exist_ok=True)
    out_path = out_dir / "xauusd_m1_3m.csv"

    with open(out_path, "w", newline="") as f:
        w = csv.writer(f)
        w.writerow(["time", "open", "high", "low", "close", "volume"])
        for r in unique:
            w.writerow([int(r["time"]), r["open"], r["high"], r["low"], r["close"], int(r["tick_volume"])])

    print(f"\nTotal: {len(unique):,} unique M1 bars")
    print(f"Saved: {out_path}")
    print(f"Size:  {os.path.getsize(out_path) / 1024 / 1024:.1f} MB")
    mt5.shutdown()

if __name__ == "__main__":
    main()
