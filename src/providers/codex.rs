use async_trait::async_trait;
use chrono::{DateTime, TimeZone, Utc};
use reqwest::{Client, StatusCode};
use serde::Deserialize;
use std::{fmt, path::PathBuf, time::Duration};
use url::Url;

use crate::model::{ProviderId, ProviderSnapshot, QuotaWindow};

use super::{ProviderError, UsageProvider};

pub const DEFAULT_ENDPOINT: &str = "https://chatgpt.com/backend-api/wham/usage";

pub struct CodexProvider {
    client: Client,
    auth_path: PathBuf,
    endpoint: Url,
}

impl CodexProvider {
    pub fn new(client: Client, auth_path: PathBuf, endpoint: Url) -> Self {
        Self {
            client,
            auth_path,
            endpoint,
        }
    }

    pub fn production() -> Result<Self, ProviderError> {
        let home = dirs::home_dir().ok_or(ProviderError::CredentialsNotFound)?;
        let auth_path = home.join(".codex").join("auth.json");
        let endpoint = Url::parse(DEFAULT_ENDPOINT).map_err(|_| ProviderError::Network)?;
        let client = Client::builder()
            .timeout(Duration::from_secs(15))
            .build()
            .map_err(|_| ProviderError::Network)?;

        Ok(Self::new(client, auth_path, endpoint))
    }

    async fn load_auth(&self) -> Result<CodexAuth, ProviderError> {
        let input = tokio::fs::read_to_string(&self.auth_path)
            .await
            .map_err(|_| ProviderError::CredentialsNotFound)?;

        parse_auth(&input)
    }
}

#[async_trait]
impl UsageProvider for CodexProvider {
    fn id(&self) -> ProviderId {
        ProviderId::Codex
    }

    async fn fetch(&self) -> Result<ProviderSnapshot, ProviderError> {
        let auth = self.load_auth().await?;

        let response = self
            .client
            .get(self.endpoint.clone())
            .header(
                reqwest::header::AUTHORIZATION,
                format!("Bearer {}", auth.access_token()),
            )
            .header("ChatGPT-Account-Id", auth.account_id())
            .header(reqwest::header::ACCEPT, "application/json")
            .send()
            .await
            .map_err(|_| ProviderError::Network)?;

        if matches!(
            response.status(),
            StatusCode::UNAUTHORIZED | StatusCode::FORBIDDEN
        ) {
            return Err(ProviderError::NotAuthenticated);
        }

        if !response.status().is_success() {
            return Err(ProviderError::Network);
        }

        let body = response.text().await.map_err(|_| ProviderError::Network)?;

        parse_usage(&body, Utc::now())
    }
}

#[derive(Clone)]
pub struct CodexAuth {
    access_token: String,
    account_id: String,
}

impl CodexAuth {
    pub fn access_token(&self) -> &str {
        &self.access_token
    }

    pub fn account_id(&self) -> &str {
        &self.account_id
    }
}

impl fmt::Debug for CodexAuth {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("CodexAuth")
            .field("access_token", &"<redacted>")
            .field("account_id", &"<redacted>")
            .finish()
    }
}

#[derive(Deserialize)]
struct AuthFile {
    tokens: AuthTokens,
}

#[derive(Deserialize)]
struct AuthTokens {
    access_token: String,
    account_id: String,
}

#[derive(Deserialize)]
struct UsageResponse {
    rate_limit: RateLimit,
}

#[derive(Deserialize)]
struct RateLimit {
    primary_window: Option<RateWindow>,
    secondary_window: Option<RateWindow>,
}

#[derive(Deserialize)]
struct RateWindow {
    used_percent: f64,
    limit_window_seconds: u64,
    reset_at: i64,
}

