use chrono::{DateTime, Utc};
use ratatui::{
    layout::Alignment,
    text::{Line, Span},
};

use crate::{
    app::{DisplayState, ProviderState},
    model::{ProviderId, ProviderSnapshot},
    providers::ProviderError,
};

use super::{
    Density, LayoutMode,
    format::{format_age, format_reset, format_usage_value},
    quota_bar::QuotaBar,
    theme::Theme,
};

const SPINNER: [&str; 4] = ["◐", "◓", "◑", "◒"];

pub fn spinner_symbol(frame: u8) -> &'static str {
    SPINNER[usize::from(frame) % SPINNER.len()]
}

pub fn content_lines(
    id: ProviderId,
    state: &ProviderState,
    mode: LayoutMode,
    inner_width: u16,
    density: Density,
    theme: Theme,
    now: DateTime<Utc>,
    spinner_frame: u8,
) -> Vec<Line<'static>> {
    if state.snapshot.is_none() {
        return empty_state_lines(id, &state.display, theme, spinner_frame);
    }

    let snapshot = state.snapshot.as_ref().expect("snapshot checked above");

    match id {
        ProviderId::Codex | ProviderId::Antigravity => {
            quota_lines(snapshot, mode, inner_width, density, theme, now)
        }
        ProviderId::OpenCode => {
            if !snapshot.quotas.is_empty() {
                quota_lines(snapshot, mode, inner_width, density, theme, now)
            } else {
                open_code_lines(snapshot, mode, inner_width, density, theme)
            }
        }
    }
}

pub fn status_title(
    id: ProviderId,
    state: &ProviderState,
    theme: Theme,
    now: DateTime<Utc>,
    spinner_frame: u8,
) -> Option<Line<'static>> {
    let snapshot = state.snapshot.as_ref()?;
    let fetched_at = snapshot.fetched_at;

    let line = match &state.display {
        DisplayState::Ready => Line::from(vec![
            Span::raw("  "),
            Span::styled("● ", theme.provider_border(id)),
            Span::styled(
                format!("updated {}", format_age(fetched_at, now)),
                theme.secondary(),
            ),
            Span::raw("  ──"),
        ]),

        DisplayState::Refreshing => Line::from(vec![
            Span::raw("  "),
            Span::styled(
                format!("{} ", spinner_symbol(spinner_frame)),
                theme.provider_border(id),
            ),
            Span::styled("refreshing…", theme.secondary()),
            Span::raw("  ──"),
        ]),

        DisplayState::Stale(_) => Line::from(vec![
            Span::raw("  "),
            Span::styled("● ", theme.provider_border(id)),
            Span::styled(
                format!("updated {}   ", format_age(fetched_at, now)),
                theme.secondary(),
            ),
            Span::styled("⚠ stale", theme.warning()),
            Span::raw("  ──"),
        ]),

        DisplayState::Unavailable(_) | DisplayState::Loading => return None,
    };

    Some(line)
}

