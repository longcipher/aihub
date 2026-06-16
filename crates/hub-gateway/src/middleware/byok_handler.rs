use std::sync::Arc;

use hub_core::provider::LiterProvider;
use scc::HashMap;

/// BYOK client cache with TTL
struct CachedClient {
    provider: Arc<LiterProvider>,
    created_at: std::time::Instant,
}

/// BYOK passthrough middleware
pub struct ByokHandler {
    client_cache: HashMap<String, CachedClient>,
    ttl: std::time::Duration,
}

impl ByokHandler {
    pub fn new(ttl: std::time::Duration) -> Self {
        Self { client_cache: HashMap::new(), ttl }
    }

    /// Get or create a LiterProvider for a BYOK key
    pub fn get_or_create_provider(
        &self,
        api_key: &str,
        model: &str,
    ) -> Result<Arc<LiterProvider>, String> {
        // Check cache first
        let cached = self.client_cache.read_sync(api_key, |_, v| {
            if v.created_at.elapsed() < self.ttl { Some(v.provider.clone()) } else { None }
        });

        if let Some(Some(provider)) = cached {
            return Ok(provider);
        }

        // Create new provider
        let provider = LiterProvider::new(api_key, None, model)
            .map_err(|e| format!("Failed to create BYOK provider: {e}"))?;
        let provider = Arc::new(provider);

        // Cache it
        let cached =
            CachedClient { provider: provider.clone(), created_at: std::time::Instant::now() };
        let _ = self.client_cache.insert_sync(api_key.to_string(), cached);

        Ok(provider)
    }

    /// Clean up expired entries
    pub fn cleanup_expired(&self) {
        let now = std::time::Instant::now();
        let mut expired_keys = Vec::new();

        self.client_cache.iter_sync(|key, value| {
            if now.duration_since(value.created_at) >= self.ttl {
                expired_keys.push(key.clone());
            }
            true // Continue iteration
        });

        for key in expired_keys {
            let _ = self.client_cache.remove_sync(&key);
        }
    }
}

impl Default for ByokHandler {
    fn default() -> Self {
        Self::new(std::time::Duration::from_secs(300)) // 5 minute TTL
    }
}
