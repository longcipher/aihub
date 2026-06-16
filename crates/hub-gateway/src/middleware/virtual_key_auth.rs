use std::sync::Arc;

use axum::{
    extract::Request,
    http::StatusCode,
    middleware::Next,
    response::{IntoResponse, Response},
};
use hub_core::{state::AppState, types::virtual_key::VirtualKey};

/// Resolved auth info injected into request extensions
#[derive(Debug, Clone)]
pub struct ResolvedAuth {
    pub provider_api_key: String,
    pub virtual_key: Option<VirtualKey>,
    pub is_byok: bool,
}

/// Virtual key authentication middleware.
///
/// Auth flow:
/// - `Authorization: Bearer hub-<key>` → virtual key lookup + validation
/// - `Authorization: Bearer sk-<key>` (or any non-hub prefix) → BYOK passthrough
/// - No Authorization header → use default provider key from config
pub async fn virtual_key_auth(
    axum::extract::State(state): axum::extract::State<Arc<AppState>>,
    mut request: Request,
    next: Next,
) -> Response {
    let auth_header =
        request.headers().get("Authorization").and_then(|v| v.to_str().ok()).map(|s| s.to_string());

    let resolved = if let Some(auth) = auth_header {
        if !auth.starts_with("Bearer ") {
            return (StatusCode::UNAUTHORIZED, "Invalid Authorization header format")
                .into_response();
        }

        let token = auth.trim_start_matches("Bearer ").trim();

        if token.is_empty() {
            return (StatusCode::UNAUTHORIZED, "Empty bearer token").into_response();
        }

        if token.starts_with("hub-") {
            // Virtual key path
            resolve_virtual_key(token, &state)
        } else {
            // BYOK passthrough path
            ResolvedAuth { provider_api_key: token.to_string(), virtual_key: None, is_byok: true }
        }
    } else {
        // No auth header — fallback to default provider key
        let config = state.current_config();
        let default_key = config.providers.first().map(|p| p.api_key.clone()).unwrap_or_default();

        ResolvedAuth { provider_api_key: default_key, virtual_key: None, is_byok: false }
    };

    // Model allowlist/denylist check is deferred to the handler level
    // since consuming the body here would prevent the handler from reading it

    // Inject resolved auth into request extensions
    request.extensions_mut().insert(resolved);
    next.run(request).await
}

/// Resolve a virtual key token against the stored virtual keys
fn resolve_virtual_key(token: &str, state: &AppState) -> ResolvedAuth {
    let virtual_keys = state.virtual_keys();

    // Look up virtual key by matching the full token against stored keys
    // In production, we'd hash the token and compare against key_hash
    // For now, we match by the token string directly against a known pattern
    for vk in &virtual_keys {
        if !vk.enabled {
            continue;
        }
        // Match by key_hash or by a simple token match
        // The key_hash is SHA-256 of the full key, stored at creation time
        // We compare the hash of the incoming token against stored hashes
        let token_hash = hash_key(token);
        if vk.key_hash == token_hash {
            return ResolvedAuth {
                provider_api_key: vk.provider_key.clone(),
                virtual_key: Some(vk.clone()),
                is_byok: false,
            };
        }
    }

    // Key not found — return a sentinel that the handler can check
    ResolvedAuth { provider_api_key: String::new(), virtual_key: None, is_byok: false }
}

/// SHA-256 hash of a key for comparison
fn hash_key(key: &str) -> String {
    use sha2::Digest;
    let mut hasher = sha2::Sha256::new();
    hasher.update(key.as_bytes());
    let result = hasher.finalize();
    hex::encode(result)
}

/// Check if a request is authorized (has valid virtual key or BYOK)
pub fn is_authorized(resolved: &ResolvedAuth) -> bool {
    if resolved.is_byok {
        return true;
    }
    if resolved.virtual_key.is_some() {
        return !resolved.provider_api_key.is_empty();
    }
    // No virtual key and no BYOK — check if default key exists
    !resolved.provider_api_key.is_empty()
}

/// Check if a model is allowed for the resolved auth
pub fn is_model_allowed(resolved: &ResolvedAuth, model: &str) -> bool {
    if let Some(ref vk) = resolved.virtual_key {
        return vk.is_model_allowed(model);
    }
    // BYOK or default — all models allowed
    true
}

#[cfg(test)]
mod tests {
    use chrono::Utc;
    use hub_core::types::virtual_key::{BudgetMode, VirtualKey};
    use uuid::Uuid;

    use super::*;

    fn create_test_virtual_key(
        key_hash: &str,
        enabled: bool,
        allowed_models: Vec<String>,
        denied_models: Vec<String>,
    ) -> VirtualKey {
        VirtualKey {
            id: Uuid::new_v4(),
            key_hash: key_hash.to_string(),
            name: "Test Key".to_string(),
            enabled,
            allowed_models,
            denied_models,
            rpm_limit: None,
            tpm_limit: None,
            monthly_budget_cents: None,
            budget_mode: BudgetMode::Hard,
            provider_key: "test-provider-key".to_string(),
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
    }

    #[test]
    fn test_hash_key_deterministic() {
        let hash1 = hash_key("hub-test-key");
        let hash2 = hash_key("hub-test-key");
        assert_eq!(hash1, hash2);
    }

    #[test]
    fn test_hash_key_different_inputs() {
        let hash1 = hash_key("hub-key-1");
        let hash2 = hash_key("hub-key-2");
        assert_ne!(hash1, hash2);
    }

    #[test]
    fn test_is_authorized_byok() {
        let resolved = ResolvedAuth {
            provider_api_key: "sk-my-key".to_string(),
            virtual_key: None,
            is_byok: true,
        };
        assert!(is_authorized(&resolved));
    }

    #[test]
    fn test_is_authorized_virtual_key() {
        let resolved = ResolvedAuth {
            provider_api_key: "provider-key".to_string(),
            virtual_key: Some(create_test_virtual_key("hash", true, vec![], vec![])),
            is_byok: false,
        };
        assert!(is_authorized(&resolved));
    }

    #[test]
    fn test_is_authorized_empty_key() {
        let resolved =
            ResolvedAuth { provider_api_key: String::new(), virtual_key: None, is_byok: false };
        assert!(!is_authorized(&resolved));
    }

    #[test]
    fn test_is_model_allowed_no_virtual_key() {
        let resolved = ResolvedAuth {
            provider_api_key: "sk-key".to_string(),
            virtual_key: None,
            is_byok: true,
        };
        assert!(is_model_allowed(&resolved, "gpt-4o"));
        assert!(is_model_allowed(&resolved, "claude-3"));
    }

    #[test]
    fn test_is_model_allowed_with_virtual_key() {
        let vk = create_test_virtual_key("hash", true, vec!["gpt-4o".to_string()], vec![]);
        let resolved = ResolvedAuth {
            provider_api_key: "key".to_string(),
            virtual_key: Some(vk),
            is_byok: false,
        };
        assert!(is_model_allowed(&resolved, "gpt-4o"));
        assert!(!is_model_allowed(&resolved, "claude-3"));
    }

    #[test]
    fn test_is_model_allowed_denied_model() {
        let vk = create_test_virtual_key("hash", true, vec![], vec!["gpt-4o".to_string()]);
        let resolved = ResolvedAuth {
            provider_api_key: "key".to_string(),
            virtual_key: Some(vk),
            is_byok: false,
        };
        assert!(!is_model_allowed(&resolved, "gpt-4o"));
        assert!(is_model_allowed(&resolved, "claude-3"));
    }
}
