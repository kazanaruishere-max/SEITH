"""AI SEITH Python Bridge — Telegram Bot API wrapper"""

from typing import Optional
from telegram import Bot
from telegram.error import TelegramError


_bot: Optional[Bot] = None


def init_telegram(token: str) -> bool:
    """Initialize Telegram bot"""
    global _bot
    try:
        _bot = Bot(token=token)
        me = _bot.get_me()
        print(f"[Telegram] Bot initialized: @{me.username}")
        return True
    except TelegramError as e:
        print(f"[Telegram] Init failed: {e}")
        return False


async def send_message(chat_id: str, text: str) -> bool:
    """Send message to Telegram chat"""
    global _bot
    if _bot is None:
        print("[Telegram] Bot not initialized")
        return False
    try:
        await _bot.send_message(chat_id=chat_id, text=text, parse_mode="HTML")
        return True
    except TelegramError as e:
        print(f"[Telegram] Send message failed: {e}")
        return False


async def send_photo(chat_id: str, photo_path: str, caption: str = "") -> bool:
    """Send photo with caption"""
    global _bot
    if _bot is None:
        print("[Telegram] Bot not initialized")
        return False
    try:
        with open(photo_path, "rb") as f:
            await _bot.send_photo(
                chat_id=chat_id, photo=f, caption=caption, parse_mode="HTML"
            )
        return True
    except TelegramError as e:
        print(f"[Telegram] Send photo failed: {e}")
        return False
    except FileNotFoundError:
        print(f"[Telegram] Photo not found: {photo_path}")
        return False
