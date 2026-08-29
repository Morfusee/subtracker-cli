use tokio::sync::mpsc;

use crate::{
    app::App,
    refresh::{ProviderRegistry, RefreshResult, spawn_refreshes},
};

pub struct RuntimeController {
    app: App,
    providers: ProviderRegistry,
    result_tx: mpsc::UnboundedSender<RefreshResult>,
}

impl RuntimeController {
    pub fn new(providers: ProviderRegistry) -> (Self, mpsc::UnboundedReceiver<RefreshResult>) {
        let (result_tx, result_rx) = mpsc::unbounded_channel();

        (
            Self {
                app: App::new(),
                providers,
                result_tx,
            },
            result_rx,
        )
    }

    pub fn app(&self) -> &App {
        &self.app
    }

    pub fn request_refresh(&mut self) {
        let ids = self.app.request_refresh();
        spawn_refreshes(ids, &self.providers, self.result_tx.clone());
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
    use std::sync::Arc;
    use tokio::time::{Duration, timeout};

    use crate::{
        app::DisplayState,
        model::{ProviderId, ProviderSnapshot},
        providers::{ProviderError, UsageProvider},
    };

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

        let (mut runtime, mut refresh_results) = RuntimeController::new(providers);
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
}
