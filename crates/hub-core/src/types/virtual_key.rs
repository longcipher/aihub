use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

/// Budget enforcement mode
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Hash, Default, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum BudgetMode {
    /// Reject requests when budget exceeded
    #[default]
    Hard,
    /// Allow requests but log warnings when budget exceeded
    Soft,
}

/// Virtual API key for gateway authentication
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Hash, ToSchema)]
pub struct VirtualKey {
    pub id: Uuid,
    pub key_hash: String,
    pub name: String,
    pub enabled: bool,

    /// Empty means all models allowed
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub allowed_models: Vec<String>,

    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub denied_models: Vec<String>,

    /// Requests per minute limit (None = unlimited)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub rpm_limit: Option<u32>,

    /// Tokens per minute limit (None = unlimited)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tpm_limit: Option<u32>,

    /// Monthly budget in cents (None = unlimited)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub monthly_budget_cents: Option<i64>,

    #[serde(default)]
    pub budget_mode: BudgetMode,

    /// Which configured provider key to use
    pub provider_key: String,

    #[serde(default = "Utc::now")]
    pub created_at: DateTime<Utc>,

    #[serde(default = "Utc::now")]
    pub updated_at: DateTime<Utc>,
}

impl VirtualKey {
    /// Check if a model is allowed for this key
    pub fn is_model_allowed(&self, model: &str) -> bool {
        // If denied_models contains the model, reject it
        if self.denied_models.iter().any(|m| m == model) {
            return false;
        }

        // If allowed_models is empty, all models are allowed
        if self.allowed_models.is_empty() {
            return true;
        }

        // Otherwise, model must be in allowed_models
        self.allowed_models.iter().any(|m| m == model)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_virtual_key(
        allowed_models: Vec<String>,
        denied_models: Vec<String>,
    ) -> VirtualKey {
        VirtualKey {
            id: Uuid::new_v4(),
            key_hash: "test-hash".to_string(),
            name: "Test Key".to_string(),
            enabled: true,
            allowed_models,
            denied_models,
            rpm_limit: None,
            tpm_limit: None,
            monthly_budget_cents: None,
            budget_mode: BudgetMode::Hard,
            provider_key: "openai".to_string(),
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
    }

    #[test]
    fn test_model_allowed_empty_allowlist() {
        let key = create_test_virtual_key(vec![], vec![]);
        assert!(key.is_model_allowed("gpt-4o"));
        assert!(key.is_model_allowed("claude-3-5-sonnet"));
    }

    #[test]
    fn test_model_allowed_specific_allowlist() {
        let key = create_test_virtual_key(vec!["gpt-4o".to_string()], vec![]);
        assert!(key.is_model_allowed("gpt-4o"));
        assert!(!key.is_model_allowed("claude-3-5-sonnet"));
    }

    #[test]
    fn test_model_denied() {
        let key = create_test_virtual_key(vec![], vec!["gpt-4o".to_string()]);
        assert!(!key.is_model_allowed("gpt-4o"));
        assert!(key.is_model_allowed("claude-3-5-sonnet"));
    }

    #[test]
    fn test_model_denied_overrides_allowlist() {
        let key = create_test_virtual_key(vec!["gpt-4o".to_string()], vec!["gpt-4o".to_string()]);
        assert!(!key.is_model_allowed("gpt-4o"));
    }
}
