"""AI SEITH Python Bridge — MT5 API wrapper"""

import MetaTrader5 as mt5
from typing import Optional
from datetime import datetime


def init_mt5(path: str = "") -> bool:
    """Initialize MT5 terminal connection"""
    initialized = mt5.initialize(path=path) if path else mt5.initialize()
    if not initialized:
        error = mt5.last_error()
        print(f"[MT5] Initialize failed: {error}")
        return False
    print(f"[MT5] Terminal initialized (build {mt5.version()})")
    return True


def login(account: int, password: str, server: str) -> bool:
    """Login to MT5 trading account"""
    authorized = mt5.login(account, password=password, server=server)
    if not authorized:
        error = mt5.last_error()
        print(f"[MT5] Login failed: {error}")
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


def get_rates(symbol: str, count: int = 100, timeframe: int = mt5.TIMEFRAME_M15) -> Optional[list]:
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


def shutdown() -> None:
    """Shutdown MT5 connection"""
    mt5.shutdown()
    print("[MT5] Shutdown")
