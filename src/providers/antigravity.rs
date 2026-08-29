use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde_json::Value;
use std::{sync::Arc, time::Duration};

use crate::model::{ProviderId, ProviderSnapshot, QuotaWindow};

use super::{
    ProviderError, UsageProvider,
    process::{CommandSpec, ProcessRunner, classify_command_output},
};

pub struct AntigravityProvider {
    runner: Arc<dyn ProcessRunner>,
    timeout: Duration,
}

impl AntigravityProvider {
    pub fn new(runner: Arc<dyn ProcessRunner>, timeout: Duration) -> Self {
        Self { runner, timeout }
    }

    pub fn command_spec() -> CommandSpec {
        CommandSpec::new("agy", ["-p", "/usage", "--output-format", "json"])
    }
}

#[async_trait]
impl UsageProvider for AntigravityProvider {
    fn id(&self) -> ProviderId {
        ProviderId::Antigravity
    }

    async fn fetch(&self) -> Result<ProviderSnapshot, ProviderError> {
        let output = self.runner.run(&Self::command_spec(), self.timeout).await?;

        let stdout = classify_command_output(&output)?;
        parse_usage(stdout, Utc::now())
    }
}

pub fn parse_usage(
    input: &str,
    fetched_at: DateTime<Utc>,
) -> Result<ProviderSnapshot, ProviderError> {
    let value: Value = serde_json::from_str(input).map_err(|_| ProviderError::ParseError)?;

    let definitions = [
        ("gemini-5h", "Gemini 5 hour", 18_000),
        ("gemini-weekly", "Gemini weekly", 604_800),
        ("3p-5h", "Claude/GPT 5 hour", 18_000),
        ("3p-weekly", "Claude/GPT weekly", 604_800),
    ];

    let mut quotas = Vec::new();

    for (key, label, seconds) in definitions {
        let Some(bucket) = find_bucket_by_id(&value, key) else {
            continue;
        };

        if bucket
            .get("disabled")
            .and_then(Value::as_bool)
            .unwrap_or(false)
        {
            continue;
        }

        let remaining_fraction = bucket
            .get("remaining_fraction")
            .and_then(Value::as_f64)
            .ok_or(ProviderError::ParseError)?;

        if !(0.0..=1.0).contains(&remaining_fraction) {
            return Err(ProviderError::ParseError);
        }

        let resets_at = match bucket.get("reset_time") {
            Some(Value::String(value)) => Some(
                DateTime::parse_from_rfc3339(value)
                    .map_err(|_| ProviderError::ParseError)?
                    .with_timezone(&Utc),
            ),
            Some(Value::Null) | None => None,
            Some(_) => return Err(ProviderError::ParseError),
        };

        let remaining_percent = remaining_fraction * 100.0;

        quotas.push(QuotaWindow {
            label: label.into(),
            used_percent: Some(100.0 - remaining_percent),
            remaining_percent: Some(remaining_percent),
            resets_at,
            window_seconds: Some(seconds),
        });
    }

    if quotas.is_empty() {
        return Err(ProviderError::UnsupportedOutput);
    }

    Ok(ProviderSnapshot {
        provider: ProviderId::Antigravity,
        account_label: None,
        quotas,
        stats: Vec::new(),
        fetched_at,
    })
}

fn find_bucket_by_id<'a>(value: &'a Value, target_id: &str) -> Option<&'a Value> {
    match value {
        Value::Object(object) => {
            if object.get("id").and_then(Value::as_str) == Some(target_id) {
                return Some(value);
            }

            if let Some(bucket) = object.get(target_id).filter(|b| b.is_object()) {
                return Some(bucket);
            }

            for child in object.values() {
                if let Some(found) = find_bucket_by_id(child, target_id) {
                    return Some(found);
                }
            }
            None
        }
        Value::Array(array) => {
            for child in array {
                if let Some(found) = find_bucket_by_id(child, target_id) {
                    return Some(found);
                }
            }
            None
        }
        _ => None,
    }
}
