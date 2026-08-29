pub mod format;
pub mod provider_card;

use chrono::{DateTime, Utc};
use ratatui::{
    Frame,
    layout::{Constraint, Layout},
    text::Line,
    widgets::{Block, Borders, Paragraph},
};

use crate::{app::App, model::ProviderId};

pub fn render(frame: &mut Frame, app: &App, now: DateTime<Utc>) {
    let card_heights = ProviderId::ALL.map(|id| {
        let state = app.provider(id);
        let content_len = provider_card::lines_for_provider(state, now).len();
        Constraint::Length((content_len as u16 + 2).max(4))
    });

    let areas = Layout::vertical([
        card_heights[0],
        card_heights[1],
        card_heights[2],
        Constraint::Length(1),
    ])
    .split(frame.area());

    for (index, id) in ProviderId::ALL.into_iter().enumerate() {
        let state = app.provider(id);
        let mut lines = provider_card::lines_for_provider(state, now);

        if let (None, crate::app::DisplayState::Unavailable(error)) =
            (&state.snapshot, &state.display)
        {
            match (id, error) {
                (ProviderId::Codex, crate::providers::ProviderError::NotAuthenticated)
                | (ProviderId::Codex, crate::providers::ProviderError::CredentialsNotFound) => {
                    lines.push(Line::from("Run `codex login`"));
                }
                (ProviderId::OpenCode, crate::providers::ProviderError::NotAuthenticated)
                | (ProviderId::OpenCode, crate::providers::ProviderError::CredentialsNotFound) => {
                    lines.push(Line::from("Connect account in OpenCode"));
                }
                (ProviderId::Antigravity, crate::providers::ProviderError::NotAuthenticated) => {
                    lines.push(Line::from("Run `agy` and sign in"));
                }
                _ => {}
            }
        }

        let card = Paragraph::new(lines).block(
            Block::default()
                .borders(Borders::ALL)
                .title(id.display_name()),
        );

        frame.render_widget(card, areas[index]);
    }

    frame.render_widget(Paragraph::new("r refresh    auto 60s    q quit"), areas[3]);
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::{TimeZone, Utc};
    use ratatui::{Terminal, backend::TestBackend};

    use crate::{
        app::App,
        model::{ProviderId, ProviderSnapshot, QuotaWindow},
        providers::ProviderError,
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

    #[test]
    fn dashboard_renders_ready_stale_and_unavailable_providers_together() {
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
                    used_percent: Some(28.0),
                    remaining_percent: Some(72.0),
                    resets_at: Some(now + chrono::Duration::hours(2)),
                    window_seconds: Some(18_000),
                }],
                stats: Vec::new(),
                fetched_at: now,
            }),
        );

        app.finish_refresh(ProviderId::OpenCode, Err(ProviderError::CliNotFound));

        app.finish_refresh(
            ProviderId::Antigravity,
            Err(ProviderError::NotAuthenticated),
        );

        let backend = TestBackend::new(90, 28);
        let mut terminal = Terminal::new(backend).unwrap();

        terminal.draw(|frame| render(frame, &app, now)).unwrap();

        let text = buffer_text(&terminal);

        assert!(text.contains("CODEX"));
        assert!(text.contains("72% remaining"));
        assert!(text.contains("OPENCODE"));
        assert!(text.contains("CLI not found"));
        assert!(text.contains("ANTIGRAVITY"));
        assert!(text.contains("Not authenticated"));
        assert!(text.contains("r refresh"));
        assert!(text.contains("q quit"));
    }
}
