use std::sync::Arc;

use axum::{extract::Request, http::StatusCode, middleware::Next, response::IntoResponse};
use hub_core::state::AppState;

use super::virtual_key_auth::ResolvedAuth;

pub async fn budget_middleware(
    axum::extract::State(state): axum::extract::State<Arc<AppState>>,
    request: Request,
    next: Next,
) -> axum::response::Response {
    let resolved = request.extensions().get::<ResolvedAuth>().cloned();
    if let Some(auth) = resolved &&
        let Some(ref vk) = auth.virtual_key
    {
        let (allowed, _) = state.budget_enforcer.check_budget(
            &vk.key_hash,
            vk.monthly_budget_cents,
            &vk.budget_mode,
        );
        if !allowed {
            return (StatusCode::PAYMENT_REQUIRED, "Budget exceeded").into_response();
        }
    }
    next.run(request).await
}
