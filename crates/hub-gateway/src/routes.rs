use std::sync::Arc;

use axum::{
    Json, Router,
    http::StatusCode,
    routing::{get, post},
};
use axum_prometheus::PrometheusMetricLayerBuilder;
use hub_core::state::AppState;

/// Create the main gateway router
pub fn create_router(state: Arc<AppState>) -> Router {
    let (prometheus_layer, metric_handle) = PrometheusMetricLayerBuilder::new()
        .with_ignore_patterns(&["/metrics", "/health"])
        .with_prefix("hub")
        .with_default_metrics()
        .build_pair();

    Router::new()
        .route("/health", get(health))
        .route("/metrics", get(|| async move { metric_handle.render() }))
        .route("/api-docs/openapi.json", get(|| async { Json(crate::openapi::get_openapi_spec()) }))
        .route("/v1/models", get(models))
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

/// No config fallback router
pub fn create_no_config_router() -> Router {
    Router::new()
        .route("/chat/completions", post(no_config_handler))
        .route("/completions", post(no_config_handler))
        .route("/embeddings", post(no_config_handler))
        .fallback(no_config_handler)
}

async fn no_config_handler() -> Result<Json<serde_json::Value>, StatusCode> {
    tracing::warn!("No configuration available - returning 404 Not Found");
    Err(StatusCode::NOT_FOUND)
}
