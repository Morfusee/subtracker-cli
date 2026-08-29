use std::{fs, time::Duration};

use subtracker_cli::{
    model::ProviderId,
    providers::{ProviderError, UsageProvider, codex::CodexProvider},
};
use tempfile::tempdir;
use url::Url;
use wiremock::{
    Mock, MockServer, ResponseTemplate,
    matchers::{header, method, path},
};

#[tokio::test]
async fn codex_fetch_uses_current_auth_and_required_headers() {
    let server = MockServer::start().await;
    let fixture = include_str!("fixtures/codex/usage-success.json");

    Mock::given(method("GET"))
        .and(path("/backend-api/wham/usage"))
        .and(header("authorization", "Bearer fixture-access-token"))
        .and(header("chatgpt-account-id", "fixture-account-id"))
        .and(header("accept", "application/json"))
        .respond_with(ResponseTemplate::new(200).set_body_raw(fixture, "application/json"))
        .expect(1)
        .mount(&server)
        .await;

    let temp = tempdir().unwrap();
    let auth_path = temp.path().join("auth.json");
    fs::write(&auth_path, include_str!("fixtures/codex/auth-success.json")).unwrap();

    let endpoint = Url::parse(&format!("{}/backend-api/wham/usage", server.uri())).unwrap();

    let provider = CodexProvider::new(
        reqwest::Client::builder()
            .timeout(Duration::from_secs(15))
            .build()
            .unwrap(),
        auth_path,
        endpoint,
    );

    let snapshot = provider.fetch().await.unwrap();

    assert_eq!(snapshot.provider, ProviderId::Codex);
    assert_eq!(snapshot.quotas[0].remaining_percent, Some(77.0));
}

#[tokio::test]
async fn codex_unauthorized_response_requires_relogin_instead_of_refreshing_oauth() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/backend-api/wham/usage"))
        .respond_with(ResponseTemplate::new(401))
        .expect(1)
        .mount(&server)
        .await;

    let temp = tempdir().unwrap();
    let auth_path = temp.path().join("auth.json");
    fs::write(&auth_path, include_str!("fixtures/codex/auth-success.json")).unwrap();

    let endpoint = Url::parse(&format!("{}/backend-api/wham/usage", server.uri())).unwrap();

    let provider = CodexProvider::new(reqwest::Client::new(), auth_path, endpoint);

    assert_eq!(
        provider.fetch().await.unwrap_err(),
        ProviderError::NotAuthenticated
    );
}
