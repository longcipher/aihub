use std::sync::Arc;

use axum::{Json, Router, extract::Extension, http::StatusCode, routing::post};
use hub_core::{
    state::AppState,
    types::{Pipeline, PipelineType, PluginConfig},
};
use liter_llm::LlmClient;

use super::super::middleware::virtual_key_auth::{ResolvedAuth, is_model_allowed};

pub fn create_pipeline_router(
    pipelines: &[Pipeline],
    _state: Arc<AppState>,
) -> Router<Arc<AppState>> {
    let mut router = Router::new();

    for pipeline in pipelines {
        for plugin in &pipeline.plugins {
            if let PluginConfig::ModelRouter { models } = plugin {
                match pipeline.r#type {
                    PipelineType::Chat => {
                        router = router.route(
                            &format!("/{}/chat/completions", pipeline.name),
                            post({
                                let models = models.clone();
                                move |state: axum::extract::State<Arc<AppState>>,
                                      auth: Extension<ResolvedAuth>,
                                      payload: Json<serde_json::Value>| {
                                    let models = models.clone();
                                    let state = state.0.clone();
                                    async move {
                                        chat_completions(state, auth.0, payload, models).await
                                    }
                                }
                            }),
                        );
                    }
                    PipelineType::Completion => {
                        router = router.route(
                            &format!("/{}/completions", pipeline.name),
                            post({
                                let models = models.clone();
                                move |state: axum::extract::State<Arc<AppState>>,
                                      auth: Extension<ResolvedAuth>,
                                      payload: Json<serde_json::Value>| {
                                    let models = models.clone();
                                    let state = state.0.clone();
                                    async move {
                                        completions(state, auth.0, payload, models).await
                                    }
                                }
                            }),
                        );
                    }
                    PipelineType::Embeddings => {
                        router = router.route(
                            &format!("/{}/embeddings", pipeline.name),
                            post({
                                let models = models.clone();
                                move |state: axum::extract::State<Arc<AppState>>,
                                      auth: Extension<ResolvedAuth>,
                                      payload: Json<serde_json::Value>| {
                                    let models = models.clone();
                                    let state = state.0.clone();
                                    async move {
                                        embeddings(state, auth.0, payload, models).await
                                    }
                                }
                            }),
                        );
                    }
                }
            }
        }
    }

    router
}

async fn chat_completions(
    state: Arc<AppState>,
    auth: ResolvedAuth,
    Json(payload): Json<serde_json::Value>,
    model_keys: Vec<String>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let model_name =
        payload.get("model").and_then(|v| v.as_str()).ok_or(StatusCode::BAD_REQUEST)?;

    if !is_model_allowed(&auth, model_name) {
        return Err(StatusCode::FORBIDDEN);
    }

    for key in &model_keys {
        if let Some(model) = state.model_registry.get(key) &&
            model.model_type == model_name
        {
            let client = model.provider.client();
            let request: liter_llm::types::chat::ChatCompletionRequest =
                serde_json::from_value(payload.clone()).map_err(|_| StatusCode::BAD_REQUEST)?;
            let response =
                client.chat(request).await.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
            let response_json =
                serde_json::to_value(response).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
            return Ok(Json(response_json));
        }
    }

    Err(StatusCode::NOT_FOUND)
}

async fn completions(
    state: Arc<AppState>,
    _auth: ResolvedAuth,
    Json(payload): Json<serde_json::Value>,
    model_keys: Vec<String>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let model_name =
        payload.get("model").and_then(|v| v.as_str()).ok_or(StatusCode::BAD_REQUEST)?;

    for key in &model_keys {
        if let Some(model) = state.model_registry.get(key) &&
            model.model_type == model_name
        {
            // TODO: implement completions via liter-llm
            return Err(StatusCode::NOT_IMPLEMENTED);
        }
    }

    Err(StatusCode::NOT_FOUND)
}

async fn embeddings(
    state: Arc<AppState>,
    auth: ResolvedAuth,
    Json(payload): Json<serde_json::Value>,
    model_keys: Vec<String>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let model_name =
        payload.get("model").and_then(|v| v.as_str()).ok_or(StatusCode::BAD_REQUEST)?;

    if !is_model_allowed(&auth, model_name) {
        return Err(StatusCode::FORBIDDEN);
    }

    for key in &model_keys {
        if let Some(model) = state.model_registry.get(key) &&
            model.model_type == model_name
        {
            let client = model.provider.client();
            let request: liter_llm::types::embedding::EmbeddingRequest =
                serde_json::from_value(payload.clone()).map_err(|_| StatusCode::BAD_REQUEST)?;
            let response =
                client.embed(request).await.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
            let response_json =
                serde_json::to_value(response).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
            return Ok(Json(response_json));
        }
    }

    Err(StatusCode::NOT_FOUND)
}
