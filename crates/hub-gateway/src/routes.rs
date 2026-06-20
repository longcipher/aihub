use std::sync::Arc;

use axum::{Json, Router, http::StatusCode, routing::get};
use axum_prometheus::PrometheusMetricLayerBuilder;
use hub_core::state::AppState;

use crate::middleware::{
    budget_enforcer::budget_middleware, byok_handler::byok_handler_middleware,
    rate_limiter::rate_limit_middleware, virtual_key_auth::virtual_key_auth,
};

pub fn create_router(state: Arc<AppState>) -> Router {
    let (prometheus_layer, metric_handle) = PrometheusMetricLayerBuilder::new()
        .with_ignore_patterns(&["/metrics", "/health"])
        .with_prefix("hub")
        .with_default_metrics()
        .build_pair();

    // Build pipeline routes from config
    let config = state.current_config();
    let pipeline_router =
        crate::pipeline::handler::create_pipeline_router(&config.pipelines, state.clone());

    // Compose middleware stack: auth → rate limit → budget → BYOK
    let protected_routes = pipeline_router
        .layer(axum::middleware::from_fn_with_state(state.clone(), virtual_key_auth))
        .layer(axum::middleware::from_fn_with_state(state.clone(), rate_limit_middleware))
        .layer(axum::middleware::from_fn_with_state(state.clone(), budget_middleware))
        .layer(axum::middleware::from_fn_with_state(state.clone(), byok_handler_middleware));

    Router::new()
        .route("/health", get(health))
        .route("/metrics", get(|| async move { metric_handle.render() }))
        .route("/api-docs/openapi.json", get(|| async { Json(crate::openapi::get_openapi_spec()) }))
        .route("/v1/models", get(models))
        .merge(protected_routes)
        .layer(prometheus_layer)
        .with_state(state)
}

/// Health check endpoint
#[utoipa::path(
    get,
    path = "/health",
    responses(
        (status = 200, description = "Service is healthy")
    )
)]
pub async fn health() -> &'static str {
    "Working!"
}

/// Models list endpoint
#[utoipa::path(
    get,
    path = "/v1/models",
    responses(
        (status = 200, description = "List of available models")
    )
)]
pub async fn models(
    state: axum::extract::State<Arc<AppState>>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let config = state.current_config();
    let models: Vec<serde_json::Value> = config
        .models
        .iter()
        .map(|m| {
            serde_json::json!({
                "id": m.key,
                "object": "model",
                "owned_by": m.provider,
                "model_type": m.r#type,
            })
        })
        .collect();

    Ok(Json(serde_json::json!({
        "object": "list",
        "data": models,
    })))
}
