use chrono::{DateTime, Utc};

use crate::model::UsageValue;

pub fn quota_bar(remaining_percent: f64) -> String {
    let remaining = remaining_percent.clamp(0.0, 100.0);
    let filled = ((remaining / 100.0) * 10.0).round() as usize;
    let empty = 10usize.saturating_sub(filled);

    format!(
        "[{}{}] {:.0}% remaining",
        "#".repeat(filled),
        "-".repeat(empty),
        remaining
    )
}

pub fn format_reset(resets_at: DateTime<Utc>, now: DateTime<Utc>) -> String {
    let seconds = (resets_at - now).num_seconds();
    if seconds <= 0 {
        return "reset due".into();
    }

    let days = seconds / 86_400;
    let hours = (seconds % 86_400) / 3_600;
    let minutes = (seconds % 3_600) / 60;

    if days > 0 {
        format!("{days}d {hours}h")
    } else if hours > 0 {
        format!("{hours}h {minutes}m")
    } else {
        format!("{minutes}m")
    }
}

pub fn format_age(fetched_at: DateTime<Utc>, now: DateTime<Utc>) -> String {
    let seconds = (now - fetched_at).num_seconds().max(0);
    if seconds < 60 {
        "just now".into()
    } else if seconds < 3_600 {
        format!("{}m ago", seconds / 60)
    } else {
        format!("{}h ago", seconds / 3_600)
    }
}

pub fn format_usage_value(value: &UsageValue) -> String {
    match value {
        UsageValue::Count(value) => value.to_string(),
        UsageValue::Tokens(value) => {
            format!("{} tokens", compact_number(*value))
        }
        UsageValue::MoneyCents(cents) => {
            format!("${}.{:02}", cents / 100, cents.abs() % 100)
        }
        UsageValue::Text(value) => value.clone(),
    }
}

fn compact_number(value: u64) -> String {
    match value {
        1_000_000_000.. => format!("{:.1}B", value as f64 / 1_000_000_000.0).replace(".0B", "B"),
        1_000_000.. => format!("{:.1}M", value as f64 / 1_000_000.0).replace(".0M", "M"),
        1_000.. => format!("{:.1}K", value as f64 / 1_000.0).replace(".0K", "K"),
        _ => value.to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::UsageValue;
    use chrono::{TimeZone, Utc};

    #[test]
    fn quota_bar_shows_remaining_percentage() {
        assert_eq!(quota_bar(72.0), "[#######---] 72% remaining");
    }

    #[test]
    fn reset_time_is_relative_to_render_time() {
        let now = Utc.timestamp_opt(1_788_000_000, 0).single().unwrap();
        let reset = now + chrono::Duration::hours(2) + chrono::Duration::minutes(17);

        assert_eq!(format_reset(reset, now), "2h 17m");
    }

    #[test]
    fn usage_values_are_compact_and_human_readable() {
        assert_eq!(format_usage_value(&UsageValue::Count(42)), "42");
        assert_eq!(
            format_usage_value(&UsageValue::Tokens(599_000)),
            "599K tokens"
        );
        assert_eq!(format_usage_value(&UsageValue::MoneyCents(1234)), "$12.34");
    }
}
