use chrono::{DateTime, Utc};

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum ProviderId {
    Codex,
    OpenCode,
    Antigravity,
}

impl ProviderId {
    pub const ALL: [Self; 3] = [Self::Codex, Self::OpenCode, Self::Antigravity];

    pub const fn display_name(self) -> &'static str {
        match self {
            Self::Codex => "CODEX",
            Self::OpenCode => "OPENCODE",
            Self::Antigravity => "ANTIGRAVITY",
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct ProviderSnapshot {
    pub provider: ProviderId,
    pub account_label: Option<String>,
    pub quotas: Vec<QuotaWindow>,
    pub stats: Vec<UsageStat>,
    pub fetched_at: DateTime<Utc>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct QuotaWindow {
    pub label: String,
    pub used_percent: Option<f64>,
    pub remaining_percent: Option<f64>,
    pub resets_at: Option<DateTime<Utc>>,
    pub window_seconds: Option<u64>,
}

impl QuotaWindow {
    pub fn from_used_percent(
        label: impl Into<String>,
        used_percent: f64,
        resets_at: Option<DateTime<Utc>>,
        window_seconds: Option<u64>,
    ) -> Self {
        Self {
            label: label.into(),
            used_percent: Some(used_percent),
            remaining_percent: Some(100.0 - used_percent),
            resets_at,
            window_seconds,
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct UsageStat {
    pub label: String,
    pub value: UsageValue,
}

#[derive(Clone, Debug, PartialEq)]
pub enum UsageValue {
    Count(u64),
    Tokens(u64),
    MoneyCents(i64),
    Text(String),
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::{TimeZone, Utc};

    #[test]
    fn used_percent_is_converted_to_remaining_percent() {
        let reset = Utc.timestamp_opt(1_788_000_000, 0).single().unwrap();

        let window = QuotaWindow::from_used_percent("5 hour", 23.0, Some(reset), Some(18_000));

        assert_eq!(window.used_percent, Some(23.0));
        assert_eq!(window.remaining_percent, Some(77.0));
        assert_eq!(window.window_seconds, Some(18_000));
        assert_eq!(window.resets_at, Some(reset));
    }
}
