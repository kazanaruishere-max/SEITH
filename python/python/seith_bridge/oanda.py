"""AI SEITH Bridge — Market sentiment & historical data from OANDA API v20.

Sources:
1. Myfxbook Community Outlook (scraped, works with cloudscraper)
2. OANDA position book (paid API only)
3. OANDA REST API v20 — historical candles (bid/ask)
4. Fallback: None
"""

import json
import os
import re
from datetime import datetime, timezone
from typing import Optional
from urllib.request import Request, urlopen
from urllib.error import HTTPError


def _oanda_api_headers() -> dict:
    token = os.environ.get("OANDA_API_TOKEN", "")
    return {
        "Authorization": f"Bearer {token}",
        "Content-Type": "application/json",
    }


def _oanda_base_url() -> str:
    """Use demo endpoint. Change to api-fxtrade.com for live."""
    return "https://api-fxpractice.oanda.com/v3"


def get_sentiment(instrument: str = "XAU_USD") -> Optional[str]:
    """Fetch market sentiment % long/short.

    Args:
        instrument: OANDA format instrument (e.g. "XAU_USD")

    Returns JSON:
        {"long_pct": 53.0, "short_pct": 47.0, "source": "myfxbook"}
        {"long_pct": None} on failure
    """
    # 1. Myfxbook Community Outlook
    try:
        return _fetch_myfxbook_outlook(instrument)
    except Exception as e:
        print(f"[SENTIMENT] Myfxbook failed: {e}", flush=True)

    return json.dumps({"error": "unavailable", "long_pct": None})


def _fetch_myfxbook_outlook(instrument: str) -> str:
    """Scrape Myfxbook Community Outlook for long/short percentages.

    Myfxbook provides free, publicly accessible retail trader sentiment.
    URL: https://www.myfxbook.com/community/outlook
    """
    import cloudscraper
    from bs4 import BeautifulSoup

    # Convert OANDA format (XAU_USD) to Myfxbook format (XAUUSD)
    sym = instrument.replace("_", "").replace(".sml", "")

    scraper = cloudscraper.create_scraper()
    resp = scraper.get("https://www.myfxbook.com/community/outlook", timeout=15)

    if resp.status_code != 200:
        return json.dumps({"error": f"HTTP {resp.status_code}", "long_pct": None})

    soup = BeautifulSoup(resp.text, "html.parser")

    # Strategy 1: Find the data row for our symbol
    for table in soup.find_all("table"):
        rows = table.find_all("tr")
        for row in rows:
            cells = row.find_all("td")
            if len(cells) < 5:
                continue
            row_text = row.get_text()
            if sym not in row_text:
                continue
            # Found our symbol row — extract Short % and Long %
            # Cell structure: [Symbol, Short, %, lots, positions, Long, %, lots, positions]
            texts = [c.get_text(strip=True) for c in cells]
            short_pct = None
            long_pct = None
            for i, t in enumerate(texts):
                if t == "Short" and i + 1 < len(texts):
                    match = re.match(r"(\d+)%", texts[i + 1])
                    if match:
                        short_pct = float(match.group(1))
                if t == "Long" and i + 1 < len(texts):
                    match = re.match(r"(\d+)%", texts[i + 1])
                    if match:
                        long_pct = float(match.group(1))

            if long_pct and short_pct:
                return json.dumps({
                    "long_pct": long_pct,
                    "short_pct": short_pct,
                    "source": "myfxbook",
                })

    # Strategy 2: Regex fallback
    pattern = rf"{sym}.*?Short.*?(\d+)%.*?Long.*?(\d+)%"
    match = re.search(pattern, resp.text, re.DOTALL)
    if match:
        return json.dumps({
            "long_pct": float(match.group(2)),
            "short_pct": float(match.group(1)),
            "source": "myfxbook",
        })

    return json.dumps({"error": "parse failed", "long_pct": None})


def get_historical_candles(
    instrument: str = "US100_USD",
    granularity: str = "M15",
    count: int = 5000,
    from_date: str = "",
    to_date: str = "",
) -> str:
    """Fetch historical OHLC + bid/ask candles from OANDA REST API v20.

    Args:
        instrument: OANDA format (e.g. US100_USD, XAU_USD)
        granularity: M1, M5, M15, M30, H1, H4, D, W, M
        count: max candles to return (API limit 5000)
        from_date: ISO 8601 e.g. 2025-06-01T00:00:00Z
        to_date: ISO 8601

    Returns JSON:
        [{"time": "2025-06-01T13:30:00Z",
          "bo": bid_open, "bh": bid_high, "bl": bid_low, "bc": bid_close,
          "ao": ask_open, "ah": ask_high, "al": ask_low, "ac": ask_close,
          "volume": tick_count}, ...]
        {"error": "..."} on failure
    """
    try:
        url = f"{_oanda_base_url()}/instruments/{instrument}/candles"
        params = f"price=BA&granularity={granularity}&count={count}"
        if from_date:
            params += f"&from={from_date}"
        if to_date:
            params += f"&to={to_date}"

        req = Request(f"{url}?{params}", headers=_oanda_api_headers())
        resp = urlopen(req, timeout=30)
        raw = json.loads(resp.read().decode())

        candles = raw.get("candles", [])
        result = []
        for c in candles:
            if c.get("complete", False) is False:
                continue  # skip incomplete bar
            bid = c.get("bid", {})
            ask = c.get("ask", {})
            result.append({
                "time": c["time"],
                "bo": float(bid.get("o", 0)),
                "bh": float(bid.get("h", 0)),
                "bl": float(bid.get("l", 0)),
                "bc": float(bid.get("c", 0)),
                "ao": float(ask.get("o", 0)),
                "ah": float(ask.get("h", 0)),
                "al": float(ask.get("l", 0)),
                "ac": float(ask.get("c", 0)),
                "volume": int(c.get("volume", 0)),
            })
        return json.dumps(result)

    except HTTPError as e:
        body = e.read().decode()
        return json.dumps({"error": f"HTTP {e.code}: {body}"})
    except Exception as e:
        return json.dumps({"error": str(e)})
