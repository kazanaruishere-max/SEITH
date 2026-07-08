"""AI SEITH Python Bridge — Telegram Bot API wrapper"""

import asyncio
import sys
from typing import Optional
from telegram import Bot
from telegram.error import TelegramError

_bot: Optional[Bot] = None
_loop: Optional[asyncio.AbstractEventLoop] = None


def _get_loop() -> asyncio.AbstractEventLoop:
    global _loop
    if _loop is None or _loop.is_closed():
        _loop = asyncio.new_event_loop()
        asyncio.set_event_loop(_loop)
    return _loop


def init_telegram(token: str) -> bool:
    """Initialize Telegram bot"""
    global _bot
    try:
        _bot = Bot(token=token)
        loop = _get_loop()
        me = loop.run_until_complete(_bot.get_me())
        print(f"[Telegram] Bot initialized: @{me.username}", file=sys.stderr)
        return True
    except TelegramError as e:
        print(f"[Telegram] Init failed: {e}", file=sys.stderr)
        return False


def send_message(chat_id: str, text: str) -> bool:
    """Send message to Telegram chat (sync)"""
    global _bot
    if _bot is None:
        print("[Telegram] Bot not initialized", file=sys.stderr)
        return False
    try:
        loop = _get_loop()
        loop.run_until_complete(_bot.send_message(chat_id=chat_id, text=text, parse_mode="HTML"))
        return True
    except TelegramError as e:
        print(f"[Telegram] Send message failed: {e}", file=sys.stderr)
        return False


def send_photo(chat_id: str, photo_path: str, caption: str = "") -> bool:
    """Send photo with caption (sync)"""
    global _bot
    if _bot is None:
        print("[Telegram] Bot not initialized", file=sys.stderr)
        return False
    try:
        loop = _get_loop()
        with open(photo_path, "rb") as f:
            loop.run_until_complete(
                _bot.send_photo(chat_id=chat_id, photo=f, caption=caption, parse_mode="HTML")
            )
        return True
    except TelegramError as e:
        print(f"[Telegram] Send photo failed: {e}", file=sys.stderr)
        return False
    except FileNotFoundError:
        print(f"[Telegram] Photo not found: {photo_path}", file=sys.stderr)
        return False
