use std::future::Future;
use std::time::Duration;

use crate::api::spotify::SpotifyError;

// ---------------------------------------------------------------------------
// Config
// ---------------------------------------------------------------------------

pub struct RetryConfig {
    pub max_retries: u32,
    pub base_delay_ms: u64,
    pub max_delay_ms: u64,
}

impl Default for RetryConfig {
    fn default() -> Self {
        Self {
            max_retries: 3,
            base_delay_ms: 1000,
            max_delay_ms: 10_000,
        }
    }
}

// ---------------------------------------------------------------------------
// Retry logic
// ---------------------------------------------------------------------------

/// Retry an async operation with exponential backoff.
///
/// - 429 (rate-limited): waits `retry_after_secs`, retries up to `max_retries`.
/// - 500–503 / network errors: exponential backoff up to `max_retries`.
/// - All other errors: returned immediately.
pub async fn retry_with_backoff<F, Fut, T>(
    config: &RetryConfig,
    mut operation: F,
) -> Result<T, SpotifyError>
where
    F: FnMut() -> Fut,
    Fut: Future<Output = Result<T, SpotifyError>>,
{
    let mut attempt = 0u32;

    loop {
        match operation().await {
            Ok(val) => return Ok(val),
            Err(e) if attempt >= config.max_retries => return Err(e),
            Err(SpotifyError::RateLimited { retry_after_secs }) => {
                attempt += 1;
                tokio::time::sleep(Duration::from_secs(retry_after_secs)).await;
            }
            Err(SpotifyError::Api { status, .. }) if (500..=503).contains(&status) => {
                attempt += 1;
                let delay = backoff_delay(config, attempt);
                tokio::time::sleep(delay).await;
            }
            Err(SpotifyError::Http(_)) => {
                attempt += 1;
                let delay = backoff_delay(config, attempt);
                tokio::time::sleep(delay).await;
            }
            Err(e) => return Err(e),
        }
    }
}

fn backoff_delay(config: &RetryConfig, attempt: u32) -> Duration {
    let delay_ms = config
        .base_delay_ms
        .saturating_mul(2u64.saturating_pow(attempt.saturating_sub(1)));
    let capped = delay_ms.min(config.max_delay_ms);
    Duration::from_millis(capped)
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicU32, Ordering};
    use std::sync::Arc;

    /// Helper: fast retry config so tests don't wait.
    fn test_config() -> RetryConfig {
        RetryConfig {
            max_retries: 3,
            base_delay_ms: 10,
            max_delay_ms: 100,
        }
    }

    #[tokio::test]
    async fn test_succeeds_first_try() {
        let calls = Arc::new(AtomicU32::new(0));
        let calls_clone = calls.clone();

        let result = retry_with_backoff(&test_config(), || {
            let c = calls_clone.clone();
            async move {
                c.fetch_add(1, Ordering::SeqCst);
                Ok::<_, SpotifyError>(42)
            }
        })
        .await;

        assert_eq!(result.unwrap(), 42);
        assert_eq!(calls.load(Ordering::SeqCst), 1);
    }

    #[tokio::test]
    async fn test_fails_then_succeeds() {
        let calls = Arc::new(AtomicU32::new(0));
        let calls_clone = calls.clone();

        let result = retry_with_backoff(&test_config(), || {
            let c = calls_clone.clone();
            async move {
                let n = c.fetch_add(1, Ordering::SeqCst);
                if n == 0 {
                    Err(SpotifyError::Api {
                        status: 500,
                        message: "server error".to_string(),
                    })
                } else {
                    Ok::<_, SpotifyError>("ok")
                }
            }
        })
        .await;

        assert_eq!(result.unwrap(), "ok");
        assert_eq!(calls.load(Ordering::SeqCst), 2);
    }

    #[tokio::test]
    async fn test_rate_limited_respects_retry_after() {
        tokio::time::pause();

        let calls = Arc::new(AtomicU32::new(0));
        let calls_clone = calls.clone();

        let config = RetryConfig {
            max_retries: 3,
            base_delay_ms: 10,
            max_delay_ms: 100,
        };

        let result = retry_with_backoff(&config, || {
            let c = calls_clone.clone();
            async move {
                let n = c.fetch_add(1, Ordering::SeqCst);
                if n == 0 {
                    Err(SpotifyError::RateLimited {
                        retry_after_secs: 5,
                    })
                } else {
                    Ok::<_, SpotifyError>("done")
                }
            }
        })
        .await;

        assert_eq!(result.unwrap(), "done");
        assert_eq!(calls.load(Ordering::SeqCst), 2);
    }

    #[tokio::test]
    async fn test_rate_limited_gives_up_after_max_retries() {
        tokio::time::pause();

        let calls = Arc::new(AtomicU32::new(0));
        let calls_clone = calls.clone();

        let config = RetryConfig {
            max_retries: 3,
            base_delay_ms: 10,
            max_delay_ms: 100,
        };

        let result = retry_with_backoff(&config, || {
            let c = calls_clone.clone();
            async move {
                c.fetch_add(1, Ordering::SeqCst);
                Err::<(), _>(SpotifyError::RateLimited {
                    retry_after_secs: 1,
                })
            }
        })
        .await;

        assert!(result.is_err());
        // 1 initial + 3 retries = 4 calls
        assert_eq!(calls.load(Ordering::SeqCst), 4);
        assert!(matches!(
            result.unwrap_err(),
            SpotifyError::RateLimited { .. }
        ));
    }

    #[tokio::test]
    async fn test_server_error_exponential_backoff() {
        tokio::time::pause();

        let calls = Arc::new(AtomicU32::new(0));
        let calls_clone = calls.clone();

        let config = RetryConfig {
            max_retries: 2,
            base_delay_ms: 100,
            max_delay_ms: 5000,
        };

        let result = retry_with_backoff(&config, || {
            let c = calls_clone.clone();
            async move {
                c.fetch_add(1, Ordering::SeqCst);
                Err::<(), _>(SpotifyError::Api {
                    status: 503,
                    message: "unavailable".to_string(),
                })
            }
        })
        .await;

        assert!(result.is_err());
        // 1 initial + 2 retries = 3 calls
        assert_eq!(calls.load(Ordering::SeqCst), 3);
    }

    #[tokio::test]
    async fn test_non_retryable_error_returns_immediately() {
        let calls = Arc::new(AtomicU32::new(0));
        let calls_clone = calls.clone();

        let result = retry_with_backoff(&test_config(), || {
            let c = calls_clone.clone();
            async move {
                c.fetch_add(1, Ordering::SeqCst);
                Err::<(), _>(SpotifyError::NotFound("gone".to_string()))
            }
        })
        .await;

        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), SpotifyError::NotFound(_)));
        assert_eq!(calls.load(Ordering::SeqCst), 1);
    }
}
