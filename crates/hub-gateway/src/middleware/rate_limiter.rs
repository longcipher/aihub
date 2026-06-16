use std::{
    sync::{
        Arc,
        atomic::{AtomicU32, Ordering},
    },
    time::Instant,
};

use scc::HashMap;

/// Token bucket for rate limiting
struct TokenBucket {
    tokens: AtomicU32,
    max_tokens: u32,
    last_refill: parking_lot::Mutex<Instant>,
}

impl TokenBucket {
    fn new(max_tokens: u32) -> Self {
        Self {
            tokens: AtomicU32::new(max_tokens),
            max_tokens,
            last_refill: parking_lot::Mutex::new(Instant::now()),
        }
    }

    fn try_acquire(&self) -> (bool, u32) {
        let mut last_refill = self.last_refill.lock();
        let now = Instant::now();
        let elapsed = now.duration_since(*last_refill);

        // Refill tokens based on elapsed time (1 token per second for RPM)
        if elapsed.as_secs() > 0 {
            let refill = elapsed.as_secs() as u32;
            let current = self.tokens.load(Ordering::Relaxed);
            let new_tokens = (current + refill).min(self.max_tokens);
            self.tokens.store(new_tokens, Ordering::Relaxed);
            *last_refill = now;
        }

        // Try to acquire a token
        let current = self.tokens.load(Ordering::Relaxed);
        if current > 0 {
            self.tokens.fetch_sub(1, Ordering::Relaxed);
            (true, current - 1)
        } else {
            (false, 0)
        }
    }

    fn remaining(&self) -> u32 {
        self.tokens.load(Ordering::Relaxed)
    }
}

/// Rate limiter using token bucket algorithm
pub struct RateLimiter {
    buckets: HashMap<String, Arc<TokenBucket>>,
}

impl RateLimiter {
    pub fn new() -> Self {
        Self { buckets: HashMap::new() }
    }

    /// Check if a request is allowed for the given key.
    /// Returns (allowed, remaining_tokens)
    pub fn check_rate_limit(&self, key: &str, rpm_limit: u32) -> (bool, u32) {
        // Try to get existing bucket
        let existing = self.buckets.read_sync(key, |_, v| v.clone());
        if let Some(bucket) = existing {
            return bucket.try_acquire();
        }

        // Create new bucket
        let bucket = Arc::new(TokenBucket::new(rpm_limit));
        let result = bucket.try_acquire();
        let remaining = bucket.remaining();

        // Insert bucket (ignore if another thread inserted first)
        let _ = self.buckets.insert_sync(key.to_string(), bucket);

        (result.0, remaining)
    }

    /// Get remaining tokens for a key (without acquiring)
    pub fn remaining(&self, key: &str) -> Option<u32> {
        self.buckets.read_sync(key, |_, v| v.remaining())
    }
}

impl Default for RateLimiter {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rate_limiter_allows_within_limit() {
        let limiter = RateLimiter::new();
        let (allowed, _) = limiter.check_rate_limit("key1", 5);
        assert!(allowed);
    }

    #[test]
    fn test_rate_limiter_blocks_over_limit() {
        let limiter = RateLimiter::new();
        // Exhaust the bucket
        for _ in 0..5 {
            limiter.check_rate_limit("key1", 5);
        }
        let (allowed, remaining) = limiter.check_rate_limit("key1", 5);
        assert!(!allowed);
        assert_eq!(remaining, 0);
    }

    #[test]
    fn test_rate_limiter_separate_keys() {
        let limiter = RateLimiter::new();
        // Exhaust key1
        for _ in 0..5 {
            limiter.check_rate_limit("key1", 5);
        }
        // key2 should still be allowed
        let (allowed, _) = limiter.check_rate_limit("key2", 5);
        assert!(allowed);
    }

    #[test]
    fn test_rate_limiter_remaining() {
        let limiter = RateLimiter::new();
        limiter.check_rate_limit("key1", 10);
        let remaining = limiter.remaining("key1");
        assert_eq!(remaining, Some(9));
    }

    #[test]
    fn test_rate_limiter_unknown_key() {
        let limiter = RateLimiter::new();
        assert_eq!(limiter.remaining("unknown"), None);
    }
}
