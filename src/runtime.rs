use tokio::sync::mpsc;

use crate::{
    app::App,
    refresh::{ProviderRegistry, RefreshResult, spawn_refreshes},
    updater::{UpdateCheckResult, UpdateChecker, spawn_update_check},
};

pub struct RuntimeController {
    app: App,
    providers: ProviderRegistry,
    result_tx: mpsc::UnboundedSender<RefreshResult>,
    update_checker: UpdateChecker,
    update_result_tx: mpsc::UnboundedSender<UpdateCheckResult>,
}

impl RuntimeController {
    pub fn new(
        providers: ProviderRegistry,
        update_checker: UpdateChecker,
    ) -> (
        Self,
        mpsc::UnboundedReceiver<RefreshResult>,
        mpsc::UnboundedReceiver<UpdateCheckResult>,
    ) {
        let (result_tx, result_rx) = mpsc::unbounded_channel();
        let (update_result_tx, update_result_rx) = mpsc::unbounded_channel();

        (
            Self {
                app: App::new(),
                providers,
                result_tx,
                update_checker,
                update_result_tx,
            },
            result_rx,
            update_result_rx,
        )
    }

    pub fn app(&self) -> &App {
        &self.app
    }

    pub fn request_refresh(&mut self) {
        let ids = self.app.request_refresh();
        spawn_refreshes(ids, &self.providers, self.result_tx.clone());
    }

    pub fn request_update_check(&mut self) {
        if self.app.request_update_check() {
            spawn_update_check(self.update_checker.clone(), self.update_result_tx.clone());
        }
    }

    pub fn apply_update_check_result(&mut self, result: UpdateCheckResult) {
        self.app.finish_update_check(result);
    }

    pub fn open_update_modal(&mut self) {
        self.app.open_update_modal();
    }

    pub fn close_update_modal(&mut self) {
        self.app.close_update_modal();
    }

    pub fn next_update_action(&mut self) {
        self.app.next_update_action();
    }

    pub fn previous_update_action(&mut self) {
        self.app.previous_update_action();
    }

    pub fn dismiss_update(&mut self) {
        self.app.dismiss_update();
    }

    pub fn next_provider(&mut self) {
        self.app.next_provider();
    }

    pub fn prev_provider(&mut self) {
        self.app.prev_provider();
    }

    pub fn toggle_focused_collapse(&mut self) {
        self.app.toggle_focused_collapse();
    }

    pub fn apply_refresh_result(&mut self, refresh: RefreshResult) {
        let restart = self.app.finish_refresh(refresh.id, refresh.result);

        if restart {
            spawn_refreshes([refresh.id], &self.providers, self.result_tx.clone());
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use async_trait::async_trait;
    use chrono::Utc;
    use semver::Version;
    use std::sync::Arc;
    use tokio::time::{Duration, timeout};
    use url::Url;
    use wiremock::{Mock, MockServer, ResponseTemplate, matchers::method};

    use crate::{
        app::{DisplayState, UpdateState},
        model::{ProviderId, ProviderSnapshot},
        providers::{ProviderError, UsageProvider},
    };

    fn update_checker(server: &MockServer) -> UpdateChecker {
        UpdateChecker::new(
            reqwest::Client::new(),
            Url::parse(&server.uri()).unwrap(),
            Version::parse("0.2.0").unwrap(),
        )
    }

    struct ImmediateProvider {
        id: ProviderId,
    }

    #[async_trait]
    impl UsageProvider for ImmediateProvider {
        fn id(&self) -> ProviderId {
            self.id
        }

        async fn fetch(&self) -> Result<ProviderSnapshot, ProviderError> {
            Ok(ProviderSnapshot {
                provider: self.id,
                account_label: None,
                quotas: Vec::new(),
                stats: Vec::new(),
                fetched_at: Utc::now(),
            })
        }
    }

    #[tokio::test]
    async fn request_refresh_spawns_results_and_applies_them_to_app() {
        let mut providers = ProviderRegistry::new();
        for id in ProviderId::ALL {
            providers.insert(id, Arc::new(ImmediateProvider { id }));
        }

        let server = MockServer::start().await;
        let (mut runtime, mut refresh_results, _) =
            RuntimeController::new(providers, update_checker(&server));
        runtime.request_refresh();

        for _ in 0..3 {
            let result = timeout(Duration::from_millis(200), refresh_results.recv())
                .await
                .unwrap()
                .unwrap();

            runtime.apply_refresh_result(result);
        }

        for id in ProviderId::ALL {
            assert_eq!(runtime.app().provider(id).display, DisplayState::Ready);
        }
    }

    #[tokio::test]
    async fn update_check_results_are_delivered_and_applied_to_app() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "tag_name": "v0.3.0",
                "html_url": "https://github.com/Morfusee/subtracker-cli/releases/tag/v0.3.0"
            })))
            .mount(&server)
            .await;
        let (mut runtime, _, mut update_results) =
            RuntimeController::new(ProviderRegistry::new(), update_checker(&server));

        runtime.request_update_check();
        let result = timeout(Duration::from_millis(200), update_results.recv())
            .await
            .unwrap()
            .unwrap();
        runtime.apply_update_check_result(result);

        assert!(matches!(
            runtime.app().update_state(),
            UpdateState::Available(_)
        ));
    }
}
