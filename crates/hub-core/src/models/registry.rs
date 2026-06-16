use std::sync::Arc;

use scc::HashMap;

use crate::{
    error::HubError,
    provider::LiterProvider,
    types::{ModelConfig, Provider},
};

/// Model instance with provider
pub struct ModelInstance {
    pub name: String,
    pub model_type: String,
    pub provider: Arc<LiterProvider>,
    pub provider_key: String,
}

/// Model registry using scc::HashMap for concurrent access
pub struct ModelRegistry {
    models: HashMap<String, Arc<ModelInstance>>,
}

impl ModelRegistry {
    pub fn new(model_configs: &[ModelConfig], providers: &[Provider]) -> Result<Self, HubError> {
        let models = HashMap::new();

        for config in model_configs {
            // Find the provider for this model
            let provider_config =
                providers.iter().find(|p| p.key == config.provider).ok_or_else(|| {
                    HubError::Config(format!(
                        "Provider '{}' not found for model '{}'",
                        config.provider, config.key
                    ))
                })?;

            // Build model hint for liter-llm (e.g., "openai/gpt-4o")
            let model_hint = format!("{}/{}", provider_config.r#type, config.r#type);

            // Create provider
            let provider = LiterProvider::new(
                &provider_config.api_key,
                provider_config.params.get("base_url").map(|s| s.as_str()),
                &model_hint,
            )?;

            let instance = ModelInstance {
                name: config.key.clone(),
                model_type: config.r#type.clone(),
                provider: Arc::new(provider),
                provider_key: config.provider.clone(),
            };

            let _ = models.insert_sync(config.key.clone(), Arc::new(instance));
        }

        Ok(Self { models })
    }

    pub fn get(&self, name: &str) -> Option<Arc<ModelInstance>> {
        self.models.read_sync(name, |_, v| v.clone())
    }

    pub fn list_models(&self) -> Vec<String> {
        let mut result = Vec::new();
        self.models.iter_sync(|key, _| {
            result.push(key.clone());
            true
        });
        result
    }
}
