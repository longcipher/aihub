use std::time::Duration;

use liter_llm::{ClientConfig, DefaultClient, LlmClient};

use crate::error::HubError;

/// Adapter wrapping liter-llm's DefaultClient
pub struct LiterProvider {
    client: DefaultClient,
    model_hint: String,
}

impl LiterProvider {
    /// Create a new LiterProvider
    pub fn new(api_key: &str, base_url: Option<&str>, model_hint: &str) -> Result<Self, HubError> {
        let mut config = ClientConfig::new(api_key);
        if let Some(url) = base_url {
            config.base_url = Some(url.to_string());
        }
        config.timeout = Duration::from_secs(120);

        let client = DefaultClient::new(config, Some(model_hint))
            .map_err(|e| HubError::ProviderInit(e.to_string()))?;

        Ok(Self { client, model_hint: model_hint.to_string() })
    }

    /// Get a reference to the underlying client
    pub fn client(&self) -> &impl LlmClient {
        &self.client
    }

    /// Get the model hint
    pub fn model_hint(&self) -> &str {
        &self.model_hint
    }
}