pub fn parse_auth(input: &str) -> Result<CodexAuth, ProviderError> {
    let parsed: AuthFile =
        serde_json::from_str(input).map_err(|_| ProviderError::CredentialsNotFound)?;

    if parsed.tokens.access_token.is_empty() || parsed.tokens.account_id.is_empty() {
        return Err(ProviderError::CredentialsNotFound);
    }

    Ok(CodexAuth {
        access_token: parsed.tokens.access_token,
        account_id: parsed.tokens.account_id,
    })
}

pub fn parse_usage(
    input: &str,
    fetched_at: DateTime<Utc>,
) -> Result<ProviderSnapshot, ProviderError> {
    let parsed: UsageResponse =
        serde_json::from_str(input).map_err(|_| ProviderError::ParseError)?;

    let mut quotas = Vec::new();

    if let Some(window) = parsed.rate_limit.primary_window {
        quotas.push(normalize_window(window)?);
    }

    if let Some(window) = parsed.rate_limit.secondary_window {
        quotas.push(normalize_window(window)?);
    }

    if quotas.is_empty() {
        return Err(ProviderError::UnsupportedOutput);
    }

    Ok(ProviderSnapshot {
        provider: ProviderId::Codex,
        account_label: None,
        quotas,
        stats: Vec::new(),
        fetched_at,
    })
}

fn normalize_window(window: RateWindow) -> Result<QuotaWindow, ProviderError> {
    if !(0.0..=100.0).contains(&window.used_percent) {
        return Err(ProviderError::ParseError);
    }

    let reset = Utc
        .timestamp_opt(window.reset_at, 0)
        .single()
        .ok_or(ProviderError::ParseError)?;

    Ok(QuotaWindow::from_used_percent(
        window_label(window.limit_window_seconds),
        window.used_percent,
        Some(reset),
        Some(window.limit_window_seconds),
    ))
}

fn window_label(seconds: u64) -> String {
    match seconds {
        18_000 => "5 hour".into(),
        604_800 => "Weekly".into(),
        value if value % 86_400 == 0 => format!("{} day", value / 86_400),
        value if value % 3_600 == 0 => format!("{} hour", value / 3_600),
        value => format!("{value}s"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::{TimeZone, Utc};

    #[test]
    fn parses_current_codex_auth_without_exposing_tokens_in_debug() {
        let auth =
            parse_auth(include_str!("../../tests/fixtures/codex/auth-success.json")).unwrap();

        assert_eq!(auth.access_token(), "fixture-access-token");
        assert_eq!(auth.account_id(), "fixture-account-id");

        let debug = format!("{auth:?}");
        assert!(!debug.contains("fixture-access-token"));
        assert!(!debug.contains("fixture-refresh-token"));
        assert!(!debug.contains("fixture-account-id"));
    }

    #[test]
    fn converts_codex_used_percent_to_remaining_for_each_window() {
        let fetched_at = Utc.timestamp_opt(1_788_000_100, 0).single().unwrap();
        let snapshot = parse_usage(
            include_str!("../../tests/fixtures/codex/usage-success.json"),
            fetched_at,
        )
        .unwrap();

        assert_eq!(snapshot.provider, ProviderId::Codex);
        assert_eq!(snapshot.quotas.len(), 2);
        assert_eq!(snapshot.quotas[0].remaining_percent, Some(77.0));
        assert_eq!(snapshot.quotas[0].window_seconds, Some(18_000));
        assert_eq!(snapshot.quotas[1].remaining_percent, Some(59.0));
        assert_eq!(snapshot.quotas[1].window_seconds, Some(604_800));
    }

    #[test]
    fn missing_secondary_window_is_valid() {
        let fetched_at = Utc.timestamp_opt(1_788_000_100, 0).single().unwrap();
        let snapshot = parse_usage(
            include_str!("../../tests/fixtures/codex/usage-no-secondary.json"),
            fetched_at,
        )
        .unwrap();

        assert_eq!(snapshot.quotas.len(), 1);
        assert_eq!(snapshot.quotas[0].remaining_percent, Some(90.0));
    }
}
