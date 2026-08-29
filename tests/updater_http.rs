use std::time::Duration;

use semver::Version;
use subtracker::updater::{UpdateChecker, UpdateError};
use url::Url;
use wiremock::{
    Mock, MockServer, ResponseTemplate,
    matchers::{header, method, path},
};

fn checker(server: &MockServer, timeout: Duration) -> UpdateChecker {
    UpdateChecker::new(
        reqwest::Client::builder().timeout(timeout).build().unwrap(),
        Url::parse(&format!("{}/releases/latest", server.uri())).unwrap(),
        Version::parse("0.2.0").unwrap(),
    )
}

async fn mount_release(server: &MockServer, tag: &str) {
    Mock::given(method("GET"))
        .and(path("/releases/latest"))
        .and(header("user-agent", "subtracker/0.2.0"))
        .and(header("accept", "application/vnd.github+json"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "tag_name": tag,
            "html_url": "https://github.com/Morfusee/subtracker-cli/releases/tag/v0.3.0"
        })))
        .mount(server)
        .await;
}

#[tokio::test]
async fn newer_prefixed_release_is_reported() {
    let server = MockServer::start().await;
    mount_release(&server, "v0.3.0").await;

    let update = checker(&server, Duration::from_secs(1))
        .check()
        .await
        .unwrap()
        .unwrap();

    assert_eq!(update.version, Version::parse("0.3.0").unwrap());
    assert_eq!(
        update.release_url.as_str(),
        "https://github.com/Morfusee/subtracker-cli/releases/tag/v0.3.0"
    );
}

#[tokio::test]
async fn newer_unprefixed_release_is_reported() {
    let server = MockServer::start().await;
    mount_release(&server, "0.3.0").await;
    assert!(
        checker(&server, Duration::from_secs(1))
            .check()
            .await
            .unwrap()
            .is_some()
    );
}

#[tokio::test]
async fn equal_and_older_releases_are_not_reported() {
    for tag in ["v0.2.0", "v0.1.9"] {
        let server = MockServer::start().await;
        mount_release(&server, tag).await;
        assert!(
            checker(&server, Duration::from_secs(1))
                .check()
                .await
                .unwrap()
                .is_none()
        );
    }
}

#[tokio::test]
async fn malformed_payload_and_version_are_rejected() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .respond_with(ResponseTemplate::new(200).set_body_string("not json"))
        .mount(&server)
        .await;
    assert_eq!(
        checker(&server, Duration::from_secs(1)).check().await,
        Err(UpdateError::InvalidPayload)
    );

    let server = MockServer::start().await;
    mount_release(&server, "release-next").await;
    assert_eq!(
        checker(&server, Duration::from_secs(1)).check().await,
        Err(UpdateError::InvalidVersion)
    );
}

#[tokio::test]
async fn http_error_and_timeout_are_silent_check_failures() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .respond_with(ResponseTemplate::new(500))
        .mount(&server)
        .await;
    assert_eq!(
        checker(&server, Duration::from_secs(1)).check().await,
        Err(UpdateError::Network)
    );

    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .respond_with(ResponseTemplate::new(200).set_delay(Duration::from_millis(100)))
        .mount(&server)
        .await;
    assert_eq!(
        checker(&server, Duration::from_millis(20)).check().await,
        Err(UpdateError::Network)
    );
}
