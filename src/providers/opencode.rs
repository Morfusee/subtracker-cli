use async_trait::async_trait;
use chrono::{DateTime, Utc};
use reqwest::{Client, StatusCode};
use serde::Deserialize;
use serde_json::Value;
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
        if let Ok(input) = tokio::fs::read_to_string(&self.auth_path).await
            && let Ok(auth) = parse_auth(&input)
        {
            return Ok(auth);
        }

        if let Some(parent) = self.auth_path.parent()
            && let Ok(input) = tokio::fs::read_to_string(parent.join("account.json")).await
            && let Ok(auth) = parse_auth(&input)
        {
            return Ok(auth);
        }

        Err(ProviderError::CredentialsNotFound)
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
    let value: Value =
        serde_json::from_str(input).map_err(|_| ProviderError::CredentialsNotFound)?;

    if let Some(key) = extract_key(&value) {
        return Ok(OpenCodeAuth { api_key: key });
    }

    Err(ProviderError::CredentialsNotFound)
}

fn valid_key(s: &str) -> Option<String> {
    if !s.is_empty() {
        Some(s.to_string())
    } else {
        None
    }
}

fn extract_from_val(val: &Value) -> Option<String> {
    val.get("key")
        .and_then(Value::as_str)
        .and_then(valid_key)
        .or_else(|| {
            val.get("apiKey")
                .and_then(Value::as_str)
                .and_then(valid_key)
        })
        .or_else(|| val.as_str().and_then(valid_key))
}

fn extract_key(value: &Value) -> Option<String> {
    match value {
        Value::Object(map) => {
            let candidates = ["opencode-go", "opencode", "opencode_go", "zen", "go"];
            for candidate in candidates {
                if let Some(entry) = map.get(candidate)
                    && let Some(found) = extract_from_val(entry)
                {
                    return Some(found);
                }
            }

            if let Some(found) = map.get("key").and_then(Value::as_str).and_then(valid_key) {
                return Some(found);
            }
            if let Some(found) = map
                .get("apiKey")
                .and_then(Value::as_str)
                .and_then(valid_key)
            {
                return Some(found);
            }

            for (prop_name, prop_val) in map {
                if (prop_name.contains("opencode") || prop_name.contains("zen"))
                    && let Some(found) = extract_from_val(prop_val)
                {
                    return Some(found);
                }
                if let Some(found) = extract_key(prop_val) {
                    return Some(found);
                }
            }
            None
        }
        Value::Array(arr) => {
            for item in arr {
                if let Some(found) = extract_key(item) {
                    return Some(found);
                }
            }
            None
        }
        _ => None,
    }
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
    fn parses_opencode_auth_with_opencode_key_without_exposing_in_debug() {
        let auth = parse_auth(include_str!(
            "../../tests/fixtures/opencode/auth-success.json"
        ))
        .unwrap();

        assert_eq!(auth.api_key(), "sk-opencode-fixture-key");

        let debug = format!("{auth:?}");
        assert!(!debug.contains("sk-opencode-fixture-key"));
    }

    #[test]
    fn parses_opencode_auth_with_opencode_go_key() {
        let json = r#"{"opencode-go": {"type": "api", "key": "sk-opencode-go-12345"}}"#;
        let auth = parse_auth(json).unwrap();
        assert_eq!(auth.api_key(), "sk-opencode-go-12345");
    }

    #[test]
    fn parses_opencode_account_json_structure() {
        let json = r#"{
            "version": 2,
            "active": {"opencode-go": "acc-1"},
            "accounts": {
                "acc-1": {
                    "id": "acc-1",
                    "serviceID": "opencode-go",
                    "credential": {"key": "sk-opencode-from-account"}
                }
            }
        }"#;
        let auth = parse_auth(json).unwrap();
        assert_eq!(auth.api_key(), "sk-opencode-from-account");
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
