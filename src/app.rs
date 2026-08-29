use std::collections::HashMap;

use crate::{
    model::{ProviderId, ProviderSnapshot},
    providers::ProviderError,
};

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum DisplayState {
    Loading,
    Refreshing,
    Ready,
    Stale(ProviderError),
    Unavailable(ProviderError),
}

#[derive(Clone, Debug)]
pub struct ProviderState {
    pub snapshot: Option<ProviderSnapshot>,
    pub display: DisplayState,
    in_flight: bool,
    pending_refresh: bool,
}

impl ProviderState {
    fn new() -> Self {
        Self {
            snapshot: None,
            display: DisplayState::Loading,
            in_flight: false,
            pending_refresh: false,
        }
    }
}

#[derive(Debug)]
pub struct App {
    providers: HashMap<ProviderId, ProviderState>,
    focused_index: usize,
    collapsed: [bool; ProviderId::ALL.len()],
}

impl App {
    pub fn new() -> Self {
        let providers = ProviderId::ALL
            .into_iter()
            .map(|provider| (provider, ProviderState::new()))
            .collect();

        Self {
            providers,
            focused_index: 0,
            collapsed: [false; ProviderId::ALL.len()],
        }
    }

    pub fn focused_provider(&self) -> ProviderId {
        ProviderId::ALL[self.focused_index]
    }

    pub fn is_focused(&self, id: ProviderId) -> bool {
        self.focused_provider() == id
    }

    pub fn is_collapsed(&self, id: ProviderId) -> bool {
        self.collapsed[provider_index(id)]
    }

    pub fn next_provider(&mut self) {
        self.focused_index = (self.focused_index + 1) % ProviderId::ALL.len();
    }

    pub fn prev_provider(&mut self) {
        self.focused_index =
            (self.focused_index + ProviderId::ALL.len() - 1) % ProviderId::ALL.len();
    }

    pub fn toggle_focused_collapse(&mut self) {
        self.collapsed[self.focused_index] = !self.collapsed[self.focused_index];
    }

    pub fn provider(&self, id: ProviderId) -> &ProviderState {
        self.providers
            .get(&id)
            .expect("all provider states are created at startup")
    }

    pub fn request_refresh(&mut self) -> Vec<ProviderId> {
        let mut start = Vec::new();

        for id in ProviderId::ALL {
            let state = self
                .providers
                .get_mut(&id)
                .expect("all provider states are created at startup");

            if state.in_flight {
                state.pending_refresh = true;
                continue;
            }

            state.in_flight = true;
            state.display = DisplayState::Refreshing;
            start.push(id);
        }

        start
    }

    pub fn finish_refresh(
        &mut self,
        id: ProviderId,
        result: Result<ProviderSnapshot, ProviderError>,
    ) -> bool {
        let state = self
            .providers
            .get_mut(&id)
            .expect("all provider states are created at startup");

        state.in_flight = false;

        match result {
            Ok(snapshot) => {
                state.snapshot = Some(snapshot);
                state.display = DisplayState::Ready;
            }
            Err(error) if state.snapshot.is_some() => {
                state.display = DisplayState::Stale(error);
            }
            Err(error) => {
                state.display = DisplayState::Unavailable(error);
            }
        }

        if state.pending_refresh {
            state.pending_refresh = false;
            state.in_flight = true;
            state.display = DisplayState::Refreshing;
            true
        } else {
            false
        }
    }
}

fn provider_index(id: ProviderId) -> usize {
    ProviderId::ALL
        .iter()
        .position(|candidate| *candidate == id)
        .expect("provider id belongs to ProviderId::ALL")
}

impl Default for App {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::{TimeZone, Utc};

    fn snapshot(provider: ProviderId) -> ProviderSnapshot {
        ProviderSnapshot {
            provider,
            account_label: None,
            quotas: Vec::new(),
            stats: Vec::new(),
            fetched_at: Utc.timestamp_opt(1_788_000_000, 0).single().unwrap(),
        }
    }

    #[test]
    fn startup_refresh_starts_all_three_providers() {
        let mut app = App::new();

        assert_eq!(app.request_refresh(), ProviderId::ALL.to_vec());
    }

    #[test]
    fn refresh_requests_are_coalesced_while_provider_is_in_flight() {
        let mut app = App::new();
        let first = app.request_refresh();
        assert_eq!(first, ProviderId::ALL.to_vec());

        let second = app.request_refresh();
        assert!(second.is_empty());

        let start_again = app.finish_refresh(ProviderId::Codex, Ok(snapshot(ProviderId::Codex)));

        assert!(start_again);
    }

    #[test]
    fn failed_refresh_keeps_previous_snapshot_and_marks_it_stale() {
        let mut app = App::new();
        app.request_refresh();
        app.finish_refresh(ProviderId::Codex, Ok(snapshot(ProviderId::Codex)));

        app.request_refresh();
        app.finish_refresh(ProviderId::Codex, Err(ProviderError::Network));

        let state = app.provider(ProviderId::Codex);
        assert!(state.snapshot.is_some());
        assert_eq!(state.display, DisplayState::Stale(ProviderError::Network));
    }

    #[test]
    fn first_refresh_failure_is_unavailable_without_affecting_other_provider_state() {
        let mut app = App::new();
        app.request_refresh();

        app.finish_refresh(ProviderId::Codex, Err(ProviderError::NotAuthenticated));
        app.finish_refresh(ProviderId::OpenCode, Ok(snapshot(ProviderId::OpenCode)));

        assert_eq!(
            app.provider(ProviderId::Codex).display,
            DisplayState::Unavailable(ProviderError::NotAuthenticated)
        );
        assert_eq!(
            app.provider(ProviderId::OpenCode).display,
            DisplayState::Ready
        );
    }

    #[test]
    fn provider_focus_starts_on_codex_and_wraps_in_both_directions() {
        let mut app = App::new();

        assert_eq!(app.focused_provider(), ProviderId::Codex);

        app.prev_provider();
        assert_eq!(app.focused_provider(), ProviderId::Antigravity);

        app.next_provider();
        assert_eq!(app.focused_provider(), ProviderId::Codex);
        app.next_provider();
        assert_eq!(app.focused_provider(), ProviderId::OpenCode);
    }

    #[test]
    fn collapse_flags_are_independent_and_toggle_only_the_focused_provider() {
        let mut app = App::new();

        app.toggle_focused_collapse();
        assert!(app.is_collapsed(ProviderId::Codex));
        assert!(!app.is_collapsed(ProviderId::OpenCode));

        app.next_provider();
        app.toggle_focused_collapse();
        assert!(app.is_collapsed(ProviderId::Codex));
        assert!(app.is_collapsed(ProviderId::OpenCode));
        assert!(!app.is_collapsed(ProviderId::Antigravity));
    }
}
