"""AI SEITH Bridge — Market sentiment for Bayesian prior P(A).

Sources (tried in order):
1. Myfxbook Community Outlook (scraped, works with cloudscraper)
2. OANDA position book (paid API only)
3. Fallback: None → Rust uses neutral prior
"""

import json
import re
from typing import Optional


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
