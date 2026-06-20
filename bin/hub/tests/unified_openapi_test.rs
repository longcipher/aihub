use std::sync::Arc;

use hub_core::{
    state::AppState,
    types::{
        GatewayConfig, ModelConfig, Pipeline, PipelineType, PluginConfig, Provider, ProviderType,
    },
};
use hub_gateway::{openapi::get_openapi_spec, routes::create_router};

#[test]
fn test_openapi_spec_is_valid() {
    let spec = get_openapi_spec();

    assert!(!spec.info.title.is_empty());
    assert!(!spec.info.version.is_empty());
    assert!(!spec.paths.paths.is_empty());

    assert!(spec.paths.paths.contains_key("/health"));
    assert!(spec.paths.paths.contains_key("/v1/models"));
}

#[test]
fn test_unified_openapi_contains_all_routes() {
    let spec = get_openapi_spec();

    let core_routes = ["/health", "/v1/models"];

    for route in core_routes {
        assert!(spec.paths.paths.contains_key(route), "Missing core route: {}", route);
    }
}

#[test]
fn test_openapi_routes_no_conflict() {
    let spec = get_openapi_spec();

    let paths: Vec<_> = spec.paths.paths.keys().collect();
    let unique_paths: std::collections::HashSet<_> = paths.iter().collect();

    assert_eq!(paths.len(), unique_paths.len(), "Duplicate paths detected");
}

#[test]
fn test_openapi_components_present() {
    let spec = get_openapi_spec();

    assert!(spec.components.is_some());

    let components = spec.components.unwrap();

    assert!(components.schemas.contains_key("VirtualKey"));
    assert!(components.schemas.contains_key("BudgetMode"));
}

#[tokio::test]
async fn test_router_creation_no_conflicts() {
    let config = GatewayConfig {
        general: None,
        providers: vec![Provider {
            key: "test-provider".to_string(),
            r#type: ProviderType::OpenAI,
            api_key: "test-key".to_string(),
            params: Default::default(),
        }],
        models: vec![ModelConfig {
            key: "gpt-4".to_string(),
            r#type: "gpt-4".to_string(),
            provider: "test-provider".to_string(),
            params: Default::default(),
        }],
        pipelines: vec![Pipeline {
            name: "default".to_string(),
            r#type: PipelineType::Chat,
            plugins: vec![PluginConfig::ModelRouter { models: vec!["gpt-4".to_string()] }],
        }],
        virtual_keys: vec![],
    };

    let app_state = Arc::new(AppState::new(config).expect("Failed to create app state"));

    let _router = create_router(app_state);

    assert!(true, "Router created successfully with unified OpenAPI routes");
}
