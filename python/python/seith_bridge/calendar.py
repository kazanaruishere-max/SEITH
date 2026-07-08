"""AI SEITH Python Bridge — Economic Calendar Module
Multi-source: cloudscraper (ForexFactory bypass) + TradingEconomics API
Focus: high-impact USD events (FOMC, NFP, CPI, etc.)"""

import json
import os
from typing import Optional
from datetime import datetime, timedelta
import re

# High-impact USD events that affect XAUUSD
HIGH_IMPACT_USD = [
    "FOMC", "Non Farm Payrolls", "NF", "CPI", "PPI",
    "Retail Sales", "GDP", "Initial Jobless Claims", "Unemployment Rate",
    "ISM Manufacturing", "ISM Services", "Industrial Production",
    "Consumer Confidence", "Building Permits", "Housing Starts",
    "Durable Goods Orders", "Factory Orders", "Trade Balance",
    "Philadelphia Fed", "Empire State", "Michigan Consumer Sentiment",
    "Treasury", "Federal Budget",
]

# ── Source 1: ForexFactory via cloudscraper (bypass CloudFlare) ──

def _fetch_ff_via_cloudscraper(url: str) -> Optional[str]:
    """Fetch ForexFactory using cloudscraper to bypass CloudFlare"""
    try:
        import cloudscraper
        scraper = cloudscraper.create_scraper()
        resp = scraper.get(url, timeout=20)
        if resp.status_code != 200:
            print(f"[Calendar] ForexFactory HTTP {resp.status_code}")
            return None
    except Exception as e:
        print(f"[Calendar] ForexFactory cloudscraper failed: {e}")
        return None

    from bs4 import BeautifulSoup
    soup = BeautifulSoup(resp.text, "html.parser")
    events = []
    for row in soup.select("tr.calendar__row"):
        cols = row.select("td")
        if len(cols) < 5:
            continue
        time_text = cols[0].get_text(strip=True)
        currency = cols[1].get_text(strip=True)
        impact_el = cols[2].select_one("span")
        impact = impact_el.get("title", "") if impact_el else ""
        title = cols[3].get_text(strip=True)
        actual = cols[5].get_text(strip=True) if len(cols) > 5 else ""
        forecast = cols[6].get_text(strip=True) if len(cols) > 6 else ""
        previous = cols[7].get_text(strip=True) if len(cols) > 7 else ""

        if currency != "USD":
            continue

        events.append({
            "time": datetime.now().strftime("%Y-%m-%d") + " " + time_text,
            "currency": currency,
            "impact": impact,
            "title": title,
            "actual": actual,
            "forecast": forecast,
            "previous": previous,
        })

    return json.dumps(events) if events else None


# ── Source 2: TradingEconomics API (optional, needs API key) ──

def _fetch_via_tradingeconomics() -> Optional[str]:
    """Fetch economic calendar from TradingEconomics API"""
    api_key = os.getenv("TRADINGECONOMICS_API_KEY", "")
    if not api_key:
        return None
    try:
        import requests
        url = f"https://api.tradingeconomics.com/calendar/country/united%20states?c={api_key}&f=json"
        resp = requests.get(url, timeout=15)
        if resp.status_code != 200:
            return None
        data = resp.json()
        events = []
        for item in data[:50]:
            title = item.get("Event", "")
            if not any(kw.lower() in title.lower() for kw in HIGH_IMPACT_USD):
                continue
            events.append({
                "time": item.get("Date", ""),
                "currency": "USD",
                "impact": "High Impact",
                "title": title,
                "actual": str(item.get("Actual", "") or ""),
                "forecast": str(item.get("Forecast", "") or ""),
                "previous": str(item.get("Previous", "") or ""),
            })
        return json.dumps(events) if events else None
    except Exception as e:
        print(f"[Calendar] TradingEconomics API failed: {e}")
        return None


# ── Source 3: Hardcoded schedule fallback ──

def _generate_schedule() -> str:
    """Generate known high-impact USD event schedule as fallback"""
    today = datetime.now()
    events = []

    # NFP: first Friday of every month
    nfp_dates = []
    for m in range(1, 13):
        first_day = datetime(today.year, m, 1)
        days_to_friday = (4 - first_day.weekday()) % 7
        nfp_date = first_day + timedelta(days=days_to_friday)
        if nfp_date >= today - timedelta(days=7):
            nfp_dates.append(nfp_date)

    for d in nfp_dates[:3]:
        events.append({
            "time": d.strftime("%Y-%m-%d %H:%M:%S"),
            "currency": "USD",
            "impact": "High Impact",
            "title": "Non Farm Payrolls (NFP)",
            "actual": "",
            "forecast": "",
            "previous": "",
        })

    # FOMC: Jan, Mar, May, Jun, Jul, Sep, Nov, Dec (8x/year)
    fomc_months = [1, 3, 5, 6, 7, 9, 11, 12]
    for m in fomc_months:
        fomc_date = datetime(today.year, m, 15)
        if fomc_date >= today - timedelta(days=7):
            events.append({
                "time": fomc_date.strftime("%Y-%m-%d") + " 14:00:00",
                "currency": "USD",
                "impact": "High Impact",
                "title": "FOMC Meeting",
                "actual": "",
                "forecast": "",
                "previous": "",
            })

    # CPI: monthly, usually 10th-15th
    for m in range(1, 13):
        cpi_date = datetime(today.year, m, 13)
        if cpi_date >= today - timedelta(days=7):
            events.append({
                "time": cpi_date.strftime("%Y-%m-%d") + " 08:30:00",
                "currency": "USD",
                "impact": "High Impact",
                "title": "Consumer Price Index (CPI)",
                "actual": "",
                "forecast": "",
                "previous": "",
            })

    return json.dumps(events) if events else "[]"


# ── Main entry point ──

def fetch_economic_calendar() -> Optional[str]:
    """Fetch economic calendar: cloudscraper → API → schedule fallback"""

    # Try cloudscraper (bypass CloudFlare)
    data = _fetch_ff_via_cloudscraper("https://www.forexfactory.com/calendar")
    if data:
        return data

    # Try TradingEconomics API
    data = _fetch_via_tradingeconomics()
    if data:
        return data

    # Fallback: hardcoded schedule
    data = _generate_schedule()
    print(f"[Calendar] Using schedule fallback: {len(json.loads(data))} events")
    return data if data != "[]" else None
