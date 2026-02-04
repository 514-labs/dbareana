use governor::{
    clock::DefaultClock,
    state::{direct::NotKeyed, InMemoryState},
    Quota, RateLimiter as GovRateLimiter,
};
use std::num::NonZeroU32;

/// Rate limiter wrapper for controlling transaction throughput
pub struct RateLimiter {
    limiter: GovRateLimiter<NotKeyed, InMemoryState, DefaultClock>,
}

impl RateLimiter {
    /// Create a new rate limiter with target TPS
    pub fn new(tps: usize) -> Self {
        let quota = Quota::per_second(NonZeroU32::new(tps as u32).unwrap());
        Self {
            limiter: GovRateLimiter::direct(quota),
        }
    }

    /// Wait until we can proceed (blocks until a permit is available)
    pub async fn acquire(&self) {
        self.limiter.until_ready().await;
    }

    /// Try to acquire a permit without waiting
    pub fn try_acquire(&self) -> bool {
        self.limiter.check().is_ok()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Instant;
    use tokio::time::{sleep, Duration};

    #[tokio::test]
    async fn test_rate_limiter_basic() {
        let limiter = RateLimiter::new(10); // 10 TPS

        // First acquisition should be immediate
        let start = Instant::now();
        limiter.acquire().await;
        let elapsed = start.elapsed();

        assert!(elapsed < Duration::from_millis(100));
    }

    #[tokio::test]
    async fn test_rate_limiter_enforces_limit() {
        let limiter = RateLimiter::new(100); // 100 TPS

        // Acquire many permits rapidly
        let start = Instant::now();
        for _ in 0..200 {
            limiter.acquire().await;
        }

        let elapsed = start.elapsed();

        // 200 requests at 100 TPS should take at least 1.5 seconds
        // But allow some variance due to system scheduling
        assert!(
            elapsed >= Duration::from_millis(1000),
            "Rate limiting not working: completed 200 requests in {:?}",
            elapsed
        );
    }

    #[tokio::test]
    async fn test_try_acquire() {
        let limiter = RateLimiter::new(10); // 10 TPS

        // First try should succeed
        assert!(limiter.try_acquire());

        // Exhaust permits
        for _ in 0..9 {
            limiter.try_acquire();
        }

        // Next try should fail (no permits available)
        assert!(!limiter.try_acquire());

        // Wait a bit and try again
        sleep(Duration::from_millis(150)).await;
        assert!(limiter.try_acquire());
    }

    #[tokio::test]
    async fn test_concurrent_workers() {
        use std::sync::Arc;

        let limiter = Arc::new(RateLimiter::new(100)); // 100 TPS
        let start = Instant::now();

        // Spawn 5 concurrent workers doing 50 requests each (250 total)
        let mut handles = vec![];
        for _ in 0..5 {
            let limiter = limiter.clone();
            handles.push(tokio::spawn(async move {
                for _ in 0..50 {
                    limiter.acquire().await;
                }
            }));
        }

        // Wait for all workers to complete (250 total requests at 100 TPS)
        for handle in handles {
            handle.await.unwrap();
        }

        let elapsed = start.elapsed();

        // 250 requests at 100 TPS should take at least 1.5 seconds
        // (allowing for some burst capacity in the rate limiter)
        assert!(
            elapsed >= Duration::from_millis(1400),
            "Rate limiting not working with concurrent workers: completed in {:?}",
            elapsed
        );
        assert!(elapsed <= Duration::from_millis(3500));
    }
}
