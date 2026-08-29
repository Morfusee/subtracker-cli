use std::{collections::HashMap, sync::Arc};
use tokio::sync::mpsc;

use crate::{
    model::{ProviderId, ProviderSnapshot},
    providers::{ProviderError, UsageProvider},
};

pub type ProviderRegistry = HashMap<ProviderId, Arc<dyn UsageProvider>>;

#[derive(Debug)]
pub struct RefreshResult {
    pub id: ProviderId,
    pub result: Result<ProviderSnapshot, ProviderError>,
}

pub fn spawn_refreshes(
    ids: impl IntoIterator<Item = ProviderId>,
    providers: &ProviderRegistry,
    sender: mpsc::UnboundedSender<RefreshResult>,
) {
    for id in ids {
        let provider = providers
            .get(&id)
            .expect("requested provider must exist in registry")
            .clone();
        let sender = sender.clone();

        tokio::spawn(async move {
            let result = provider.fetch().await;
            let _ = sender.send(RefreshResult { id, result });
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use async_trait::async_trait;
    use chrono::Utc;
    use tokio::{
        sync::Barrier,
        time::{Duration, timeout},
    };

    struct BarrierProvider {
        id: ProviderId,
        barrier: Arc<Barrier>,
    }

    #[async_trait]
    impl UsageProvider for BarrierProvider {
        fn id(&self) -> ProviderId {
            self.id
        }

        async fn fetch(&self) -> Result<ProviderSnapshot, ProviderError> {
            self.barrier.wait().await;
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
    async fn all_requested_providers_start_concurrently() {
        let barrier = Arc::new(Barrier::new(4));
        let mut providers = ProviderRegistry::new();

        for id in ProviderId::ALL {
            providers.insert(
                id,
                Arc::new(BarrierProvider {
                    id,
                    barrier: barrier.clone(),
                }),
            );
        }

        let (tx, mut rx) = mpsc::unbounded_channel();

        spawn_refreshes(ProviderId::ALL, &providers, tx);

        timeout(Duration::from_millis(200), barrier.wait())
            .await
            .expect("all three providers should reach the barrier concurrently");

        let mut received = Vec::new();
        for _ in 0..3 {
            received.push(
                timeout(Duration::from_millis(200), rx.recv())
                    .await
                    .unwrap()
                    .unwrap()
                    .id,
            );
        }

        received.sort_by_key(|id| match id {
            ProviderId::Codex => 0,
            ProviderId::OpenCode => 1,
            ProviderId::Antigravity => 2,
        });

        assert_eq!(received, ProviderId::ALL.to_vec());
    }
}
