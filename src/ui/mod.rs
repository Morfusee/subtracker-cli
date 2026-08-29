pub mod format;
pub mod provider_card;
pub mod quota_bar;
pub mod theme;

use chrono::{DateTime, Utc};
use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::Modifier,
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, Paragraph},
};

use crate::{app::App, model::ProviderId};

use theme::Theme;

pub const MIN_WIDTH: u16 = 60;
pub const MIN_HEIGHT: u16 = 20;

/// Formal layout phases. See `docs/ui-layout.md` for the agent-facing spec.
/// Shorthand: P3/W = `Wide` (>=100), P2/C = `Compact` (70–99, first breakpoint), P1/N = `Narrow` (<70).
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum LayoutMode {
    /// P3/W — desktop/wide, >=100 cols. Full fidelity; quota: label20 / bar (expanded `inner-33` clamp 20..80) / ◷ under bar, no `remaining`.
    Wide,
    /// P2/C — first breakpoint (70–99). Grid auto-fit/expand: each quota is flex-col (label/bar/◷) placed in `cols=(inner+gap)/(min+gap)` grid, `col_width` expands to fill. See `provider_card.rs:140`.
    Compact,
    /// P1/N — mobile/narrow, <70 cols. Single-column flex-col (label/bar/resets in) with indent.
    Narrow,
}

