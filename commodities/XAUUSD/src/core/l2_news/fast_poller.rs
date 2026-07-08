// L2 - Fast Poller
// Async milisecond polling at T-1 minute before news release

use anyhow::Result;
use std::time::Duration;
use tokio::time::sleep;

pub struct FastPoller {
    poll_interval_ms: u64,
}

impl Default for FastPoller {
    fn default() -> Self {
        Self::new()
    }
}

impl FastPoller {
    pub fn new() -> Self {
        Self {
            poll_interval_ms: 100,
        }
    }

    pub async fn poll<F, T>(&self, mut fetch: F) -> Result<T>
    where
        F: FnMut() -> Result<T>,
    {
        loop {
            match fetch() {
                Ok(data) => return Ok(data),
                Err(e) => {
                    log::debug!("Fast poll attempt failed: {}", e);
                    sleep(Duration::from_millis(self.poll_interval_ms)).await;
                }
            }
        }
    }

    pub async fn poll_with_timeout<F, T>(
        &self,
        mut fetch: F,
        timeout_secs: u64,
    ) -> Result<Option<T>>
    where
        F: FnMut() -> Result<T>,
    {
        let deadline = tokio::time::Instant::now() + Duration::from_secs(timeout_secs);
        while tokio::time::Instant::now() < deadline {
            match fetch() {
                Ok(data) => return Ok(Some(data)),
                Err(e) => {
                    log::debug!("Fast poll: {}", e);
                    sleep(Duration::from_millis(self.poll_interval_ms)).await;
                }
            }
        }
        log::warn!("Fast poll timed out after {}s", timeout_secs);
        Ok(None)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_poll_immediate() {
        let poller = FastPoller::new();
        let result = poll_immediate_helper(poller);
        assert!(result.is_ok());
    }

    fn poll_immediate_helper(poller: FastPoller) -> Result<i32> {
        // Use tokio runtime for async test
        tokio::runtime::Runtime::new()
            .unwrap()
            .block_on(async { poller.poll(|| Ok(42)).await })
    }

    #[test]
    fn test_poll_interval_default() {
        let poller = FastPoller::new();
        assert_eq!(poller.poll_interval_ms, 100);
    }
}
