use std::sync::Arc;

use axum::{Json, Router, http::StatusCode, routing::post};
use hub_core::{
    models::registry::ModelRegistry,
    types::{Pipeline, PipelineType, PluginConfig},
};
use liter_llm::LlmClient;

/// Create a pipeline router
pub fn create_pipeline(pipeline: &Pipeline, model_registry: Arc<ModelRegistry>) -> Router {
    let mut router = Router::new();

    for plugin in &pipeline.plugins {
        if let PluginConfig::ModelRouter { models } = plugin {
            match pipeline.r#type {
                PipelineType::Chat => {
                    router = router.route(
                        "/chat/completions",
                        post({
                            let models = models.clone();
                            let registry = model_registry.clone();
                            move |payload| chat_completions(registry, payload, models)
                        }),
                    );
                }
                PipelineType::Completion => {
                    router = router.route(
                        "/completions",
                        post({
                            let models = models.clone();
                            let registry = model_registry.clone();
                            move |payload| completions(registry, payload, models)
                        }),
                    );
                }
                PipelineType::Embeddings => {
                    router = router.route(
                        "/embeddings",
                        post({
                            let models = models.clone();
                            let registry = model_registry.clone();
                            move |payload| embeddings(registry, payload, models)
                        }),
                    );
                }
            }
        }
    }

    router
}

async fn chat_completions(
    registry: Arc<ModelRegistry>,
    Json(payload): Json<serde_json::Value>,
    model_keys: Vec<String>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let model_name =
        payload.get("model").and_then(|v| v.as_str()).ok_or(StatusCode::BAD_REQUEST)?;

    // Find matching model
    for key in &model_keys {
        if let Some(model) = registry.get(key) &&
            model.model_type == model_name
        {
            // Use liter-llm to make the request
            let client = model.provider.client();

            // Convert payload to liter-llm request format
            let request: liter_llm::types::chat::ChatCompletionRequest =
                serde_json::from_value(payload.clone()).map_err(|_| StatusCode::BAD_REQUEST)?;

            // Make the request
            let response =
                client.chat(request).await.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

            // Convert response to JSON
            let response_json =
                serde_json::to_value(response).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

            return Ok(Json(response_json));
        }
    }

    Err(StatusCode::NOT_FOUND)
}

async fn completions(
    registry: Arc<ModelRegistry>,
    Json(payload): Json<serde_json::Value>,
    model_keys: Vec<String>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let model_name =
        payload.get("model").and_then(|v| v.as_str()).ok_or(StatusCode::BAD_REQUEST)?;

    // Find matching model
    for key in &model_keys {
        if let Some(model) = registry.get(key) &&
            model.model_type == model_name
        {
            // TODO: Implement completions with liter-llm
            return Err(StatusCode::NOT_IMPLEMENTED);
        }
    }

    Err(StatusCode::NOT_FOUND)
}

async fn embeddings(
    registry: Arc<ModelRegistry>,
    Json(payload): Json<serde_json::Value>,
    model_keys: Vec<String>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let model_name =
        payload.get("model").and_then(|v| v.as_str()).ok_or(StatusCode::BAD_REQUEST)?;

    // Find matching model
    for key in &model_keys {
        if let Some(model) = registry.get(key) &&
            model.model_type == model_name
        {
            // Use liter-llm to make the request
            let client = model.provider.client();

            // Convert payload to liter-llm request format
            let request: liter_llm::types::embedding::EmbeddingRequest =
                serde_json::from_value(payload.clone()).map_err(|_| StatusCode::BAD_REQUEST)?;

            // Make the request
            let response =
                client.embed(request).await.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

            // Convert response to JSON
            let response_json =
                serde_json::to_value(response).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

            return Ok(Json(response_json));
        }
    }

    Err(StatusCode::NOT_FOUND)
}
