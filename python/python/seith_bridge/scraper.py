"""AI SEITH Python Bridge — Web scraper wrapper"""

from typing import Optional
from datetime import datetime
import re

import requests
from bs4 import BeautifulSoup


def fetch_forex_factory(url: str) -> Optional[list]:
    """Scrape ForexFactory calendar for news events"""
    headers = {
        "User-Agent": (
            "Mozilla/5.0 (Windows NT 10.0; Win64; x64) "
            "AppleWebKit/537.36 (KHTML, like Gecko) "
            "Chrome/120.0.0.0 Safari/537.36"
        )
    }
    try:
        resp = requests.get(url, headers=headers, timeout=15)
        resp.raise_for_status()
    except requests.RequestException as e:
        print(f"[Scraper] ForexFactory request failed: {e}")
        return None

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

        events.append({
            "time": time_text,
            "currency": currency,
            "impact": impact,
            "title": title,
            "actual": actual,
            "forecast": forecast,
            "previous": previous,
        })

    return events if events else None


def fetch_investing_com(url: str) -> Optional[list]:
    """Scrape Investing.com calendar using Playwright"""
    try:
        from playwright.sync_api import sync_playwright
    except ImportError:
        print("[Scraper] Playwright not installed")
        return None

    try:
        with sync_playwright() as p:
            browser = p.chromium.launch(headless=True)
            page = browser.new_page()
            page.goto(url, wait_until="networkidle", timeout=30000)
            page.wait_for_selector("table.calendar_table", timeout=10000)

            rows = page.query_selector_all("tr.js-event-item")
            events = []
            for row in rows:
                cols = row.query_selector_all("td")
                if len(cols) < 6:
                    continue
                time_text = cols[0].inner_text().strip()
                currency = cols[2].inner_text().strip() if len(cols) > 2 else ""
                impact = cols[3].inner_text().strip() if len(cols) > 3 else ""
                title = cols[4].inner_text().strip() if len(cols) > 4 else ""
                actual = cols[5].inner_text().strip() if len(cols) > 5 else ""
                forecast = cols[6].inner_text().strip() if len(cols) > 6 else ""
                previous = cols[7].inner_text().strip() if len(cols) > 7 else ""

                events.append({
                    "time": time_text,
                    "currency": currency,
                    "impact": impact,
                    "title": title,
                    "actual": actual,
                    "forecast": forecast,
                    "previous": previous,
                })

            browser.close()
            return events if events else None

    except Exception as e:
        print(f"[Scraper] Investing.com failed: {e}")
        return None
