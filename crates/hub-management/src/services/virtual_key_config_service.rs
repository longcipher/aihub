use std::sync::Arc;

use hub_core::types::virtual_key::VirtualKey;

use crate::db::repositories::virtual_key_repository::VirtualKeyRepository;

/// Service for managing virtual keys in the config provider
pub struct VirtualKeyConfigService {
    repository: Arc<VirtualKeyRepository>,
}

impl VirtualKeyConfigService {
    pub fn new(repository: Arc<VirtualKeyRepository>) -> Self {
        Self { repository }
    }

    /// Fetch all virtual keys from the database
    pub async fn fetch_virtual_keys(&self) -> Result<Vec<VirtualKey>, String> {
        self.repository.list().await.map_err(|e| format!("Failed to fetch virtual keys: {e}"))
    }

    /// Get a virtual key by its hash
    pub async fn get_by_hash(&self, key_hash: &str) -> Result<Option<VirtualKey>, String> {
        self.repository
            .get_by_hash(key_hash)
            .await
            .map_err(|e| format!("Failed to get virtual key: {e}"))
    }
}
