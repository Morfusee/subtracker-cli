use chrono::{DateTime, Utc};

use crate::model::UsageValue;

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
        format!("{}m", minutes.max(1))
    }
}

pub fn format_age(fetched_at: DateTime<Utc>, now: DateTime<Utc>) -> String {
    let seconds = (now - fetched_at).num_seconds().max(0);

    if seconds < 60 {
        "just now".into()
    } else if seconds < 3_600 {
        format!("{}m ago", seconds / 60)
    } else if seconds < 86_400 {
        format!("{}h ago", seconds / 3_600)
    } else {
        format!("{}d ago", seconds / 86_400)
    }
}

pub fn format_usage_value(value: &UsageValue) -> String {
    match value {
        UsageValue::Count(value) => group_digits(*value),
        UsageValue::Tokens(value) => {
            format!("{} tokens", compact_number(*value))
        }
        UsageValue::MoneyCents(cents) => {
            let sign = if *cents < 0 { "-" } else { "" };
            let absolute = cents.unsigned_abs();
            format!("{sign}${}.{:02}", absolute / 100, absolute % 100)
        }
        UsageValue::Text(value) => value.clone(),
    }
}

fn compact_number(value: u64) -> String {
    match value {
        1_000_000_000.. => trim_suffix(format!("{:.1}", value as f64 / 1_000_000_000.0), "B"),
        1_000_000.. => trim_suffix(format!("{:.1}", value as f64 / 1_000_000.0), "M"),
        1_000.. => trim_suffix(format!("{:.1}", value as f64 / 1_000.0), "K"),
        _ => value.to_string(),
    }
}

fn trim_suffix(number: String, suffix: &str) -> String {
    let number = number.strip_suffix(".0").unwrap_or(&number);
    format!("{number}{suffix}")
}

fn group_digits(value: u64) -> String {
    let digits = value.to_string();
    let mut grouped = String::new();

    for (index, character) in digits.chars().enumerate() {
        if index > 0 && (digits.len() - index).is_multiple_of(3) {
            grouped.push(',');
        }
        grouped.push(character);
    }

    grouped
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::UsageValue;
    use chrono::{TimeZone, Utc};

    #[test]
    fn reset_time_is_compact_and_relative() {
        let now = Utc.timestamp_opt(1_788_000_000, 0).single().unwrap();

        assert_eq!(
            format_reset(
                now + chrono::Duration::hours(2) + chrono::Duration::minutes(17),
                now
            ),
            "2h 17m"
        );
        assert_eq!(
            format_reset(
                now + chrono::Duration::days(6) + chrono::Duration::hours(5),
                now
            ),
            "6d 5h"
        );
    }

    #[test]
    fn update_age_uses_short_status_copy() {
        let now = Utc.timestamp_opt(1_788_000_000, 0).single().unwrap();

        assert_eq!(format_age(now, now), "just now");
        assert_eq!(
            format_age(now - chrono::Duration::minutes(3), now),
            "3m ago"
        );
    }

    #[test]
    fn usage_values_match_the_designed_open_code_card() {
        assert_eq!(format_usage_value(&UsageValue::Count(2_277)), "2,277");
        assert_eq!(
            format_usage_value(&UsageValue::Tokens(312_300_000)),
            "312.3M tokens"
        );
        assert_eq!(
            format_usage_value(&UsageValue::MoneyCents(12_050)),
            "$120.50"
        );
    }
}
