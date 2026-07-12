"""AI SEITH Python Bridge — MT5 API wrapper"""

import MetaTrader5 as mt5
from typing import Optional, Union
from datetime import datetime

_initialized = False


def init_mt5(path: str = "") -> bool:
    """Initialize MT5 terminal connection"""
    global _initialized
    if _initialized:
        return True
    # Attempt initialize
    initialized = False
    if path:
        print(f"[MT5] Attempting initialize with path: {path}")
        initialized = mt5.initialize(path)
    else:
        print("[MT5] Attempting initialize with default path")
        initialized = mt5.initialize()

    if not initialized:
        err_code, err_desc = mt5.last_error()
        print(f"[MT5] Initialize failed: code={err_code}, desc={err_desc}")
        return False
    _initialized = True
    print(f"[MT5] Terminal initialized (build {mt5.version()})")
    return True


def login(account: Union[int, str], password: str, server: str) -> bool:
    """Login to MT5 trading account"""
    if not init_mt5():
        return False
    authorized = mt5.login(account, password=password, server=server)
    if not authorized:
        err_code, err_desc = mt5.last_error()
        print(f"[MT5] Login failed: code={err_code}, desc={err_desc}")
        return False
    print(f"[MT5] Logged in to account {account} on {server}")
    return True


def get_price(symbol: str) -> Optional[float]:
    """Get current bid price for symbol"""
    tick = mt5.symbol_info_tick(symbol)
    if tick is None:
        print(f"[MT5] No tick data for {symbol}")
        return None
    return tick.bid


def get_tick(symbol: str) -> Optional[dict]:
    """Get full tick data"""
    tick = mt5.symbol_info_tick(symbol)
    if tick is None:
        return None
    return {
        "bid": tick.bid,
        "ask": tick.ask,
        "time": datetime.fromtimestamp(tick.time),
    }


def get_rates_json(symbol: str, count: int = 100, timeframe: int = mt5.TIMEFRAME_M1) -> Optional[str]:
    """Get OHLCV rates as JSON string for Rust FFI."""
    rates = mt5.copy_rates_from_pos(symbol, timeframe, 0, count)
    if rates is None:
        return None
    result = []
    for r in rates:
        result.append({
            "time": int(r['time']),
            "open": float(r['open']),
            "high": float(r['high']),
            "low": float(r['low']),
            "close": float(r['close']),
            "volume": int(r['tick_volume']),
        })
    return json.dumps(result)
    """Get OHLCV rates"""
    rates = mt5.copy_rates_from_pos(symbol, timeframe, 0, count)
    if rates is None:
        return None
    result = []
    for r in rates:
        result.append({
            "time": datetime.fromtimestamp(r.time),
            "open": r.open,
            "high": r.high,
            "low": r.low,
            "close": r.close,
            "volume": r.tick_volume,
        })
    return result


def place_order(
    symbol: str,
    order_type: int,
    volume: float,
    price: float,
    sl: float = 0.0,
    tp: float = 0.0,
    comment: str = "",
) -> Optional[int]:
    """Place order via MT5"""
    request = {
        "action": mt5.TRADE_ACTION_DEAL,
        "symbol": symbol,
        "volume": volume,
        "type": order_type,
        "price": price,
        "sl": sl,
        "tp": tp,
        "deviation": 10,
        "magic": 1001,
        "comment": comment,
        "type_time": mt5.ORDER_TIME_GTC,
        "type_filling": mt5.ORDER_FILLING_IOC,
    }
    result = mt5.order_send(request)
    if result.retcode != mt5.TRADE_RETCODE_DONE:
        print(f"[MT5] Order failed: {result.comment} (code {result.retcode})")
        return None
    print(f"[MT5] Order placed: ticket={result.order}")
    return result.order


