use thiserror::Error;

#[derive(Error, Debug)]
pub enum HubError {
    #[error("Configuration error: {0}")]
    Config(String),

    #[error("Provider initialization error: {0}")]
    ProviderInit(String),

    #[error("Provider request error: {0}")]
    ProviderRequest(String),

    #[error("Internal error: {0}")]
    Internal(String),
}
