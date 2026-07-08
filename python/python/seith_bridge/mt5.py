"""AI SEITH Python Bridge — MT5 API wrapper"""
# Stub only — no implementation yet

import MetaTrader5 as mt5
from typing import Optional


def init_mt5() -> bool:
    """Initialize MT5 terminal connection"""
    raise NotImplementedError("MT5 init not yet implemented")


def get_price(symbol: str) -> Optional[float]:
    """Get current price for symbol"""
    raise NotImplementedError("Price fetch not yet implemented")


def place_order(symbol: str, order_type: int, volume: float) -> Optional[int]:
    """Place order via MT5"""
    raise NotImplementedError("Order placement not yet implemented")
