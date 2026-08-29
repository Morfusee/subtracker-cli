use async_trait::async_trait;
use chrono::{DateTime, Utc};
use reqwest::{Client, StatusCode};
use serde::Deserialize;
use std::{fmt, path::PathBuf, time::Duration};
use url::Url;

use crate::model::{ProviderId, ProviderSnapshot, QuotaWindow};

use super::{ProviderError, UsageProvider};

pub const DEFAULT_ENDPOINT: &str = "https://opencode.ai/zen/go/v1/usage";

pub struct OpenCodeProvider {
    client: Client,
    auth_path: PathBuf,
    endpoint: Url,
}

impl OpenCodeProvider {
    pub fn new(client: Client, auth_path: PathBuf, endpoint: Url) -> Self {
        Self {
            client,
            auth_path,
            endpoint,
        }
    }

    pub fn production() -> Result<Self, ProviderError> {
        let home = dirs::home_dir().ok_or(ProviderError::CredentialsNotFound)?;
        let auth_path = home
            .join(".local")
            .join("share")
            .join("opencode")
            .join("auth.json");
        let endpoint = Url::parse(DEFAULT_ENDPOINT).map_err(|_| ProviderError::Network)?;
        let client = Client::builder()
            .timeout(Duration::from_secs(15))
            .build()
            .map_err(|_| ProviderError::Network)?;

        Ok(Self::new(client, auth_path, endpoint))
    }

    async fn load_auth(&self) -> Result<OpenCodeAuth, ProviderError> {
        let input = tokio::fs::read_to_string(&self.auth_path)
            .await
            .map_err(|_| ProviderError::CredentialsNotFound)?;

        parse_auth(&input)
    }
}

#[async_trait]
impl UsageProvider for OpenCodeProvider {
    fn id(&self) -> ProviderId {
        ProviderId::OpenCode
    }

    async fn fetch(&self) -> Result<ProviderSnapshot, ProviderError> {
        let auth = self.load_auth().await?;

        let response = self
            .client
            .get(self.endpoint.clone())
            .header(
                reqwest::header::AUTHORIZATION,
                format!("Bearer {}", auth.api_key()),
            )
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
pub struct OpenCodeAuth {
    api_key: String,
}

impl OpenCodeAuth {
    pub fn api_key(&self) -> &str {
        &self.api_key
    }
}

impl fmt::Debug for OpenCodeAuth {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("OpenCodeAuth")
            .field("api_key", &"<redacted>")
            .finish()
    }
}

#[derive(Deserialize)]
struct AuthFile {
    opencode: Option<OpenCodeCredentials>,
    key: Option<String>,
}

#[derive(Deserialize)]
struct OpenCodeCredentials {
    key: Option<String>,
}

#[derive(Deserialize)]
struct UsageResponse {
    usage: UsageData,
}

#[derive(Deserialize)]
struct UsageData {
    rolling: Option<UsageWindow>,
    weekly: Option<UsageWindow>,
    monthly: Option<UsageWindow>,
}

#[derive(Deserialize)]
struct UsageWindow {
    percent: f64,
    #[serde(rename = "resetsAt")]
    resets_at: Option<String>,
}

pub fn parse_auth(input: &str) -> Result<OpenCodeAuth, ProviderError> {
    let parsed: AuthFile =
        serde_json::from_str(input).map_err(|_| ProviderError::CredentialsNotFound)?;

    let api_key = parsed
        .opencode
        .and_then(|c| c.key)
        .or(parsed.key)
        .filter(|k| !k.is_empty())
        .ok_or(ProviderError::CredentialsNotFound)?;

    Ok(OpenCodeAuth { api_key })
}

pub fn parse_usage(
    input: &str,
    fetched_at: DateTime<Utc>,
) -> Result<ProviderSnapshot, ProviderError> {
    let parsed: UsageResponse =
        serde_json::from_str(input).map_err(|_| ProviderError::ParseError)?;

    let definitions = [
        (parsed.usage.rolling, "5 hour", 18_000),
        (parsed.usage.weekly, "Weekly", 604_800),
        (parsed.usage.monthly, "Monthly", 2_592_000),
    ];

    let mut quotas = Vec::new();

    for (maybe_window, label, seconds) in definitions {
        let Some(window) = maybe_window else {
            continue;
        };

        if !(0.0..=100.0).contains(&window.percent) {
            return Err(ProviderError::ParseError);
        }

        let resets_at = match window.resets_at {
            Some(resets) => Some(
                DateTime::parse_from_rfc3339(&resets)
                    .map_err(|_| ProviderError::ParseError)?
                    .with_timezone(&Utc),
            ),
            None => None,
        };

        quotas.push(QuotaWindow::from_used_percent(
            label,
            window.percent,
            resets_at,
            Some(seconds),
        ));
    }

    if quotas.is_empty() {
        return Err(ProviderError::UnsupportedOutput);
    }

    Ok(ProviderSnapshot {
        provider: ProviderId::OpenCode,
        account_label: None,
        quotas,
        stats: Vec::new(),
        fetched_at,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::{TimeZone, Utc};

    #[test]
    fn parses_opencode_auth_without_exposing_key_in_debug() {
        let auth = parse_auth(include_str!(
            "../../tests/fixtures/opencode/auth-success.json"
        ))
        .unwrap();

        assert_eq!(auth.api_key(), "sk-opencode-fixture-key");

        let debug = format!("{auth:?}");
        assert!(!debug.contains("sk-opencode-fixture-key"));
    }

    #[test]
    fn converts_opencode_used_percent_to_remaining_for_each_window() {
        let fetched_at = Utc.timestamp_opt(1_788_000_100, 0).single().unwrap();
        let snapshot = parse_usage(
            include_str!("../../tests/fixtures/opencode/usage-success.json"),
            fetched_at,
        )
        .unwrap();

        assert_eq!(snapshot.provider, ProviderId::OpenCode);
        assert_eq!(snapshot.quotas.len(), 3);

        assert_eq!(snapshot.quotas[0].label, "5 hour");
        assert_eq!(snapshot.quotas[0].remaining_percent, Some(91.0));
        assert_eq!(snapshot.quotas[0].window_seconds, Some(18_000));

        assert_eq!(snapshot.quotas[1].label, "Weekly");
        assert_eq!(snapshot.quotas[1].remaining_percent, Some(88.0));
        assert_eq!(snapshot.quotas[1].window_seconds, Some(604_800));

        assert_eq!(snapshot.quotas[2].label, "Monthly");
        assert_eq!(snapshot.quotas[2].remaining_percent, Some(85.0));
        assert_eq!(snapshot.quotas[2].window_seconds, Some(2_592_000));
    }

    #[test]
    fn missing_monthly_window_is_valid() {
        let fetched_at = Utc.timestamp_opt(1_788_000_100, 0).single().unwrap();
        let snapshot = parse_usage(
            include_str!("../../tests/fixtures/opencode/usage-no-monthly.json"),
            fetched_at,
        )
        .unwrap();

        assert_eq!(snapshot.quotas.len(), 2);
        assert_eq!(snapshot.quotas[0].remaining_percent, Some(75.0));
        assert_eq!(snapshot.quotas[1].remaining_percent, Some(50.0));
    }
}
