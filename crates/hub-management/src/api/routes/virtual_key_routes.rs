use std::sync::Arc;

use axum::{
    Json, Router,
    extract::Path,
    http::StatusCode,
    routing::{get, post},
};
use hub_core::types::virtual_key::VirtualKey;
use uuid::Uuid;

use crate::services::virtual_key_service::VirtualKeyService;

/// Create virtual key routes
pub fn virtual_key_routes(service: Arc<VirtualKeyService>) -> Router {
    Router::new()
        .route("/", get(list_virtual_keys).post(create_virtual_key))
        .route("/{id}", get(get_virtual_key).put(update_virtual_key).delete(delete_virtual_key))
        .route("/{id}/rotate", post(rotate_virtual_key))
        .with_state(service)
}

async fn list_virtual_keys(
    axum::extract::State(service): axum::extract::State<Arc<VirtualKeyService>>,
) -> Result<Json<Vec<VirtualKey>>, StatusCode> {
    match service.list_keys().await {
        Ok(keys) => Ok(Json(keys)),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

async fn create_virtual_key(
    axum::extract::State(service): axum::extract::State<Arc<VirtualKeyService>>,
    Json(key): Json<VirtualKey>,
) -> Result<Json<VirtualKey>, StatusCode> {
    match service.create_key(&key).await {
        Ok(created) => Ok(Json(created)),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

async fn get_virtual_key(
    axum::extract::State(service): axum::extract::State<Arc<VirtualKeyService>>,
    Path(id): Path<Uuid>,
) -> Result<Json<VirtualKey>, StatusCode> {
    match service.get_key(id).await {
        Ok(Some(key)) => Ok(Json(key)),
        Ok(None) => Err(StatusCode::NOT_FOUND),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

async fn update_virtual_key(
    axum::extract::State(service): axum::extract::State<Arc<VirtualKeyService>>,
    Path(id): Path<Uuid>,
    Json(mut key): Json<VirtualKey>,
) -> Result<Json<VirtualKey>, StatusCode> {
    key.id = id;
    match service.update_key(&key).await {
        Ok(updated) => Ok(Json(updated)),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

async fn delete_virtual_key(
    axum::extract::State(service): axum::extract::State<Arc<VirtualKeyService>>,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, StatusCode> {
    match service.delete_key(id).await {
        Ok(true) => Ok(StatusCode::NO_CONTENT),
        Ok(false) => Err(StatusCode::NOT_FOUND),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

async fn rotate_virtual_key(
    axum::extract::State(service): axum::extract::State<Arc<VirtualKeyService>>,
    Path(id): Path<Uuid>,
) -> Result<Json<VirtualKey>, StatusCode> {
    match service.rotate_key(id).await {
        Ok(rotated) => Ok(Json(rotated)),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}
