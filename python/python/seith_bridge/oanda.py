"""AI SEITH Bridge — Sentiment data for Bayesian prior.

Currently using built-in market structure prior (no external API needed).
Provides prior P(A) as fallback when external sentiment unavailable.
"""

import json
from typing import Optional


def get_sentiment(instrument: str = "XAU_USD") -> Optional[str]:
    """Fetch market sentiment % long/short.

    External sentiment sources are aggressively blocked (Cloudflare).
    Returns None to trigger built-in market-structure prior in Rust.
    """
    return json.dumps({"error": "unavailable", "long_pct": None})
