use async_trait::async_trait;
use thiserror::Error;

use crate::model::{ProviderId, ProviderSnapshot};

pub mod codex;
pub mod opencode;
pub mod process;

#[derive(Clone, Debug, Error, Eq, PartialEq)]
pub enum ProviderError {
    #[error("CLI not found")]
    CliNotFound,
    #[error("credentials not found")]
    CredentialsNotFound,
    #[error("not authenticated")]
    NotAuthenticated,
    #[error("provider timed out")]
    Timeout,
    #[error("network request failed")]
    Network,
    #[error("provider command failed")]
    CommandFailed,
    #[error("provider output could not be parsed")]
    ParseError,
    #[error("provider output format is unsupported")]
    UnsupportedOutput,
}

#[async_trait]
pub trait UsageProvider: Send + Sync {
    fn id(&self) -> ProviderId;
    async fn fetch(&self) -> Result<ProviderSnapshot, ProviderError>;
}
