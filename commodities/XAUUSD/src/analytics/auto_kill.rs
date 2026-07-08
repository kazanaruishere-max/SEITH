// Auto-Kill Protection
// Delete all pending orders T+3 minutes post-news release

use anyhow::Result;

const AUTO_KILL_DELAY_MINUTES: i64 = 3;

pub fn is_auto_kill_time(
    entry_time: &chrono::DateTime<chrono::Utc>,
    now: &chrono::DateTime<chrono::Utc>,
) -> bool {
    let elapsed = (*now - *entry_time).num_minutes();
    elapsed >= AUTO_KILL_DELAY_MINUTES
}

pub async fn kill_pending_orders() -> Result<u32> {
    log::warn!(
        "Auto-Kill: deleting all pending orders (T+{} min)",
        AUTO_KILL_DELAY_MINUTES
    );
    todo!("Call MT5 OrderDeleteAllPending via bridge")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_not_yet_auto_kill() {
        let entry = chrono::Utc::now();
        let now = entry + chrono::Duration::minutes(1);
        assert!(!is_auto_kill_time(&entry, &now));
    }

    #[test]
    fn test_trigger_auto_kill() {
        let entry = chrono::Utc::now();
        let now = entry + chrono::Duration::minutes(5);
        assert!(is_auto_kill_time(&entry, &now));
    }
}