fn quota_lines(
    snapshot: &ProviderSnapshot,
    mode: LayoutMode,
    inner_width: u16,
    density: Density,
    theme: Theme,
    now: DateTime<Utc>,
) -> Vec<Line<'static>> {
    let mut lines = Vec::new();

    let filtered: Vec<&crate::model::QuotaWindow> = snapshot
        .quotas
        .iter()
        .filter(|q| q.remaining_percent.is_some())
        .collect();

    if filtered.is_empty() {
        return lines;
    }

    if density == Density::Normal {
        lines.push(Line::from("")); // Top interior padding
    }

    let pad_indent = match density {
        Density::Dense => 2,
        _ => 4,
    };
    let pad_str = " ".repeat(pad_indent);

    for (q_idx, quota) in filtered.iter().enumerate() {
        let remaining = quota.remaining_percent.expect("filtered");
        let is_last = q_idx + 1 == filtered.len();

        match mode {
            LayoutMode::Wide => {
                let bar_width = match density {
                    Density::Normal => inner_width.saturating_sub(48).clamp(16, 60),
                    Density::Compact | Density::Spaced => {
                        inner_width.saturating_sub(46).clamp(12, 40)
                    }
                    Density::Dense => inner_width.saturating_sub(42).clamp(6, 24),
                };
                let mut spans = vec![
                    Span::raw(pad_str.clone()),
                    Span::styled(format!("{:<18}", quota.label), theme.primary()),
                    Span::raw(" "),
                ];
                spans.extend(QuotaBar::new(remaining).spans(bar_width, theme));
                if let Some(reset) = quota.resets_at {
                    spans.push(Span::styled(
                        format!("   ◷ {}", format_reset(reset, now)),
                        theme.secondary(),
                    ));
                }
                lines.push(Line::from(spans));
            }
            LayoutMode::Compact => {
                let bar_width = match density {
                    Density::Normal => inner_width.saturating_sub(44).clamp(12, 28),
                    Density::Compact | Density::Spaced => {
                        inner_width.saturating_sub(42).clamp(8, 20)
                    }
                    Density::Dense => inner_width.saturating_sub(38).clamp(4, 14),
                };
                let mut spans = vec![
                    Span::raw(pad_str.clone()),
                    Span::styled(format!("{:<18}", quota.label), theme.primary()),
                    Span::raw(" "),
                ];
                spans.extend(QuotaBar::new(remaining).spans(bar_width, theme));
                if let Some(reset) = quota.resets_at {
                    spans.push(Span::styled(
                        format!("   ◷ {}", format_reset(reset, now)),
                        theme.secondary(),
                    ));
                }
                lines.push(Line::from(spans));
            }
            LayoutMode::Narrow => {
                let bar_width = match density {
                    Density::Normal => inner_width.saturating_sub(26).clamp(8, 24),
                    Density::Compact | Density::Spaced => {
                        inner_width.saturating_sub(26).clamp(6, 20)
                    }
                    Density::Dense => inner_width.saturating_sub(24).clamp(4, 12),
                };
                lines.push(Line::from(vec![
                    Span::raw(pad_str.clone()),
                    Span::styled(quota.label.clone(), theme.primary()),
                ]));
                let mut line2_spans = vec![Span::raw(pad_str.clone())];
                line2_spans.extend(QuotaBar::new(remaining).spans(bar_width, theme));
                if let Some(reset) = quota.resets_at {
                    line2_spans.push(Span::styled(
                        format!("  resets in {}", format_reset(reset, now)),
                        theme.secondary(),
                    ));
                }
                lines.push(Line::from(line2_spans));
            }
        }

        if !is_last && density != Density::Dense {
            lines.push(Line::from("")); // Spacing between each label-bar element
        }
    }

    if density == Density::Normal {
        lines.push(Line::from("")); // Bottom interior padding
    }

    lines
}

