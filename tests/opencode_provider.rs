mod support;

use std::{sync::Arc, time::Duration};

use subtracker_cli::{
    model::{ProviderId, UsageValue},
    providers::{UsageProvider, opencode::OpenCodeProvider, process::CommandSpec},
};

use support::FixtureRunner;

#[tokio::test]
async fn opencode_stats_are_normalized_to_four_v1_values() {
    let runner = Arc::new(FixtureRunner::success(
        CommandSpec::new("opencode", ["stats"]),
        include_str!("fixtures/opencode/stats-success.txt"),
    ));

    let provider = OpenCodeProvider::new(runner, Duration::from_secs(15));
    let snapshot = provider.fetch().await.unwrap();

    assert_eq!(snapshot.provider, ProviderId::OpenCode);
    assert!(snapshot.quotas.is_empty());
    assert_eq!(snapshot.stats.len(), 4);

    assert_eq!(snapshot.stats[0].label, "Sessions");
    assert_eq!(snapshot.stats[0].value, UsageValue::Count(42));

    assert_eq!(snapshot.stats[1].label, "Total Cost");
    assert_eq!(snapshot.stats[1].value, UsageValue::MoneyCents(1234));

    assert_eq!(snapshot.stats[2].label, "Input");
    assert_eq!(snapshot.stats[2].value, UsageValue::Tokens(599_000));

    assert_eq!(snapshot.stats[3].label, "Output");
    assert_eq!(snapshot.stats[3].value, UsageValue::Tokens(18_000));
}
