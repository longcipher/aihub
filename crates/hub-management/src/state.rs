use std::sync::Arc;

use axum::Router;
use sqlx::PgPool;

use crate::{
    api::routes::virtual_key_routes::virtual_key_routes,
    db::repositories::virtual_key_repository::VirtualKeyRepository,
    services::virtual_key_service::VirtualKeyService,
};

/// Create management API router
pub fn create_management_router(pool: PgPool) -> Router {
    let virtual_key_repo = VirtualKeyRepository::new(pool);
    let virtual_key_service = Arc::new(VirtualKeyService::new(virtual_key_repo));

    Router::new()
        .nest("/api/v1/management/virtual-keys", virtual_key_routes(virtual_key_service))
        .route("/health", axum::routing::get(|| async { "Management API is healthy" }))
}
