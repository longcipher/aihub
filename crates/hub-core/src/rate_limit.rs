use std::{
    sync::atomic::{AtomicU64, Ordering},
    time::Instant,
};

use scc::HashMap;

struct Bucket {
    tokens: AtomicU64,
    max_tokens: u64,
    last_refill: std::sync::Mutex<Instant>,
}

impl Bucket {
    fn new(max_tokens: u64) -> Self {
        Self {
            tokens: AtomicU64::new(max_tokens),
            max_tokens,
            last_refill: std::sync::Mutex::new(Instant::now()),
        }
    }

    fn try_acquire(&self) -> bool {
        let mut last = self.last_refill.lock().unwrap();
        let now = Instant::now();
        let elapsed = now.duration_since(*last);
        if elapsed.as_secs() > 0 {
            let refill = elapsed.as_secs();
            let cur = self.tokens.load(Ordering::Relaxed);
            self.tokens.store((cur + refill).min(self.max_tokens), Ordering::Relaxed);
            *last = now;
        }
        let cur = self.tokens.load(Ordering::Relaxed);
        if cur > 0 {
            self.tokens.fetch_sub(1, Ordering::Relaxed);
            true
        } else {
            false
        }
    }
}

pub struct RateLimiter {
    buckets: HashMap<String, std::sync::Arc<Bucket>>,
}

impl RateLimiter {
    pub fn new() -> Self {
        Self { buckets: HashMap::new() }
    }

    pub fn check(&self, key: &str, rpm_limit: u32) -> bool {
        if let Some(bucket) = self.buckets.read_sync(key, |_, v| v.clone()) {
            return bucket.try_acquire();
        }
        let bucket = std::sync::Arc::new(Bucket::new(rpm_limit as u64));
        let allowed = bucket.try_acquire();
        let _ = self.buckets.insert_sync(key.to_string(), bucket);
        allowed
    }
}

impl Default for RateLimiter {
    fn default() -> Self {
        Self::new()
    }
}
