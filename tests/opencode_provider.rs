use std::{fs, time::Duration};

use subtracker::{
    model::ProviderId,
    providers::{ProviderError, UsageProvider, opencode::OpenCodeProvider},
};
use tempfile::tempdir;
use url::Url;
use wiremock::{
    Mock, MockServer, ResponseTemplate,
    matchers::{header, method, path},
};

#[tokio::test]
async fn opencode_fetch_uses_current_auth_and_bearer_header() {
    let server = MockServer::start().await;
    let fixture = include_str!("fixtures/opencode/usage-success.json");

    Mock::given(method("GET"))
        .and(path("/zen/go/v1/usage"))
        .and(header("authorization", "Bearer sk-opencode-fixture-key"))
        .and(header("accept", "application/json"))
        .respond_with(ResponseTemplate::new(200).set_body_raw(fixture, "application/json"))
        .expect(1)
        .mount(&server)
        .await;

    let temp = tempdir().unwrap();
    let auth_path = temp.path().join("auth.json");
    fs::write(
        &auth_path,
        include_str!("fixtures/opencode/auth-success.json"),
    )
    .unwrap();

    let endpoint = Url::parse(&format!("{}/zen/go/v1/usage", server.uri())).unwrap();

    let provider = OpenCodeProvider::new(
        reqwest::Client::builder()
            .timeout(Duration::from_secs(15))
            .build()
            .unwrap(),
        auth_path,
        endpoint,
    );

    let snapshot = provider.fetch().await.unwrap();

    assert_eq!(snapshot.provider, ProviderId::OpenCode);
    assert_eq!(snapshot.quotas.len(), 3);
    assert_eq!(snapshot.quotas[0].remaining_percent, Some(91.0));
    assert_eq!(snapshot.quotas[1].remaining_percent, Some(88.0));
    assert_eq!(snapshot.quotas[2].remaining_percent, Some(85.0));
}

#[tokio::test]
async fn opencode_unauthorized_response_surfaces_not_authenticated() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/zen/go/v1/usage"))
        .respond_with(ResponseTemplate::new(401))
        .expect(1)
        .mount(&server)
        .await;

    let temp = tempdir().unwrap();
    let auth_path = temp.path().join("auth.json");
    fs::write(
        &auth_path,
        include_str!("fixtures/opencode/auth-success.json"),
    )
    .unwrap();

    let endpoint = Url::parse(&format!("{}/zen/go/v1/usage", server.uri())).unwrap();

    let provider = OpenCodeProvider::new(reqwest::Client::new(), auth_path, endpoint);

    assert_eq!(
        provider.fetch().await.unwrap_err(),
        ProviderError::NotAuthenticated
    );
}
