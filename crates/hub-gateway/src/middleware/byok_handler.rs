use std::sync::Arc;

use axum::{extract::Request, http::StatusCode, middleware::Next, response::IntoResponse};
use hub_core::state::AppState;

use super::virtual_key_auth::ResolvedAuth;

pub async fn byok_handler_middleware(
    axum::extract::State(state): axum::extract::State<Arc<AppState>>,
    request: Request,
    next: Next,
) -> axum::response::Response {
    let resolved = request.extensions().get::<ResolvedAuth>().cloned();
    if let Some(ref auth) = resolved {
        if auth.is_byok {
            // BYOK: validate the provider key exists in config
            let config = state.current_config();
            if config.providers.is_empty() {
                return (StatusCode::SERVICE_UNAVAILABLE, "No providers configured").into_response();
            }
        } else if !auth.is_byok && auth.virtual_key.is_none() && auth.provider_api_key.is_empty() {
            return (StatusCode::UNAUTHORIZED, "No valid authentication").into_response();
        }
    }
    next.run(request).await
}
