use thiserror::Error;

#[derive(Error, Debug)]
pub enum HubError {
    #[error("Configuration error: {0}")]
    Config(String),

    #[error("Provider initialization error: {0}")]
    ProviderInit(String),

    #[error("Provider request error: {0}")]
    ProviderRequest(String),

    #[error("Model not found: {0}")]
    ModelNotFound(String),

    #[error("Virtual key error: {0}")]
    VirtualKey(String),

    #[error("Authentication error: {0}")]
    Auth(String),

    #[error("Rate limit exceeded")]
    RateLimitExceeded,

    #[error("Budget exceeded")]
    BudgetExceeded,

    #[error("Invalid request: {0}")]
    BadRequest(String),

    #[error("Internal error: {0}")]
    Internal(String),
}
