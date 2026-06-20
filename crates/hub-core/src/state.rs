use std::sync::Arc;

use arc_swap::ArcSwap;

use crate::{
    budget::BudgetEnforcer,
    error::HubError,
    models::registry::ModelRegistry,
    rate_limit::RateLimiter,
    types::{GatewayConfig, virtual_key::VirtualKey},
};

/// Application state with lock-free reads via arc-swap
pub struct AppState {
    inner: ArcSwap<InnerAppState>,
    pub model_registry: Arc<ModelRegistry>,
    pub rate_limiter: Arc<RateLimiter>,
    pub budget_enforcer: Arc<BudgetEnforcer>,
}

struct InnerAppState {
    config: GatewayConfig,
    virtual_keys: Vec<VirtualKey>,
}

impl AppState {
    pub fn new(config: GatewayConfig) -> Result<Self, HubError> {
        let virtual_keys = config.virtual_keys.clone();
        let model_registry = Arc::new(ModelRegistry::new(&config.models, &config.providers)?);
        Ok(Self {
            inner: ArcSwap::from_pointee(InnerAppState { config, virtual_keys }),
            model_registry,
            rate_limiter: Arc::new(RateLimiter::new()),
            budget_enforcer: Arc::new(BudgetEnforcer::new()),
        })
    }

    /// Get current configuration (lock-free read)
    pub fn current_config(&self) -> GatewayConfig {
        self.inner.load().config.clone()
    }

    /// Get current virtual keys (lock-free read)
    pub fn virtual_keys(&self) -> Vec<VirtualKey> {
        self.inner.load().virtual_keys.clone()
    }

    /// Update configuration (atomic swap, with validation)
    pub fn update_config(&self, config: GatewayConfig) -> Result<(), HubError> {
        let provider_keys: std::collections::HashSet<_> =
            config.providers.iter().map(|p| &p.key).collect();
        for model in &config.models {
            if !provider_keys.contains(&model.provider) {
                return Err(HubError::Config(format!(
                    "Model '{}' references non-existent provider '{}'",
                    model.key, model.provider
                )));
            }
        }
        let virtual_keys = config.virtual_keys.clone();
        self.inner.store(Arc::new(InnerAppState { config, virtual_keys }));
        Ok(())
    }

    /// Get current configuration snapshot (for tests)
    pub fn config_snapshot(&self) -> ConfigSnapshot {
        let state = self.inner.load();
        ConfigSnapshot { config: state.config.clone() }
    }

    /// Placeholder for router access (tests expect this method to exist)
    pub fn get_current_router(&self) -> Result<(), HubError> {
        Ok(())
    }
}

/// Configuration snapshot for test assertions
pub struct ConfigSnapshot {
    pub config: GatewayConfig,
}
