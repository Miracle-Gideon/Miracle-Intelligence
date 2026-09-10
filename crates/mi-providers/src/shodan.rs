use async_trait::async_trait;
use reqwest::{Client, StatusCode};
use serde_json::Value;

use crate::{KeyInfo, Provider, ProviderCapabilities, ProviderError, ProviderResult};

const BASE_URL: &str = "https://api.shodan.io";

pub struct ShodanProvider {
    api_key: String,
    client: Client,
}

impl ShodanProvider {
    pub fn new(api_key: impl Into<String>) -> Self {
        Self {
            api_key: api_key.into(),
            client: Client::new(),
        }
    }

    /// Maps a raw HTTP response into either the parsed JSON body or a
    /// typed ProviderError, so every method handles auth/rate-limit/credit
    /// failures the same way.
    async fn handle_response(resp: reqwest::Response) -> Result<Value, ProviderError> {
        match resp.status() {
            StatusCode::OK => Ok(resp.json::<Value>().await?),
            StatusCode::UNAUTHORIZED | StatusCode::FORBIDDEN => Err(ProviderError::InvalidKey),
            StatusCode::TOO_MANY_REQUESTS => {
                let retry_after_secs = resp
                    .headers()
                    .get("Retry-After")
                    .and_then(|v| v.to_str().ok())
                    .and_then(|s| s.parse().ok())
                    .unwrap_or(60);
                Err(ProviderError::RateLimited { retry_after_secs })
            }
            StatusCode::PAYMENT_REQUIRED => Err(ProviderError::CreditsExhausted),
            StatusCode::NOT_FOUND => Err(ProviderError::NotFound),
            other => {
                let body = resp.text().await.unwrap_or_default();
                Err(ProviderError::UnexpectedResponse(format!(
                    "status {other}: {body}"
                )))
            }
        }
    }
}

#[async_trait]
impl Provider for ShodanProvider {
    fn name(&self) -> &str {
        "shodan"
    }

    async fn host_info(&self, ip: &str) -> Result<ProviderResult, ProviderError> {
        let url = format!("{BASE_URL}/shodan/host/{ip}");
        let resp = self.client.get(&url).query(&[("key", &self.api_key)]).send().await?;
        let raw = Self::handle_response(resp).await?;
        Ok(ProviderResult {
            provider: self.name().to_string(),
            raw,
        })
    }

    async fn search(&self, query: &str) -> Result<Vec<ProviderResult>, ProviderError> {
        let url = format!("{BASE_URL}/shodan/host/search");
        let resp = self
            .client
            .get(&url)
            .query(&[("key", self.api_key.as_str()), ("query", query)])
            .send()
            .await?;
        let body = Self::handle_response(resp).await?;

        let matches = body
            .get("matches")
            .and_then(|m| m.as_array())
            .cloned()
            .unwrap_or_default();

        Ok(matches
            .into_iter()
            .map(|raw| ProviderResult {
                provider: self.name().to_string(),
                raw,
            })
            .collect())
    }

    async fn validate_key(&self) -> Result<KeyInfo, ProviderError> {
        let url = format!("{BASE_URL}/api-info");
        let resp = self.client.get(&url).query(&[("key", &self.api_key)]).send().await?;
        let body = Self::handle_response(resp).await?;

        Ok(KeyInfo {
            plan: body.get("plan").and_then(|v| v.as_str()).map(String::from),
            credits_remaining: body.get("query_credits").and_then(|v| v.as_i64()),
        })
    }

    fn capabilities(&self) -> ProviderCapabilities {
        ProviderCapabilities {
            host_info: true,
            search: true,
            history: false, // Shodan history requires a higher-tier plan; Phase 2
        }
    }
}
