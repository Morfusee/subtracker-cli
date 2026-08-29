pub mod format;
pub mod provider_card;
pub mod quota_bar;
pub mod theme;

use chrono::{DateTime, Utc};
use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, Paragraph},
};

use crate::{app::App, model::ProviderId};

use theme::Theme;

pub const MIN_WIDTH: u16 = 60;
pub const MIN_HEIGHT: u16 = 20;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum LayoutMode {
    Wide,
    Compact,
    Narrow,
}

impl LayoutMode {
    pub const fn for_width(width: u16) -> Self {
        if width >= 100 {
            Self::Wide
        } else if width >= 70 {
            Self::Compact
        } else {
            Self::Narrow
        }
    }
}

pub fn render(frame: &mut Frame, app: &App, now: DateTime<Utc>, spinner_frame: u8, theme: Theme) {
    let area = frame.area();

    if area.width < MIN_WIDTH || area.height < MIN_HEIGHT {
        render_minimum_size_message(frame, area, theme);
        return;
    }

    let mode = LayoutMode::for_width(area.width);
    let content_width = area.width.min(120);
    let inner_width = content_width.saturating_sub(8);

    let cards = ProviderId::ALL.map(|id| {
        let lines = provider_card::content_lines(
            id,
            app.provider(id),
            mode,
            inner_width,
            theme,
            now,
            spinner_frame,
        );
        (id, lines)
    });

    let heights = cards.each_ref().map(|(_, lines)| {
        u16::try_from(lines.len())
            .unwrap_or(u16::MAX)
            .saturating_add(2)
    });

    let cards_required_height = heights
        .into_iter()
        .fold(0u16, u16::saturating_add)
        .saturating_add(3) // three one-row gaps (between cards + before footer)
        .saturating_add(1); // footer

    if cards_required_height > area.height {
        render_content_too_tall_message(frame, area, theme);
        return;
    }

    let show_header = area.height >= cards_required_height.saturating_add(7);
    let total_required_height = if show_header {
        cards_required_height.saturating_add(7)
    } else {
        cards_required_height
    };

    let h_offset = (area.width.saturating_sub(content_width)) / 2;
    let v_offset = (area.height.saturating_sub(total_required_height)) / 2;

    let centered_area = Rect {
        x: area.x + h_offset,
        y: area.y + v_offset,
        width: content_width,
        height: total_required_height,
    };

    let mut constraints = Vec::new();
    if show_header {
        constraints.push(Constraint::Length(6)); // 0: Header
        constraints.push(Constraint::Length(1)); // 1: Gap
    }
    constraints.push(Constraint::Length(heights[0])); // Card 0
    constraints.push(Constraint::Length(1)); // Gap
    constraints.push(Constraint::Length(heights[1])); // Card 1
    constraints.push(Constraint::Length(1)); // Gap
    constraints.push(Constraint::Length(heights[2])); // Card 2
    constraints.push(Constraint::Length(1)); // Gap
    constraints.push(Constraint::Length(1)); // Footer

    let areas = Layout::default()
        .direction(Direction::Vertical)
        .constraints(constraints)
        .split(centered_area);

    let offset = if show_header {
        frame.render_widget(header(theme), areas[0]);
        2
    } else {
        0
    };

    for (card_index, area_index) in [(0, offset), (1, offset + 2), (2, offset + 4)] {
        let (id, lines) = &cards[card_index];
        let provider_state = app.provider(*id);

        let top_title = Line::from(vec![
            Span::raw("──  "),
            Span::styled(id.display_name(), theme.provider_title(*id)),
            Span::raw("  "),
        ]);

        let mut block = Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .border_style(theme.provider_border(*id))
            .padding(ratatui::widgets::Padding::horizontal(3))
            .title(top_title);

        if let Some(status) =
            provider_card::status_title(*id, provider_state, theme, now, spinner_frame)
        {
            block = block.title_bottom(status.alignment(Alignment::Right));
        }

        frame.render_widget(
            Paragraph::new(lines.clone()).block(block),
            areas[area_index],
        );
    }

    frame.render_widget(footer(mode, theme), areas[offset + 6]);
}

