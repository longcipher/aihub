use std::sync::Arc;

use hub_core::{
    state::AppState,
    types::{
        GatewayConfig, ModelConfig, Pipeline, PipelineType, PluginConfig, Provider, ProviderType,
    },
};

fn create_test_config_with_multiple_pipelines() -> GatewayConfig {
    let provider = Provider {
        key: "test-provider".to_string(),
        r#type: ProviderType::OpenAI,
        api_key: "test-key".to_string(),
        params: Default::default(),
    };

    let model = ModelConfig {
        key: "test-model".to_string(),
        r#type: "gpt-4".to_string(),
        provider: "test-provider".to_string(),
        params: Default::default(),
    };

    let pipeline1 = Pipeline {
        name: "default".to_string(),
        r#type: PipelineType::Chat,
        plugins: vec![PluginConfig::ModelRouter { models: vec!["test-model".to_string()] }],
    };

    let pipeline2 = Pipeline {
        name: "special".to_string(),
        r#type: PipelineType::Chat,
        plugins: vec![PluginConfig::ModelRouter { models: vec!["test-model".to_string()] }],
    };

    GatewayConfig {
        general: None,
        providers: vec![provider],
        models: vec![model],
        pipelines: vec![pipeline1, pipeline2],
        virtual_keys: vec![],
    }
}

#[tokio::test]
async fn test_pipeline_header_routing_multiple_pipelines_exist() {
    let config = create_test_config_with_multiple_pipelines();
    let app_state = Arc::new(AppState::new(config).unwrap());

    let _router = app_state.get_current_router();

    let current_config = app_state.current_config();
    assert_eq!(current_config.pipelines.len(), 2);
    assert_eq!(current_config.pipelines[0].name, "default");
    assert_eq!(current_config.pipelines[1].name, "special");
}

#[tokio::test]
async fn test_pipeline_header_routing_configuration_updates() {
    let config = create_test_config_with_multiple_pipelines();
    let app_state = Arc::new(AppState::new(config).unwrap());

    let current_config = app_state.current_config();
    assert_eq!(current_config.pipelines.len(), 2);

    let mut updated_config = current_config.clone();
    let pipeline3 = Pipeline {
        name: "third".to_string(),
        r#type: PipelineType::Chat,
        plugins: vec![PluginConfig::ModelRouter { models: vec!["test-model".to_string()] }],
    };
    updated_config.pipelines.push(pipeline3);

    let result = app_state.update_config(updated_config);
    assert!(result.is_ok(), "Config update should succeed");

    let updated_current_config = app_state.current_config();
    assert_eq!(updated_current_config.pipelines.len(), 3);
    assert_eq!(updated_current_config.pipelines[2].name, "third");
}
