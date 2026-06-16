use utoipa::OpenApi;

#[derive(OpenApi)]
#[openapi(
    info(
        title = "Hub LLM Gateway",
        version = "0.10.0",
        description = "A universal LLM API gateway with BYOK virtual key support"
    ),
    paths(crate::routes::health, crate::routes::models,),
    components(schemas(
        hub_core::types::virtual_key::VirtualKey,
        hub_core::types::virtual_key::BudgetMode,
    ))
)]
pub struct ApiDoc;

pub fn get_openapi_spec() -> utoipa::openapi::OpenApi {
    ApiDoc::openapi()
}