impl LayoutMode {
    /// Map terminal width to the formal phase. Thresholds are the media-query contract.
    pub const fn for_width(width: u16) -> Self {
        if width >= 100 {
            Self::Wide // P3/W
        } else if width >= 70 {
            Self::Compact // P2/C — first snap below W
        } else {
            Self::Narrow // P1/N
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Density {
    Normal,
    Compact,
    Spaced,
    Dense,
}

impl Density {
    pub const CANDIDATES: [Self; 4] = [Self::Normal, Self::Compact, Self::Spaced, Self::Dense];

    pub const fn card_padding(self) -> u16 {
        match self {
            Self::Normal => 3,
            Self::Compact | Self::Spaced => 2,
            Self::Dense => 1,
        }
    }

    pub const fn gap(self) -> u16 {
        match self {
            Self::Normal | Self::Compact | Self::Spaced => 1,
            Self::Dense => 0,
        }
    }

    pub const fn show_header(self) -> bool {
        match self {
            Self::Normal | Self::Compact => true,
            Self::Spaced | Self::Dense => false,
        }
    }
}

struct DashboardLayout {
    density: Density,
    content_width: u16,
    cards: [(ProviderId, Vec<Line<'static>>); 3],
    heights: [u16; 3],
    show_header: bool,
    required_height: u16,
}

impl DashboardLayout {
    fn new(
        app: &App,
        area: Rect,
        mode: LayoutMode,
        density: Density,
        theme: Theme,
        now: DateTime<Utc>,
        spinner_frame: u8,
    ) -> Self {
        let content_width = area.width.min(120);
        let inner_width = content_width.saturating_sub(2 + density.card_padding() * 2);
        let cx = provider_card::CardRenderContext {
            mode,
            inner_width,
            density,
            theme,
            now,
            spinner_frame,
        };
        let cards = ProviderId::ALL.map(|id| {
            let lines = if app.is_collapsed(id) {
                Vec::new()
            } else {
                provider_card::content_lines(id, app.provider(id), cx)
            };
            (id, lines)
        });
        let heights = cards.each_ref().map(|(_, lines)| {
            u16::try_from(lines.len())
                .unwrap_or(u16::MAX)
                .saturating_add(2)
        });
        let gaps = match density {
            Density::Normal => 4,  // 1 below header + 2 between cards + 1 before footer
            Density::Compact => 3, // 1 below header + 2 between cards
            Density::Spaced => 3,  // 2 between cards + 1 before footer
            Density::Dense => 0,
        };
        let cards_height = heights
            .iter()
            .copied()
            .fold(0u16, u16::saturating_add)
            .saturating_add(gaps)
            .saturating_add(1);
        let show_header = density.show_header();
        let header_height = if show_header { 6 } else { 0 };
        let required_height = cards_height.saturating_add(header_height);

        Self {
            density,
            content_width,
            cards,
            heights,
            show_header,
            required_height,
        }
    }

    fn fits(&self, area: Rect) -> bool {
        self.content_width <= area.width && self.required_height <= area.height
    }
}

pub fn render(frame: &mut Frame, app: &App, now: DateTime<Utc>, spinner_frame: u8, theme: Theme) {
    let area = frame.area();

    let mode = LayoutMode::for_width(area.width);
    let mut layouts = Density::CANDIDATES
        .map(|density| DashboardLayout::new(app, area, mode, density, theme, now, spinner_frame));
    let selected = layouts
        .iter()
        .position(|layout| layout.fits(area))
        .unwrap_or(layouts.len() - 1);
    let layout = &mut layouts[selected];

    let total_required_height = layout.required_height.min(area.height);
    let h_offset = (area.width.saturating_sub(layout.content_width)) / 2;
    let v_offset = (area.height.saturating_sub(total_required_height)) / 2;

    let centered_area = Rect {
        x: area.x + h_offset,
        y: area.y + v_offset,
        width: layout.content_width,
        height: total_required_height,
    };

    let mut constraints = Vec::new();
    if layout.show_header {
        constraints.push(Constraint::Length(6));
        if layout.density.gap() > 0 {
            constraints.push(Constraint::Length(1)); // Gap below logo
        }
    }

    let mut card_indices = [0usize; 3];
    for (index, height) in layout.heights.iter().copied().enumerate() {
        card_indices[index] = constraints.len();
        constraints.push(Constraint::Length(height));
        if layout.density.gap() > 0 && index + 1 < layout.heights.len() {
            constraints.push(Constraint::Length(layout.density.gap()));
        }
    }
    if layout.density == Density::Normal || layout.density == Density::Spaced {
        constraints.push(Constraint::Length(1)); // Gap before footer
    }
    let footer_index = constraints.len();
    constraints.push(Constraint::Length(1));

    let areas = Layout::default()
        .direction(Direction::Vertical)
        .constraints(constraints)
        .split(centered_area);

    if layout.show_header && !areas.is_empty() {
        let logo_width: u16 = 25;
        let logo_area = Rect {
            x: areas[0].x + areas[0].width.saturating_sub(logo_width) / 2,
            y: areas[0].y,
            width: logo_width.min(areas[0].width),
            height: 6.min(areas[0].height),
        };
        frame.render_widget(header(theme), logo_area);
    }

    for (card_index, &area_index) in card_indices.iter().enumerate() {
        if area_index >= areas.len() {
            continue;
        }
        let (id, lines) = &layout.cards[card_index];
        let provider_state = app.provider(*id);

        let focused = app.is_focused(*id);
        let collapsed = app.is_collapsed(*id);
        let focus_marker = if focused { "▸ " } else { "  " };
        let collapse_marker = if collapsed { "[+]" } else { "[-]" };
        let title_style = if focused {
            theme
                .provider_title(*id)
                .add_modifier(Modifier::REVERSED)
        } else {
            theme.provider_title(*id)
        };
        let border_style = if focused {
            theme
                .provider_border(*id)
                .add_modifier(Modifier::BOLD)
        } else {
            theme.provider_border(*id)
        };

        let top_title = Line::from(vec![
            Span::raw("──  "),
            Span::styled(
                format!("{focus_marker}{collapse_marker} {}", id.display_name()),
                title_style,
            ),
            Span::raw("  "),
        ]);

        let mut block = Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .border_style(border_style)
            .padding(ratatui::widgets::Padding::horizontal(
                layout.density.card_padding(),
            ))
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

    if footer_index < areas.len() {
        let footer_mode = match layout.density {
            Density::Dense => LayoutMode::Narrow,
            Density::Compact | Density::Spaced => {
                if mode == LayoutMode::Wide {
                    LayoutMode::Compact
                } else {
                    mode
                }
            }
            Density::Normal => mode,
        };
        frame.render_widget(footer(footer_mode, theme), areas[footer_index]);
    }
}

fn header(theme: Theme) -> Paragraph<'static> {
    let logo_style = theme.provider_border(ProviderId::Codex);
    // keep all 6 lines left-aligned within a centered block so the STC art stays aligned
    let lines = vec![
        Line::from(Span::styled("███████╗████████╗ ██████╗", logo_style)),
        Line::from(Span::styled("██╔════╝╚══██╔══╝██╔════╝", logo_style)),
        Line::from(Span::styled("███████╗   ██║   ██║", logo_style)),
        Line::from(Span::styled("╚════██║   ██║   ██║", logo_style)),
        Line::from(Span::styled("███████║   ██║   ╚██████╗", logo_style)),
        Line::from(Span::styled("╚══════╝   ╚═╝    ╚═════╝", logo_style)),
    ];

    Paragraph::new(lines) // left-aligned; centered as a block in render()
}

fn footer(mode: LayoutMode, theme: Theme) -> Paragraph<'static> {
    let spans = match mode {
        LayoutMode::Wide => vec![
            Span::styled("[j/k/↑/↓]", theme.primary()),
            Span::styled(" select   ", theme.secondary()),
            Span::styled("[Space/Enter]", theme.primary()),
            Span::styled(" collapse   ", theme.secondary()),
            Span::styled("[r]", theme.primary()),
            Span::styled(" refresh   ", theme.secondary()),
            Span::styled("[q]", theme.primary()),
            Span::styled(" quit", theme.secondary()),
        ],
        LayoutMode::Compact => vec![
            Span::styled("[j/k/↑/↓]", theme.primary()),
            Span::styled(" select  ", theme.secondary()),
            Span::styled("[Space/Enter]", theme.primary()),
            Span::styled(" toggle  ", theme.secondary()),
            Span::styled("[r]", theme.primary()),
            Span::styled(" refresh  ", theme.secondary()),
            Span::styled("[q]", theme.primary()),
            Span::styled(" quit", theme.secondary()),
        ],
        LayoutMode::Narrow => vec![
            Span::styled("[j/k]", theme.primary()),
            Span::styled(" move  ", theme.secondary()),
            Span::styled("[Space]", theme.primary()),
            Span::styled(" toggle  ", theme.secondary()),
            Span::styled("[r]", theme.primary()),
            Span::styled(" refresh  ", theme.secondary()),
            Span::styled("[q]", theme.primary()),
            Span::styled(" quit", theme.secondary()),
        ],
    };

    Paragraph::new(Line::from(spans)).alignment(Alignment::Center)
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

    fn render_ready(width: u16, height: u16, now: DateTime<Utc>) -> String {
        let backend = TestBackend::new(width, height);
        let mut terminal = Terminal::new(backend).unwrap();
        let app = ready_app(now);

        terminal
            .draw(|frame| render(frame, &app, now, 0, Theme::new(ColorMode::None)))
            .unwrap();

        buffer_text(&terminal)
    }

    #[test]
    fn height_constrained_dashboard_keeps_all_provider_data() {
        let now = Utc.timestamp_opt(1_788_000_000, 0).single().unwrap();
        let text = render_ready(120, 31, now);

        for expected in [
            "CODEX",
            "OPENCODE",
            "ANTIGRAVITY",
            "5 hour",
            "Weekly",
            "Sessions",
            "Total Cost",
            "Input",
            "Output",
            "Gemini 5 hour",
            "Gemini weekly",
            "Claude/GPT 5 hour",
            "Claude/GPT weekly",
        ] {
            assert!(text.contains(expected), "missing {expected:?}");
        }
    }

    #[test]
    fn constrained_dashboard_uses_full_area_and_keeps_all_provider_data() {
        let now = Utc.timestamp_opt(1_788_000_000, 0).single().unwrap();
        let text = render_ready(68, 31, now);

        for expected in ["CODEX", "OPENCODE", "ANTIGRAVITY", "65%", "312.3M tokens"] {
            assert!(text.contains(expected), "missing {expected:?}");
        }
        assert!(!text.contains("Terminal too small"));
    }

    #[test]
    fn logo_is_present_on_standard_height_and_hidden_on_narrow_height() {
        let now = Utc.timestamp_opt(1_788_000_000, 0).single().unwrap();
        // Height >= 28 renders logo with full element spacing
        for (w, h) in [(120, 32), (80, 28)] {
            let text = render_ready(w, h, now);
            assert!(text.contains("███████╗"), "logo must be present at {w}x{h}");
            assert!(
                text.contains("CODEX") && text.contains("OPENCODE") && text.contains("ANTIGRAVITY"),
                "all cards must be present at {w}x{h}"
            );
        }

        // Narrow heights (e.g. 24 or 20 rows) hide logo to preserve breathing room between elements
        for (w, h) in [(80, 24), (65, 20)] {
            let narrow_text = render_ready(w, h, now);
            assert!(
                !narrow_text.contains("███████╗"),
                "logo must be hidden at {w}x{h} to preserve spacing"
            );
            for card in ["CODEX", "OPENCODE", "ANTIGRAVITY"] {
                assert!(narrow_text.contains(card), "missing {card} at {w}x{h}");
            }
        }
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
        let backend = TestBackend::new(120, 50);
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
        assert!(!text.contains("remaining"));
        assert!(text.contains("[j/k/↑/↓] select"));
        assert!(text.contains("[Space/Enter] collapse"));
        assert!(text.contains("[r] refresh"));
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
        assert!(text.contains("[j/k] move"));
        assert!(text.contains("[Space] toggle"));
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

        // ponytail: no hindrance – small terminals still render dashboard, not a blocking message
        assert!(!text.contains("Terminal too small"));
        assert!(
            text.contains("CODEX") || text.contains("OPENCODE") || text.contains("ANTIGRAVITY")
        );
    }

    #[test]
    fn collapsed_card_has_border_only_height_and_no_body_lines() {
        let now = Utc.timestamp_opt(1_788_000_000, 0).single().unwrap();
        let mut app = ready_app(now);
        app.toggle_focused_collapse();

        let layout = DashboardLayout::new(
            &app,
            Rect::new(0, 0, 120, 50),
            LayoutMode::Wide,
            Density::Normal,
            Theme::new(ColorMode::None),
            now,
            0,
        );

        assert!(layout.cards[0].1.is_empty());
        assert_eq!(layout.heights[0], 2);
        assert!(!layout.cards[1].1.is_empty());
    }

    #[test]
    fn rendered_titles_show_focus_and_collapse_state() {
        let now = Utc.timestamp_opt(1_788_000_000, 0).single().unwrap();
        let backend = TestBackend::new(120, 50);
        let mut terminal = Terminal::new(backend).unwrap();
        let mut app = ready_app(now);

        terminal
            .draw(|frame| render(frame, &app, now, 0, Theme::new(ColorMode::None)))
            .unwrap();
        assert!(buffer_text(&terminal).contains("▸ [-] CODEX"));

        app.toggle_focused_collapse();
        app.next_provider();
        terminal
            .draw(|frame| render(frame, &app, now, 0, Theme::new(ColorMode::None)))
            .unwrap();
        let text = buffer_text(&terminal);
        assert!(text.contains("[+] CODEX"));
        assert!(text.contains("▸ [-] OPENCODE"));
    }
}
