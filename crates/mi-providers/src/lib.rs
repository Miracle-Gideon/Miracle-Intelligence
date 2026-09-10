//! mi-providers: the Provider trait plus concrete implementations.
//!
//! Every provider returns raw, unmodified payloads wrapped in
//! `ProviderResult` — normalization into Asset/Observation happens one
//! layer up, so providers stay simple, swappable, and easy to test.

use async_trait::async_trait;
use thiserror::Error;

pub mod shodan;

#[derive(Debug, Error)]
pub enum ProviderError {
    #[error("request failed: {0}")]
    Request(#[from] reqwest::Error),
    #[error("invalid or unauthorized API key")]
    InvalidKey,
    #[error("rate limited, retry after {retry_after_secs}s")]
    RateLimited { retry_after_secs: u64 },
    #[error("query credits exhausted")]
    CreditsExhausted,
    #[error("no results for query")]
    NotFound,
    #[error("unexpected response shape: {0}")]
    UnexpectedResponse(String),
}

#[derive(Debug, Clone)]
pub struct ProviderResult {
    pub provider: String,
    pub raw: serde_json::Value,
}

#[derive(Debug, Clone)]
pub struct ProviderCapabilities {
    pub host_info: bool,
    pub search: bool,
    pub history: bool,
}

#[async_trait]
pub trait Provider: Send + Sync {
    fn name(&self) -> &str;

    async fn host_info(&self, ip: &str) -> Result<ProviderResult, ProviderError>;
    async fn search(&self, query: &str) -> Result<Vec<ProviderResult>, ProviderError>;

    /// Validates the configured API key against the provider's own
    /// account/info endpoint. Used on first launch and by `mi auth list-keys`.
    async fn validate_key(&self) -> Result<KeyInfo, ProviderError>;

    fn capabilities(&self) -> ProviderCapabilities;
}

#[derive(Debug, Clone)]
pub struct KeyInfo {
    pub plan: Option<String>,
    pub credits_remaining: Option<i64>,
}
