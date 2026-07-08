-- Trades table: riwayat seluruh eksekusi
CREATE TABLE IF NOT EXISTS trades (
    id          TEXT PRIMARY KEY,
    instrument  TEXT NOT NULL,
    signal_tier TEXT NOT NULL CHECK(signal_tier IN ('TIER_1', 'TIER_2', 'NEWS')),
    order_type  TEXT NOT NULL CHECK(order_type IN ('LIMIT', 'STOP', 'INSTANT')),
    direction   TEXT NOT NULL CHECK(direction IN ('BUY', 'SELL')),
    entry_price REAL NOT NULL,
    stop_loss   REAL NOT NULL,
    take_profit REAL NOT NULL,
    volume      REAL NOT NULL DEFAULT 0.01,
    status      TEXT NOT NULL DEFAULT 'PENDING' CHECK(status IN ('PENDING', 'OPEN', 'CLOSED', 'CANCELLED')),
    result      TEXT CHECK(result IN ('WIN', 'LOSS', 'BREAKEVEN')),
    slippage    REAL,
    spread_open REAL,
    profit_cent REAL,
    opened_at   TEXT NOT NULL,
    closed_at   TEXT,
    created_at  TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE INDEX IF NOT EXISTS idx_trades_instrument ON trades(instrument);
CREATE INDEX IF NOT EXISTS idx_trades_status ON trades(status);
CREATE INDEX IF NOT EXISTS idx_trades_opened_at ON trades(opened_at);

-- System state: skip counter, session status
CREATE TABLE IF NOT EXISTS system_state (
    key   TEXT PRIMARY KEY,
    value TEXT NOT NULL,
    updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);

-- Config: runtime parameters
CREATE TABLE IF NOT EXISTS config (
    key         TEXT PRIMARY KEY,
    value       TEXT NOT NULL,
    description TEXT,
    updated_at  TEXT NOT NULL DEFAULT (datetime('now'))
);
