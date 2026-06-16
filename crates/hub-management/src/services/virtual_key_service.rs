use hub_core::types::virtual_key::VirtualKey;
use uuid::Uuid;

use crate::db::repositories::virtual_key_repository::VirtualKeyRepository;

pub struct VirtualKeyService {
    repository: VirtualKeyRepository,
}

impl VirtualKeyService {
    pub fn new(repository: VirtualKeyRepository) -> Self {
        Self { repository }
    }

    pub async fn create_key(&self, key: &VirtualKey) -> Result<VirtualKey, String> {
        self.repository.create(key).await.map_err(|e| format!("Failed to create virtual key: {e}"))
    }

    pub async fn get_key(&self, id: Uuid) -> Result<Option<VirtualKey>, String> {
        self.repository.get_by_id(id).await.map_err(|e| format!("Failed to get virtual key: {e}"))
    }

    pub async fn list_keys(&self) -> Result<Vec<VirtualKey>, String> {
        self.repository.list().await.map_err(|e| format!("Failed to list virtual keys: {e}"))
    }

    pub async fn update_key(&self, key: &VirtualKey) -> Result<VirtualKey, String> {
        self.repository.update(key).await.map_err(|e| format!("Failed to update virtual key: {e}"))
    }

    pub async fn delete_key(&self, id: Uuid) -> Result<bool, String> {
        self.repository.delete(id).await.map_err(|e| format!("Failed to delete virtual key: {e}"))
    }

    pub async fn rotate_key(&self, id: Uuid) -> Result<VirtualKey, String> {
        let mut key = self
            .repository
            .get_by_id(id)
            .await
            .map_err(|e| format!("Failed to get virtual key: {e}"))?
            .ok_or_else(|| "Virtual key not found".to_string())?;

        // Generate new key hash (in production, generate a new actual key)
        let new_key = format!("hub-{}", uuid::Uuid::new_v4());
        key.key_hash = sha256_hash(&new_key);
        key.updated_at = chrono::Utc::now();

        self.repository.update(&key).await.map_err(|e| format!("Failed to rotate virtual key: {e}"))
    }
}

fn sha256_hash(input: &str) -> String {
    use sha2::Digest;
    let mut hasher = sha2::Sha256::new();
    hasher.update(input.as_bytes());
    let result = hasher.finalize();
    hex::encode(result)
}
