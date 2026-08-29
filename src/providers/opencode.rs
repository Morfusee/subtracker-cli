use async_trait::async_trait;
use chrono::{DateTime, Utc};
use std::{sync::Arc, time::Duration};

use crate::model::{ProviderId, ProviderSnapshot, UsageStat, UsageValue};

use super::{
    ProviderError, UsageProvider,
    process::{CommandSpec, ProcessRunner, classify_command_output},
};

pub struct OpenCodeProvider {
    runner: Arc<dyn ProcessRunner>,
    timeout: Duration,
}

impl OpenCodeProvider {
    pub fn new(runner: Arc<dyn ProcessRunner>, timeout: Duration) -> Self {
        Self { runner, timeout }
    }

    pub fn command_spec() -> CommandSpec {
        CommandSpec::new("opencode", ["stats"])
    }
}

#[async_trait]
impl UsageProvider for OpenCodeProvider {
    fn id(&self) -> ProviderId {
        ProviderId::OpenCode
    }

    async fn fetch(&self) -> Result<ProviderSnapshot, ProviderError> {
        let output = self.runner.run(&Self::command_spec(), self.timeout).await?;

        let stdout = classify_command_output(&output)?;
        parse_stats(stdout, Utc::now())
    }
}

fn parse_stats(input: &str, fetched_at: DateTime<Utc>) -> Result<ProviderSnapshot, ProviderError> {
    let clean = strip_ansi_escapes::strip(input);
    let text = String::from_utf8_lossy(&clean);

    let sessions = parse_integer(row_value(&text, "Sessions")?)?;
    let total_cost = parse_money_cents(row_value(&text, "Total Cost")?)?;
    let input_tokens = parse_compact_count(row_value(&text, "Input")?)?;
    let output_tokens = parse_compact_count(row_value(&text, "Output")?)?;

    Ok(ProviderSnapshot {
        provider: ProviderId::OpenCode,
        account_label: None,
        quotas: Vec::new(),
        stats: vec![
            UsageStat {
                label: "Sessions".into(),
                value: UsageValue::Count(sessions),
            },
            UsageStat {
                label: "Total Cost".into(),
                value: UsageValue::MoneyCents(total_cost),
            },
            UsageStat {
                label: "Input".into(),
                value: UsageValue::Tokens(input_tokens),
            },
            UsageStat {
                label: "Output".into(),
                value: UsageValue::Tokens(output_tokens),
            },
        ],
        fetched_at,
    })
}

fn row_value<'a>(input: &'a str, label: &str) -> Result<&'a str, ProviderError> {
    input
        .lines()
        .map(|line| line.trim().trim_matches('│').trim())
        .find_map(|row| {
            row.strip_prefix(label)
                .map(str::trim)
                .filter(|value| !value.is_empty())
        })
        .ok_or(ProviderError::UnsupportedOutput)
}

fn parse_integer(value: &str) -> Result<u64, ProviderError> {
    value
        .replace(',', "")
        .parse()
        .map_err(|_| ProviderError::ParseError)
}

fn parse_money_cents(value: &str) -> Result<i64, ProviderError> {
    let value = value.strip_prefix('$').ok_or(ProviderError::ParseError)?;

    let amount: f64 = value.parse().map_err(|_| ProviderError::ParseError)?;
    Ok((amount * 100.0).round() as i64)
}

fn parse_compact_count(value: &str) -> Result<u64, ProviderError> {
    let value = value.replace(',', "");
    let (number, multiplier) = match value.chars().last() {
        Some('K') | Some('k') => (&value[..value.len() - 1], 1_000.0),
        Some('M') | Some('m') => (&value[..value.len() - 1], 1_000_000.0),
        Some('B') | Some('b') => (&value[..value.len() - 1], 1_000_000_000.0),
        Some(_) => (value.as_str(), 1.0),
        None => return Err(ProviderError::ParseError),
    };

    let parsed: f64 = number.parse().map_err(|_| ProviderError::ParseError)?;
    Ok((parsed * multiplier).round() as u64)
}
