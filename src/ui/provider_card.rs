use chrono::{DateTime, Utc};
use ratatui::text::Line;

use crate::{
    app::{DisplayState, ProviderState},
    providers::ProviderError,
};

use super::format::{format_age, format_reset, format_usage_value, quota_bar};

pub fn lines_for_provider(state: &ProviderState, now: DateTime<Utc>) -> Vec<Line<'static>> {
    let mut lines = Vec::new();

    if let Some(snapshot) = &state.snapshot {
        for quota in &snapshot.quotas {
            let Some(remaining) = quota.remaining_percent else {
                continue;
            };

            let mut text = format!("{}  {}", quota.label, quota_bar(remaining));

            if let Some(reset) = quota.resets_at {
                text.push_str(&format!("  resets {}", format_reset(reset, now)));
            }

            lines.push(Line::from(text));
        }

        for stat in &snapshot.stats {
            lines.push(Line::from(format!(
                "{:<12} {}",
                stat.label,
                format_usage_value(&stat.value)
            )));
        }

        lines.push(Line::from(format!(
            "updated {}",
            format_age(snapshot.fetched_at, now)
        )));
    }

    match &state.display {
        DisplayState::Loading => {
            lines.push(Line::from("Loading..."));
        }
        DisplayState::Refreshing if state.snapshot.is_none() => {
            lines.push(Line::from("Loading..."));
        }
        DisplayState::Refreshing => {
            lines.push(Line::from("Refreshing..."));
        }
        DisplayState::Ready => {}
        DisplayState::Stale(error) => {
            lines.push(Line::from(format!(
                "! Refresh failed — showing previous data ({})",
                error_message(error)
            )));
        }
        DisplayState::Unavailable(error) => {
            lines.push(Line::from(format!("! {}", error_message(error))));
        }
    }

    lines
}

fn error_message(error: &ProviderError) -> &'static str {
    match error {
        ProviderError::CliNotFound => "CLI not found",
        ProviderError::CredentialsNotFound => "Credentials not found — run provider login",
        ProviderError::NotAuthenticated => "Not authenticated — run provider login",
        ProviderError::Timeout => "Provider timed out",
        ProviderError::Network => "Network request failed",
        ProviderError::CommandFailed => "Provider command failed",
        ProviderError::ParseError => "Provider output could not be parsed",
        ProviderError::UnsupportedOutput => "Provider output format is unsupported",
    }
}