def place_pending_order(
    symbol: str,
    order_type: int,
    volume: float,
    price: float,
    sl: float = 0.0,
    tp: float = 0.0,
    comment: str = "AI SEITH",
) -> Optional[int]:
    """Place pending order (Limit/Stop) via MT5 with SL/TP bracket."""
    request = {
        "action": mt5.TRADE_ACTION_PENDING,
        "symbol": symbol,
        "volume": volume,
        "type": order_type,
        "price": price,
        "sl": sl,
        "tp": tp,
        "deviation": 10,
        "magic": 1001,
        "comment": comment,
        "type_time": mt5.ORDER_TIME_GTC,
        "type_filling": mt5.ORDER_FILLING_RETURN,
    }
    result = mt5.order_send(request)
    if result.retcode != mt5.TRADE_RETCODE_DONE:
        print(f"[MT5] Pending order failed: {result.comment} (code {result.retcode})")
        return None
    print(f"[MT5] Pending order placed: ticket={result.order}")
    return result.order


import json

def get_tick_json(symbol: str) -> Optional[str]:
    """Get tick data as JSON string for Rust FFI."""
    tick = mt5.symbol_info_tick(symbol)
    if tick is None:
        return None
    return json.dumps({
        "bid": tick.bid,
        "ask": tick.ask,
        "time": datetime.fromtimestamp(tick.time).isoformat(),
    })


def get_dom_json(symbol: str) -> Optional[str]:
    """Get DOM as JSON string for Rust FFI."""
    mt5.symbol_select(symbol, True)
    subscribed = mt5.market_book_add(symbol)
    if not subscribed:
        return None
    import time
    time.sleep(0.5)
    raw = mt5.market_book_get(symbol)
    if raw is None or len(raw) == 0:
        return None
    asks_raw = [b for b in raw if b.type == 1]
    bids_raw = [b for b in raw if b.type == 2]
    asks = [{"price": round(b.price, 3), "volume": b.volume} for b in reversed(asks_raw)]
    bids = [{"price": round(b.price, 3), "volume": b.volume} for b in bids_raw]
    best_ask = asks[0]["price"] if asks else 0.0
    best_bid = bids[0]["price"] if bids else 0.0
    return json.dumps({
        "symbol": symbol,
        "asks": asks,
        "bids": bids,
        "best_ask": best_ask,
        "best_bid": best_bid,
        "level_count": len(raw),
    })


def get_dom(symbol: str) -> Optional[dict]:
    """Get Depth of Market snapshot.

    Raw MT5 book returned as single list sorted HIGHEST→LOWEST price.
    MT5 BookInfo types:
      type=1 = BOOK_TYPE_SELL = ASK (Limit Sell, higher prices)
      type=2 = BOOK_TYPE_BUY  = BID (Limit Buy, lower prices)

    Returns parsed dict:
      - asks: type=1, sorted LOWEST→HIGHEST (best ask = index 0)
      - bids: type=2, sorted HIGHEST→LOWEST (best bid = index 0)
      - best_ask, best_bid, level_count
    """
    subscribed = mt5.market_book_add(symbol)
    if not subscribed:
        return None

    raw = mt5.market_book_get(symbol)
    if raw is None or len(raw) == 0:
        return None

    # type=1 = ASK (sell limit), type=2 = BID (buy limit)
    # Raw array is HIGHEST→LOWEST price, ASK at top, BID at bottom
    asks_raw = [b for b in raw if b.type == 1]   # higher price zone
    bids_raw = [b for b in raw if b.type == 2]    # lower price zone

    # ASK: reverse to LOWEST→HIGHEST (best ask = index 0)
    asks = [{"price": round(b.price, 3), "volume": b.volume} for b in reversed(asks_raw)]
    # BID: keep HIGHEST→LOWEST (best bid = index 0)
    bids = [{"price": round(b.price, 3), "volume": b.volume} for b in bids_raw]

    best_ask = asks[0]["price"] if asks else 0.0
    best_bid = bids[0]["price"] if bids else 0.0

    return {
        "symbol": symbol,
        "asks": asks,
        "bids": bids,
        "best_ask": best_ask,
        "best_bid": best_bid,
        "level_count": len(raw),
    }


def shutdown() -> None:
    """Shutdown MT5 connection"""
    global _initialized
    if not _initialized:
        return
    mt5.shutdown()
    _initialized = False
    print("[MT5] Shutdown")