fn open_code_lines(
    snapshot: &ProviderSnapshot,
    mode: LayoutMode,
    inner_width: u16,
    density: Density,
    theme: Theme,
) -> Vec<Line<'static>> {
    let sessions = stat_value(snapshot, "Sessions");
    let total_cost = stat_value(snapshot, "Total Cost");
    let input = stat_value(snapshot, "Input");
    let output = stat_value(snapshot, "Output");

    let mut lines = Vec::new();
    if density == Density::Normal {
        lines.push(Line::from("")); // Top interior padding
    }

    match mode {
        LayoutMode::Wide => {
            lines.push(stat_grid_line(
                "Sessions", &sessions, "Input", &input, theme,
            ));
            lines.push(stat_grid_line(
                "Total Cost",
                &total_cost,
                "Output",
                &output,
                theme,
            ));
        }
        LayoutMode::Compact => {
            if density != Density::Normal && inner_width >= 56 {
                lines.push(stat_grid_line(
                    "Sessions", &sessions, "Input", &input, theme,
                ));
                lines.push(stat_grid_line(
                    "Total Cost",
                    &total_cost,
                    "Output",
                    &output,
                    theme,
                ));
            } else {
                let rows = [
                    ("Sessions", sessions.as_str()),
                    ("Total Cost", total_cost.as_str()),
                    ("Input", input.as_str()),
                    ("Output", output.as_str()),
                ];

                let raw: Vec<Vec<Span<'static>>> = rows
                    .into_iter()
                    .map(|(label, value)| {
                        vec![
                            Span::styled(format!("{label:<14}"), theme.secondary()),
                            Span::styled(value.to_owned(), theme.primary()),
                        ]
                    })
                    .collect();

                let max_width = raw
                    .iter()
                    .map(|spans| spans.iter().map(|s| s.width() as u16).sum::<u16>())
                    .max()
                    .unwrap_or(0);
                let pad = inner_width.saturating_sub(max_width) / 2;
                let pad_str = " ".repeat(pad as usize);

                for spans in raw {
                    let mut padded = Vec::with_capacity(spans.len() + 1);
                    if pad > 0 {
                        padded.push(Span::raw(pad_str.clone()));
                    }
                    padded.extend(spans);
                    lines.push(Line::from(padded));
                }
            }
        }
        LayoutMode::Narrow => {
            if density == Density::Dense && inner_width >= 56 {
                lines.push(stat_grid_line(
                    "Sessions", &sessions, "Input", &input, theme,
                ));
                lines.push(stat_grid_line(
                    "Total Cost",
                    &total_cost,
                    "Output",
                    &output,
                    theme,
                ));
            } else {
                let pad_indent = match density {
                    Density::Dense => 2,
                    _ => 4,
                };
                let pad_str = " ".repeat(pad_indent);
                lines.push(stat_line_with_pad("Sessions", &sessions, &pad_str, theme));
                lines.push(stat_line_with_pad(
                    "Total Cost",
                    &total_cost,
                    &pad_str,
                    theme,
                ));
                lines.push(stat_line_with_pad("Input", &input, &pad_str, theme));
                lines.push(stat_line_with_pad("Output", &output, &pad_str, theme));
            }
        }
    }

    if density == Density::Normal {
        lines.push(Line::from("")); // Bottom interior padding
    }

    lines
}

fn stat_line_with_pad(label: &str, value: &str, pad: &str, theme: Theme) -> Line<'static> {
    Line::from(vec![
        Span::raw(pad.to_owned()),
        Span::styled(format!("{label:<12}"), theme.secondary()),
        Span::styled(value.to_owned(), theme.primary()),
    ])
}

fn stat_value(snapshot: &ProviderSnapshot, label: &str) -> String {
    snapshot
        .stats
        .iter()
        .find(|stat| stat.label == label)
        .map(|stat| format_usage_value(&stat.value))
        .unwrap_or_else(|| "—".into())
}

fn stat_grid_line(
    left_label: &str,
    left_value: &str,
    right_label: &str,
    right_value: &str,
    theme: Theme,
) -> Line<'static> {
    Line::from(vec![
        Span::raw("    "),
        Span::styled(format!("{left_label:<12}"), theme.secondary()),
        Span::styled(format!("{left_value:<18}"), theme.primary()),
        Span::styled("│  ", theme.secondary()),
        Span::styled(format!("{right_label:<10}"), theme.secondary()),
        Span::styled(right_value.to_owned(), theme.primary()),
    ])
}

