"""AI SEITH Bridge — OANDA REST API v3 for sentiment data.

Provides:
- Position book (% long/short) for Bayesian prior P(A)
- Accessible from Rust via PyO3/JSON bridge
"""

import json
import os
from typing import Optional
from urllib.request import Request, urlopen

_OANDA_TOKEN: Optional[str] = None
_OANDA_ACCOUNT: Optional[str] = None
_BASE_URL: Optional[str] = None


def init_oanda(token: str, account_id: str, practice: bool = True) -> bool:
    """Initialize OANDA REST API client.

    Args:
        token: API token from OANDA (bearer token).
        account_id: OANDA account ID (format: "101-001-1234567-001").
        practice: True for demo/practice, False for live.
    """
    global _OANDA_TOKEN, _OANDA_ACCOUNT, _BASE_URL
    _OANDA_TOKEN = token
    _OANDA_ACCOUNT = account_id
    _BASE_URL = "https://api-fxpractice.oanda.com" if practice else "https://api-fxtrade.oanda.com"
    print(f"[OANDA] Initialized practice={practice}", flush=True)
    return True


def get_sentiment(instrument: str = "XAU_USD") -> Optional[str]:
    """Fetch OANDA position book and return % long/short as JSON.

    Returns JSON string: {"long_pct": 0.65, "short_pct": 0.35, "time": "..."}
    WARNING: positionBook API requires a premium OANDA account tier.
    Some demo accounts may return {"error": "not available"}.
    """
    if not _OANDA_TOKEN or not _OANDA_ACCOUNT:
        # Try env vars as fallback
        token = os.environ.get("OANDA_API_TOKEN")
        account = os.environ.get("OANDA_ACCOUNT_ID")
        if token and account:
            init_oanda(token, account)
        else:
            return json.dumps({"error": "OANDA not configured", "long_pct": None})

    url = f"{_BASE_URL}/v3/accounts/{_OANDA_ACCOUNT}/positionBook?instrument={instrument}"
    headers = {
        "Authorization": f"Bearer {_OANDA_TOKEN}",
        "Content-Type": "application/json",
    }

    try:
        req = Request(url, headers=headers)
        with urlopen(req, timeout=10) as resp:
            data = json.loads(resp.read().decode())

        # Parse position book
        if "positionBook" not in data:
            return json.dumps({"error": "No positionBook in response", "long_pct": None})

        pb = data["positionBook"]
        long_pct = pb.get("longPositionPercent", 50.0)
        short_pct = pb.get("shortPositionPercent", 50.0)
        timestamp = pb.get("time", "")

        return json.dumps({
            "long_pct": long_pct,
            "short_pct": short_pct,
            "time": timestamp,
        })

    except Exception as e:
        return json.dumps({"error": str(e), "long_pct": None})
