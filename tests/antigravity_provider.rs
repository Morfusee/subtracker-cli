mod support;

use std::{sync::Arc, time::Duration};

use subtracker::{
    model::ProviderId,
    providers::{UsageProvider, antigravity::AntigravityProvider, process::CommandSpec},
};

use support::FixtureRunner;

#[tokio::test]
async fn antigravity_usage_command_returns_normalized_quota_windows() {
    let runner = Arc::new(FixtureRunner::success(
        CommandSpec::new("agy", ["-p", "/usage", "--output-format", "json"]),
        include_str!("fixtures/antigravity/usage-success.json"),
    ));

    let provider = AntigravityProvider::new(runner, Duration::from_secs(15));
    let snapshot = provider.fetch().await.unwrap();

    assert_eq!(snapshot.provider, ProviderId::Antigravity);
    assert_eq!(snapshot.quotas.len(), 4);

    assert_eq!(snapshot.quotas[0].label, "Gemini 5 hour");
    assert_eq!(snapshot.quotas[0].remaining_percent, Some(98.0));

    assert_eq!(snapshot.quotas[1].label, "Gemini weekly");
    assert_eq!(snapshot.quotas[1].remaining_percent, Some(41.0));

    assert_eq!(snapshot.quotas[2].label, "Claude/GPT 5 hour");
    assert_eq!(snapshot.quotas[2].remaining_percent, Some(63.0));

    assert_eq!(snapshot.quotas[3].label, "Claude/GPT weekly");
    assert_eq!(snapshot.quotas[3].remaining_percent, Some(100.0));
}

#[tokio::test]
async fn antigravity_parses_real_cli_group_bucket_format() {
    let real_output = r#"{
      "status": "SUCCESS",
      "command": {
        "name": "usage",
        "data": {
          "groups": [
            {
              "name": "Gemini Models",
              "buckets": [
                {
                  "id": "gemini-weekly",
                  "name": "Weekly Limit Remaining",
                  "remaining_fraction": 0.91,
                  "reset_time": "2026-09-05T04:50:15Z"
                },
                {
                  "id": "gemini-5h",
                  "name": "Five Hour Limit Remaining",
                  "remaining_fraction": 0.52,
                  "reset_time": "2026-08-29T09:50:15Z"
                }
              ]
            },
            {
              "name": "Claude and GPT models",
              "buckets": [
                {
                  "id": "3p-weekly",
                  "remaining_fraction": 1,
                  "reset_time": "2026-09-05T08:59:18Z"
                },
                {
                  "id": "3p-5h",
                  "remaining_fraction": 0.75,
                  "reset_time": "2026-08-29T13:59:18Z"
                }
              ]
            }
          ]
        }
      }
    }"#;

    let runner = Arc::new(FixtureRunner::success(
        CommandSpec::new("agy", ["-p", "/usage", "--output-format", "json"]),
        real_output,
    ));

    let provider = AntigravityProvider::new(runner, Duration::from_secs(15));
    let snapshot = provider.fetch().await.unwrap();

    assert_eq!(snapshot.provider, ProviderId::Antigravity);
    assert_eq!(snapshot.quotas.len(), 4);
    assert_eq!(snapshot.quotas[0].label, "Gemini 5 hour");
    assert_eq!(snapshot.quotas[0].remaining_percent, Some(52.0));
    assert_eq!(snapshot.quotas[1].label, "Gemini weekly");
    assert_eq!(snapshot.quotas[1].remaining_percent, Some(91.0));
    assert_eq!(snapshot.quotas[2].label, "Claude/GPT 5 hour");
    assert_eq!(snapshot.quotas[2].remaining_percent, Some(75.0));
    assert_eq!(snapshot.quotas[3].label, "Claude/GPT weekly");
    assert_eq!(snapshot.quotas[3].remaining_percent, Some(100.0));
}
