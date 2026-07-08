// Trade Journal — Record trade results to SQLite

use anyhow::Result;
use shared::analytics::TradeRecord;

pub async fn save_trade(pool: &sqlx::SqlitePool, trade: &TradeRecord) -> Result<()> {
    sqlx::query(
        "INSERT INTO trades (id, instrument, signal_tier, order_type, direction, entry_price, stop_loss, take_profit, volume, status, opened_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)"
    )
    .bind(&trade.id).bind(&trade.instrument).bind(&trade.signal_tier)
    .bind(&trade.order_type).bind(&trade.direction).bind(trade.entry_price)
    .bind(trade.stop_loss).bind(trade.take_profit).bind(trade.volume)
    .bind(&trade.status).bind(&trade.opened_at)
    .execute(pool).await?;
    Ok(())
}

pub async fn close_trade(
    pool: &sqlx::SqlitePool,
    id: &str,
    result: &str,
    slippage: f64,
    profit_cent: f64,
) -> Result<()> {
    sqlx::query(
        "UPDATE trades SET status = 'CLOSED', result = ?1, slippage = ?2, profit_cent = ?3, closed_at = datetime('now')
         WHERE id = ?4"
    )
    .bind(result).bind(slippage).bind(profit_cent).bind(id)
    .execute(pool).await?;
    Ok(())
}

pub async fn get_recent_trades(
    pool: &sqlx::SqlitePool,
    instrument: &str,
    limit: i64,
) -> Result<Vec<TradeRecord>> {
    let rows = sqlx::query_as::<_, (String, String, String, String, String, f64, f64, f64, f64, String, Option<String>, Option<f64>, Option<f64>, Option<f64>, String, Option<String>)>(
        "SELECT id, instrument, signal_tier, order_type, direction, entry_price, stop_loss, take_profit, volume, status, result, slippage, spread_open, profit_cent, opened_at, closed_at
         FROM trades WHERE instrument = ?1 ORDER BY opened_at DESC LIMIT ?2"
    )
    .bind(instrument).bind(limit)
    .fetch_all(pool).await?;

    Ok(rows
        .into_iter()
        .map(|r| TradeRecord {
            id: r.0,
            instrument: r.1,
            signal_tier: r.2,
            order_type: r.3,
            direction: r.4,
            entry_price: r.5,
            stop_loss: r.6,
            take_profit: r.7,
            volume: r.8,
            status: r.9,
            result: r.10,
            slippage: r.11,
            spread_open: r.12,
            profit_cent: r.13,
            opened_at: r.14,
            closed_at: r.15,
        })
        .collect())
}

#[cfg(test)]
mod tests {
    use shared::analytics::TradeRecord;

    #[test]
    fn test_new_trade_pending() {
        let t = TradeRecord::new("XAUUSD", "TIER_1", "BUY", 100.0, 99.0, 102.0, 0.5);
        assert_eq!(t.status, "PENDING");
        assert_eq!(t.instrument, "XAUUSD");
    }
}