fn header(theme: Theme) -> Paragraph<'static> {
    let logo_style = theme.provider_border(ProviderId::Codex);
    let lines = vec![
        Line::from(Span::styled(
            "███████╗████████╗ ██████╗                   ",
            logo_style,
        )),
        Line::from(vec![
            Span::styled("██╔════╝╚══██╔══╝██╔════╝   ", logo_style),
            Span::styled(
                "SUBTRACKER      ",
                theme.primary().add_modifier(ratatui::style::Modifier::BOLD),
            ),
        ]),
        Line::from(vec![
            Span::styled("███████╗   ██║   ██║        ", logo_style),
            Span::styled("AI Usage Monitor", theme.secondary()),
        ]),
        Line::from(Span::styled(
            "╚════██║   ██║   ██║                        ",
            logo_style,
        )),
        Line::from(Span::styled(
            "███████║   ██║   ╚██████╗                   ",
            logo_style,
        )),
        Line::from(Span::styled(
            "╚══════╝   ╚═╝    ╚═════╝                   ",
            logo_style,
        )),
    ];

    Paragraph::new(lines).alignment(Alignment::Center)
}

fn footer(mode: LayoutMode, theme: Theme) -> Paragraph<'static> {
    let spans = match mode {
        LayoutMode::Wide => vec![
            Span::styled("[r]", theme.primary()),
            Span::styled(" refresh        ", theme.secondary()),
            Span::styled("◷ auto 60s        ", theme.secondary()),
            Span::styled("[q]", theme.primary()),
            Span::styled(" quit        ", theme.secondary()),
            Span::styled("[Ctrl+C]", theme.primary()),
            Span::styled(" exit", theme.secondary()),
        ],
        LayoutMode::Compact => vec![
            Span::styled("[r]", theme.primary()),
            Span::styled(" refresh   ", theme.secondary()),
            Span::styled("60s auto   ", theme.secondary()),
            Span::styled("[q]", theme.primary()),
            Span::styled(" quit   ", theme.secondary()),
            Span::styled("[Ctrl+C]", theme.primary()),
            Span::styled(" exit", theme.secondary()),
        ],
        LayoutMode::Narrow => vec![
            Span::styled("[r]", theme.primary()),
            Span::styled(" refresh   ", theme.secondary()),
            Span::styled("[q]", theme.primary()),
            Span::styled(" quit   ", theme.secondary()),
            Span::styled("[^C]", theme.primary()),
            Span::styled(" exit", theme.secondary()),
        ],
    };

    Paragraph::new(Line::from(spans)).alignment(Alignment::Center)
}

fn render_minimum_size_message(frame: &mut Frame, area: Rect, theme: Theme) {
    render_center_message(
        frame,
        area,
        theme,
        vec![
            "Subtracker".into(),
            "".into(),
            "Terminal too small.".into(),
            "Resize to at least 60x20.".into(),
        ],
    );
}

fn render_content_too_tall_message(frame: &mut Frame, area: Rect, theme: Theme) {
    render_center_message(
        frame,
        area,
        theme,
        vec![
            "Subtracker".into(),
            "".into(),
            "Terminal too small for current provider data.".into(),
            "Increase terminal height.".into(),
        ],
    );
}