fn empty_state_lines(
    id: ProviderId,
    display: &DisplayState,
    theme: Theme,
    spinner_frame: u8,
) -> Vec<Line<'static>> {
    match display {
        DisplayState::Loading | DisplayState::Refreshing => vec![
            Line::from(""),
            Line::from(vec![
                Span::styled(
                    format!("{} ", spinner_symbol(spinner_frame)),
                    theme.provider_border(id),
                ),
                Span::styled("Loading…", theme.secondary()),
            ])
            .alignment(Alignment::Center),
            Line::from(""),
        ],

        DisplayState::Unavailable(error) | DisplayState::Stale(error) => {
            let (title, hint) = unavailable_copy(id, error);

            vec![
                Line::from(""),
                Line::from(Span::styled(title, theme.error())).alignment(Alignment::Center),
                Line::from(Span::styled(hint, theme.secondary())).alignment(Alignment::Center),
                Line::from(""),
            ]
        }

        DisplayState::Ready => vec![
            Line::from(Span::styled("No provider data", theme.secondary()))
                .alignment(Alignment::Center),
        ],
    }
}

fn unavailable_copy(id: ProviderId, error: &ProviderError) -> (String, String) {
    match (id, error) {
        (
            ProviderId::Codex,
            ProviderError::NotAuthenticated | ProviderError::CredentialsNotFound,
        ) => ("⚠  Session unavailable".into(), "Run `codex login`".into()),
        (ProviderId::OpenCode, ProviderError::CliNotFound) => {
            ("◌  CLI not found".into(), "`opencode`".into())
        }
        (
            ProviderId::OpenCode,
            ProviderError::NotAuthenticated | ProviderError::CredentialsNotFound,
        ) => (
            "⚠  Not authenticated".into(),
            "Connect account in OpenCode".into(),
        ),
        (ProviderId::Antigravity, ProviderError::CliNotFound) => {
            ("◌  CLI not found".into(), "`agy`".into())
        }
        (ProviderId::Antigravity, ProviderError::NotAuthenticated) => (
            "⚠  Not authenticated".into(),
            "Run `agy` and sign in".into(),
        ),
        _ => (
            format!("⚠  {}", error_title(error)),
            "Press `r` to retry".into(),
        ),
    }
}

