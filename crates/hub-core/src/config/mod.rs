pub mod hash;

use std::path::Path;

use crate::{error::HubError, types::GatewayConfig};

/// Load configuration from a YAML file with environment variable substitution
pub fn load_config(path: &Path) -> Result<GatewayConfig, HubError> {
    let content = std::fs::read_to_string(path)
        .map_err(|e| HubError::Config(format!("Failed to read config file: {e}")))?;

    let expanded = expand_env_vars(&content)?;

    let config: GatewayConfig = serde_yaml::from_str(&expanded)
        .map_err(|e| HubError::Config(format!("Failed to parse config: {e}")))?;

    Ok(config)
}

/// Expand `${VAR}` and `${VAR:-default}` environment variable references in a string
fn expand_env_vars(content: &str) -> Result<String, HubError> {
    let mut result = String::with_capacity(content.len());
    let mut chars = content.chars();
    while let Some(c) = chars.next() {
        if c == '$' {
            if let Some('{') = chars.next() {
                let mut var_name = String::new();
                let mut default_value = None;
                let mut found_close = false;
                while let Some(ch) = chars.next() {
                    if ch == '}' {
                        found_close = true;
                        break;
                    } else if ch == ':' {
                        // Read default value after :-
                        if chars.next() == Some('-') {
                            let mut def = String::new();
                            for d in chars.by_ref() {
                                if d == '}' {
                                    found_close = true;
                                    break;
                                }
                                def.push(d);
                            }
                            default_value = Some(def);
                        }
                        break;
                    } else {
                        var_name.push(ch);
                    }
                }
                if found_close || default_value.is_some() {
                    if let Ok(val) = std::env::var(&var_name) {
                        result.push_str(&val);
                    } else if let Some(def) = default_value {
                        result.push_str(&def);
                    } else {
                        return Err(HubError::Config(format!(
                            "Environment variable '{var_name}' not found"
                        )));
                    }
                } else {
                    result.push('$');
                    result.push('{');
                    result.push_str(&var_name);
                }
            } else {
                result.push(c);
            }
        } else {
            result.push(c);
        }
    }
    Ok(result)
}

/// Calculate a hash of the configuration for change detection
pub fn calculate_config_hash(config: &GatewayConfig) -> u64 {
    use std::{
        collections::hash_map::DefaultHasher,
        hash::{Hash, Hasher},
    };

    let mut hasher = DefaultHasher::new();
    config.hash(&mut hasher);
    hasher.finish()
}

pub mod validation {
    use crate::types::GatewayConfig;

    pub fn validate_gateway_config(config: &GatewayConfig) -> Result<Vec<String>, Vec<String>> {
        let mut errors = Vec::new();

        if config.providers.is_empty() {
            errors.push("At least one provider is required".into());
        }
        if config.models.is_empty() {
            errors.push("At least one model is required".into());
        }
        if config.pipelines.is_empty() {
            errors.push("At least one pipeline is required".into());
        }

        let provider_keys: std::collections::HashSet<_> =
            config.providers.iter().map(|p| &p.key).collect();
        for model in &config.models {
            if !provider_keys.contains(&model.provider) {
                errors.push(format!(
                    "Model '{}' references non-existent provider '{}'",
                    model.key, model.provider
                ));
            }
        }

        if errors.is_empty() { Ok(errors) } else { Err(errors) }
    }
}

/// Compare two configurations for equality (ignoring order-dependent fields)
pub fn configs_are_equal(a: &GatewayConfig, b: &GatewayConfig) -> bool {
    a == b
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use super::*;
    use crate::types::*;

    fn create_test_config() -> GatewayConfig {
        GatewayConfig {
            general: Some(General { trace_content_enabled: true }),
            providers: vec![Provider {
                key: "openai".to_string(),
                r#type: ProviderType::OpenAI,
                api_key: "sk-test".to_string(),
                params: HashMap::new(),
            }],
            models: vec![ModelConfig {
                key: "gpt-4o".to_string(),
                r#type: "gpt-4o".to_string(),
                provider: "openai".to_string(),
                params: HashMap::new(),
            }],
            pipelines: vec![Pipeline {
                name: "default".to_string(),
                r#type: PipelineType::Chat,
                plugins: vec![PluginConfig::ModelRouter { models: vec!["gpt-4o".to_string()] }],
            }],
            virtual_keys: vec![],
        }
    }

    #[test]
    fn test_config_roundtrip_yaml() {
        let config = create_test_config();
        let yaml = serde_yaml::to_string(&config).expect("serialize");
        let deserialized: GatewayConfig = serde_yaml::from_str(&yaml).expect("deserialize");
        assert_eq!(config, deserialized);
    }

    #[test]
    fn test_config_roundtrip_json() {
        let config = create_test_config();
        let json = serde_json::to_string(&config).expect("serialize");
        let deserialized: GatewayConfig = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(config, deserialized);
    }

    #[test]
    fn test_config_hash_deterministic() {
        let config = create_test_config();
        let hash1 = calculate_config_hash(&config);
        let hash2 = calculate_config_hash(&config);
        assert_eq!(hash1, hash2);
    }

    #[test]
    fn test_config_hash_different_configs() {
        let config1 = create_test_config();
        let mut config2 = create_test_config();
        config2.models.push(ModelConfig {
            key: "claude-3".to_string(),
            r#type: "claude-3".to_string(),
            provider: "anthropic".to_string(),
            params: HashMap::new(),
        });
        assert_ne!(calculate_config_hash(&config1), calculate_config_hash(&config2));
    }

    #[test]
    fn test_load_config_from_yaml() {
        // Build a config, serialize it, then deserialize to verify the format
        let config = create_test_config();
        let yaml = serde_yaml::to_string(&config).expect("serialize");
        let deserialized: GatewayConfig = serde_yaml::from_str(&yaml).expect("deserialize");
        assert_eq!(config, deserialized);
    }

    #[test]
    fn test_config_with_virtual_keys() {
        use chrono::Utc;
        use uuid::Uuid;

        use crate::types::virtual_key::*;

        let config = GatewayConfig {
            general: None,
            providers: vec![],
            models: vec![],
            pipelines: vec![],
            virtual_keys: vec![VirtualKey {
                id: Uuid::new_v4(),
                key_hash: "test-hash".to_string(),
                name: "Test".to_string(),
                enabled: true,
                allowed_models: vec!["gpt-4o".to_string()],
                denied_models: vec![],
                rpm_limit: Some(60),
                tpm_limit: None,
                monthly_budget_cents: Some(5000),
                budget_mode: BudgetMode::Hard,
                provider_key: "openai".to_string(),
                created_at: Utc::now(),
                updated_at: Utc::now(),
            }],
        };

        let yaml = serde_yaml::to_string(&config).expect("serialize");
        let deserialized: GatewayConfig = serde_yaml::from_str(&yaml).expect("deserialize");
        assert_eq!(deserialized.virtual_keys.len(), 1);
        assert_eq!(deserialized.virtual_keys[0].name, "Test");
    }
}