fn render_center_message(frame: &mut Frame, area: Rect, theme: Theme, lines: Vec<String>) {
    let message_height = u16::try_from(lines.len()).unwrap_or(4).min(area.height);

    let vertical = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Min(0),
            Constraint::Length(message_height),
            Constraint::Min(0),
        ])
        .split(area);

    let lines = lines
        .into_iter()
        .map(|line| Line::from(Span::styled(line, theme.primary())).alignment(Alignment::Center))
        .collect::<Vec<_>>();

    frame.render_widget(Paragraph::new(lines), vertical[1]);
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::{TimeZone, Utc};
    use ratatui::{Terminal, backend::TestBackend};

    use crate::{
        app::App,
        model::{ProviderId, ProviderSnapshot, QuotaWindow, UsageStat, UsageValue},
        providers::ProviderError,
        ui::theme::{ColorMode, Theme},
    };

    fn buffer_text(terminal: &Terminal<TestBackend>) -> String {
        let buffer = terminal.backend().buffer();
        (0..buffer.area.height)
            .map(|y| {
                (0..buffer.area.width)
                    .map(|x| buffer[(x, y)].symbol())
                    .collect::<String>()
            })
            .collect::<Vec<_>>()
            .join("\n")
    }

    fn ready_app(now: chrono::DateTime<Utc>) -> App {
        let mut app = App::new();
        app.request_refresh();

        app.finish_refresh(
            ProviderId::Codex,
            Ok(ProviderSnapshot {
                provider: ProviderId::Codex,
                account_label: None,
                quotas: vec![
                    QuotaWindow {
                        label: "5 hour".into(),
                        used_percent: Some(35.0),
                        remaining_percent: Some(65.0),
                        resets_at: Some(now + chrono::Duration::hours(1)),
                        window_seconds: Some(18_000),
                    },
                    QuotaWindow {
                        label: "Weekly".into(),
                        used_percent: Some(21.0),
                        remaining_percent: Some(79.0),
                        resets_at: Some(now + chrono::Duration::days(6)),
                        window_seconds: Some(604_800),
                    },
                ],
                stats: Vec::new(),
                fetched_at: now,
            }),
        );

        app.finish_refresh(
            ProviderId::OpenCode,
            Ok(ProviderSnapshot {
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
            }),
        );

        app.finish_refresh(
            ProviderId::Antigravity,
            Ok(ProviderSnapshot {
                provider: ProviderId::Antigravity,
                account_label: None,
                quotas: vec![
                    QuotaWindow {
                        label: "Gemini 5 hour".into(),
                        used_percent: Some(55.0),
                        remaining_percent: Some(45.0),
                        resets_at: Some(now + chrono::Duration::minutes(3)),
                        window_seconds: Some(18_000),
                    },
                    QuotaWindow {
                        label: "Gemini weekly".into(),
                        used_percent: Some(10.0),
                        remaining_percent: Some(90.0),
                        resets_at: Some(now + chrono::Duration::days(6)),
                        window_seconds: Some(604_800),
                    },
                    QuotaWindow {
                        label: "Claude/GPT 5 hour".into(),
                        used_percent: Some(0.0),
                        remaining_percent: Some(100.0),
                        resets_at: Some(now + chrono::Duration::hours(5)),
                        window_seconds: Some(18_000),
                    },
                    QuotaWindow {
                        label: "Claude/GPT weekly".into(),
                        used_percent: Some(0.0),
                        remaining_percent: Some(100.0),
                        resets_at: Some(now + chrono::Duration::days(7)),
                        window_seconds: Some(604_800),
                    },
                ],
                stats: Vec::new(),
                fetched_at: now,
            }),
        );

        app
    }

    #[test]
    fn layout_breakpoints_match_the_design() {
        assert_eq!(LayoutMode::for_width(69), LayoutMode::Narrow);
        assert_eq!(LayoutMode::for_width(70), LayoutMode::Compact);
        assert_eq!(LayoutMode::for_width(99), LayoutMode::Compact);
        assert_eq!(LayoutMode::for_width(100), LayoutMode::Wide);
    }

    #[test]
    fn wide_dashboard_has_three_cards_bars_stats_and_full_footer() {
        let now = Utc.timestamp_opt(1_788_000_000, 0).single().unwrap();
        let backend = TestBackend::new(120, 32);
        let mut terminal = Terminal::new(backend).unwrap();
        let app = ready_app(now);

        terminal
            .draw(|frame| {
                render(frame, &app, now, 0, Theme::new(ColorMode::None));
            })
            .unwrap();

        let text = buffer_text(&terminal);

        assert!(text.contains("CODEX"));
        assert!(text.contains("OPENCODE"));
        assert!(text.contains("ANTIGRAVITY"));
        assert!(text.contains("▓▓▓▓"));
        assert!(text.contains("65%"));
        assert!(text.contains("312.3M tokens"));
        assert!(text.contains("remaining"));
        assert!(text.contains("[r] refresh"));
        assert!(text.contains("auto 60s"));
        assert!(text.contains("[q] quit"));
    }

    #[test]
    fn narrow_dashboard_uses_multiline_quota_copy_and_short_footer() {
        let now = Utc.timestamp_opt(1_788_000_000, 0).single().unwrap();
        let backend = TestBackend::new(65, 42);
        let mut terminal = Terminal::new(backend).unwrap();
        let app = ready_app(now);

        terminal
            .draw(|frame| {
                render(frame, &app, now, 0, Theme::new(ColorMode::None));
            })
            .unwrap();

        let text = buffer_text(&terminal);

        assert!(text.contains("resets in"));
        assert!(text.contains("[r] refresh"));
        assert!(text.contains("[q] quit"));
        assert!(!text.contains("auto 60s"));
    }

    #[test]
    fn mixed_stale_and_unavailable_states_share_one_dashboard() {
        let now = Utc.timestamp_opt(1_788_000_000, 0).single().unwrap();
        let mut app = App::new();
        app.request_refresh();
        app.finish_refresh(
            ProviderId::Codex,
            Ok(ProviderSnapshot {
                provider: ProviderId::Codex,
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
            }),
        );
        app.finish_refresh(
            ProviderId::Antigravity,
            Ok(ProviderSnapshot {
                provider: ProviderId::Antigravity,
                account_label: None,
                quotas: vec![QuotaWindow {
                    label: "Gemini 5 hour".into(),
                    used_percent: Some(55.0),
                    remaining_percent: Some(45.0),
                    resets_at: Some(now + chrono::Duration::minutes(3)),
                    window_seconds: Some(18_000),
                }],
                stats: Vec::new(),
                fetched_at: now,
            }),
        );
        app.finish_refresh(ProviderId::OpenCode, Err(ProviderError::CliNotFound));

        // Now refresh Codex again and fail it, making it Stale
        app.request_refresh();
        app.finish_refresh(ProviderId::Codex, Err(ProviderError::Network));
        app.finish_refresh(
            ProviderId::Antigravity,
            Ok(ProviderSnapshot {
                provider: ProviderId::Antigravity,
                account_label: None,
                quotas: vec![QuotaWindow {
                    label: "Gemini 5 hour".into(),
                    used_percent: Some(55.0),
                    remaining_percent: Some(45.0),
                    resets_at: Some(now + chrono::Duration::minutes(3)),
                    window_seconds: Some(18_000),
                }],
                stats: Vec::new(),
                fetched_at: now,
            }),
        );
        app.finish_refresh(ProviderId::OpenCode, Err(ProviderError::CliNotFound));

        let backend = TestBackend::new(120, 32);
        let mut terminal = Terminal::new(backend).unwrap();

        terminal
            .draw(|frame| {
                render(
                    frame,
                    &app,
                    now + chrono::Duration::minutes(3),
                    0,
                    Theme::new(ColorMode::None),
                );
            })
            .unwrap();

        let text = buffer_text(&terminal);

        assert!(text.contains("stale"));
        assert!(text.contains("updated 3m ago"));
        assert!(text.contains("OPENCODE"));
        assert!(text.contains("CLI not found"));
    }

    #[test]
    fn absolute_minimum_size_has_deliberate_fallback_copy() {
        let now = Utc.timestamp_opt(1_788_000_000, 0).single().unwrap();
        let backend = TestBackend::new(50, 15);
        let mut terminal = Terminal::new(backend).unwrap();
        let app = ready_app(now);

        terminal
            .draw(|frame| {
                render(frame, &app, now, 0, Theme::new(ColorMode::None));
            })
            .unwrap();

        let text = buffer_text(&terminal);

        assert!(text.contains("Terminal too small"));
        assert!(text.contains("60x20"));
    }
}