fn error_title(error: &ProviderError) -> &'static str {
    match error {
        ProviderError::CliNotFound => "CLI not found",
        ProviderError::CredentialsNotFound => "Credentials not found",
        ProviderError::NotAuthenticated => "Not authenticated",
        ProviderError::Timeout => "Provider timed out",
        ProviderError::Network => "Network request failed",
        ProviderError::CommandFailed => "Provider command failed",
        ProviderError::ParseError => "Provider output could not be parsed",
        ProviderError::UnsupportedOutput => "Provider output format is unsupported",
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;

    use crate::{
        app::{App, DisplayState},
        model::{ProviderId, ProviderSnapshot, QuotaWindow, UsageStat, UsageValue},
        providers::ProviderError,
        ui::theme::{ColorMode, Theme},
    };

    fn plain(lines: &[Line<'_>]) -> String {
        lines
            .iter()
            .map(|line| {
                line.spans
                    .iter()
                    .map(|span| span.content.as_ref())
                    .collect::<String>()
            })
            .collect::<Vec<_>>()
            .join("\n")
    }

    fn quota_snapshot(provider: ProviderId, now: DateTime<Utc>) -> ProviderSnapshot {
        ProviderSnapshot {
            provider,
            account_label: None,
            quotas: vec![QuotaWindow {
                label: "5 hour".into(),
                used_percent: Some(35.0),
                remaining_percent: Some(65.0),
                resets_at: Some(now + chrono::Duration::hours(1)),
                window_seconds: Some(18_000),
            }],
            stats: Vec::new(),
            fetched_at: now,
        }
    }

    #[test]
    fn spinner_cycles_through_four_stable_frames() {
        assert_eq!(spinner_symbol(0), "◐");
        assert_eq!(spinner_symbol(1), "◓");
        assert_eq!(spinner_symbol(2), "◑");
        assert_eq!(spinner_symbol(3), "◒");
        assert_eq!(spinner_symbol(4), "◐");
    }

    #[test]
    fn wide_quota_row_has_bar_percentage_remaining_and_reset() {
        let now = Utc.timestamp_opt(1_788_000_000, 0).single().unwrap();
        let mut app = App::new();
        app.request_refresh();
        app.finish_refresh(
            ProviderId::Codex,
            Ok(quota_snapshot(ProviderId::Codex, now)),
        );

        let lines = content_lines(
            ProviderId::Codex,
            app.provider(ProviderId::Codex),
            LayoutMode::Wide,
            110,
            Density::Normal,
            Theme::new(ColorMode::None),
            now,
            0,
        );
        let text = plain(&lines);

        assert!(text.contains("5 hour"));
        assert!(text.contains("65%"));
        assert!(!text.contains("remaining"));
        assert!(text.contains("◷ 1h 0m"));

        let status = status_title(
            ProviderId::Codex,
            app.provider(ProviderId::Codex),
            Theme::new(ColorMode::None),
            now,
            0,
        )
        .expect("status title present");
        let status_text = plain(&[status]);
        assert!(status_text.contains("● updated just now"));
    }

    #[test]
    fn narrow_quota_rows_use_multiline_reset_copy() {
        let now = Utc.timestamp_opt(1_788_000_000, 0).single().unwrap();
        let mut app = App::new();
        app.request_refresh();
        app.finish_refresh(
            ProviderId::Codex,
            Ok(quota_snapshot(ProviderId::Codex, now)),
        );

        let text = plain(&content_lines(
            ProviderId::Codex,
            app.provider(ProviderId::Codex),
            LayoutMode::Narrow,
            58,
            Density::Normal,
            Theme::new(ColorMode::None),
            now,
            0,
        ));

        assert!(text.contains("5 hour"));
        assert!(text.contains("65%"));
        assert!(text.contains("resets in 1h 0m"));
    }

    #[test]
    fn stale_state_keeps_values_and_adds_explicit_stale_copy() {
        let now = Utc.timestamp_opt(1_788_000_000, 0).single().unwrap();
        let mut app = App::new();
        app.request_refresh();
        app.finish_refresh(
            ProviderId::Codex,
            Ok(quota_snapshot(ProviderId::Codex, now)),
        );
        app.request_refresh();
        app.finish_refresh(ProviderId::Codex, Err(ProviderError::Network));

        assert_eq!(
            app.provider(ProviderId::Codex).display,
            DisplayState::Stale(ProviderError::Network)
        );

        let text = plain(&content_lines(
            ProviderId::Codex,
            app.provider(ProviderId::Codex),
            LayoutMode::Compact,
            80,
            Density::Normal,
            Theme::new(ColorMode::None),
            now + chrono::Duration::minutes(3),
            0,
        ));

        assert!(text.contains("65%"));

        let status = status_title(
            ProviderId::Codex,
            app.provider(ProviderId::Codex),
            Theme::new(ColorMode::None),
            now + chrono::Duration::minutes(3),
            0,
        )
        .expect("status title present");
        let status_text = plain(&[status]);
        assert!(status_text.contains("updated 3m ago"));
        assert!(status_text.contains("stale"));
    }

    #[test]
    fn first_load_codex_auth_failure_is_a_deliberate_empty_state() {
        let now = Utc.timestamp_opt(1_788_000_000, 0).single().unwrap();
        let mut app = App::new();
        app.request_refresh();
        app.finish_refresh(ProviderId::Codex, Err(ProviderError::NotAuthenticated));

        let text = plain(&content_lines(
            ProviderId::Codex,
            app.provider(ProviderId::Codex),
            LayoutMode::Wide,
            110,
            Density::Normal,
            Theme::new(ColorMode::None),
            now,
            0,
        ));

        assert!(text.contains("Session unavailable"));
        assert!(text.contains("Run `codex login`"));
        assert!(!text.contains("updated"));
    }

    #[test]
    fn open_code_wide_uses_two_columns_but_compact_stacks() {
        let now = Utc.timestamp_opt(1_788_000_000, 0).single().unwrap();
        let snapshot = ProviderSnapshot {
            provider: ProviderId::OpenCode,
            account_label: None,
            quotas: Vec::new(),
            stats: vec![
                UsageStat {
                    label: "Sessions".into(),
                    value: UsageValue::Count(2_277),
                },
                UsageStat {
                    label: "Total Cost".into(),
                    value: UsageValue::MoneyCents(12_050),
                },
                UsageStat {
                    label: "Input".into(),
                    value: UsageValue::Tokens(312_300_000),
                },
                UsageStat {
                    label: "Output".into(),
                    value: UsageValue::Tokens(15_300_000),
                },
            ],
            fetched_at: now,
        };

        let mut app = App::new();
        app.request_refresh();
        app.finish_refresh(ProviderId::OpenCode, Ok(snapshot));

        let wide = plain(&content_lines(
            ProviderId::OpenCode,
            app.provider(ProviderId::OpenCode),
            LayoutMode::Wide,
            110,
            Density::Normal,
            Theme::new(ColorMode::None),
            now,
            0,
        ));
        let compact = plain(&content_lines(
            ProviderId::OpenCode,
            app.provider(ProviderId::OpenCode),
            LayoutMode::Compact,
            80,
            Density::Normal,
            Theme::new(ColorMode::None),
            now,
            0,
        ));

        assert!(wide.contains("│"));
        assert!(!compact.contains("│"));
        assert!(compact.contains("Sessions"));
        assert!(compact.contains("Output"));
    }

    #[test]
    fn dense_quota_card_preserves_data_without_blank_spacing() {
        let now = Utc.timestamp_opt(1_788_000_000, 0).single().unwrap();
        let mut snapshot = quota_snapshot(ProviderId::Codex, now);
        snapshot.quotas.push(QuotaWindow {
            label: "Weekly".into(),
            used_percent: Some(21.0),
            remaining_percent: Some(79.0),
            resets_at: Some(now + chrono::Duration::days(6)),
            window_seconds: Some(604_800),
        });
        let mut app = App::new();
        app.request_refresh();
        app.finish_refresh(ProviderId::Codex, Ok(snapshot));

        let lines = content_lines(
            ProviderId::Codex,
            app.provider(ProviderId::Codex),
            LayoutMode::Wide,
            110,
            Density::Dense,
            Theme::new(ColorMode::None),
            now,
            0,
        );
        let text = plain(&lines);

        assert!(text.contains("5 hour"));
        assert!(text.contains("Weekly"));
        assert!(text.contains("65%"));
        assert!(text.contains("79%"));
        assert!(!text.contains("\n\n"));
    }

    #[test]
    fn quota_elements_have_spacing_and_border_caps() {
        let now = Utc.timestamp_opt(1_788_000_000, 0).single().unwrap();
        let mut snapshot = quota_snapshot(ProviderId::Codex, now);
        snapshot.quotas.push(QuotaWindow {
            label: "Weekly".into(),
            used_percent: Some(21.0),
            remaining_percent: Some(79.0),
            resets_at: Some(now + chrono::Duration::days(6)),
            window_seconds: Some(604_800),
        });
        let mut app = App::new();
        app.request_refresh();
        app.finish_refresh(ProviderId::Codex, Ok(snapshot));

        let lines = content_lines(
            ProviderId::Codex,
            app.provider(ProviderId::Codex),
            LayoutMode::Wide,
            110,
            Density::Normal,
            Theme::new(ColorMode::None),
            now,
            0,
        );
        let text = plain(&lines);

        assert!(
            text.contains("\n\n"),
            "must have vertical spacing between elements"
        );
        assert!(text.contains("│▓"), "bars must have vertical border caps");
        assert!(text.contains("5 hour"));
        assert!(text.contains("Weekly"));
    }
}
