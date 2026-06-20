use std::sync::Arc;

use axum::{extract::Request, http::StatusCode, middleware::Next, response::IntoResponse};
use hub_core::state::AppState;

use super::virtual_key_auth::ResolvedAuth;

pub async fn rate_limit_middleware(
    axum::extract::State(state): axum::extract::State<Arc<AppState>>,
    request: Request,
    next: Next,
) -> axum::response::Response {
    let resolved = request.extensions().get::<ResolvedAuth>().cloned();
    if let Some(auth) = resolved {
        let key = auth
            .virtual_key
            .as_ref()
            .map(|vk| vk.key_hash.clone())
            .unwrap_or_else(|| "default".to_string());
        let rpm = auth.virtual_key.as_ref().and_then(|vk| vk.rpm_limit).unwrap_or(u32::MAX);
        if !state.rate_limiter.check(&key, rpm) {
            return (StatusCode::TOO_MANY_REQUESTS, "Rate limit exceeded").into_response();
        }
    }
    next.run(request).await
}
