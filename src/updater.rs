use std::{
    io,
    process::{Command, Stdio},
    time::Duration,
};

use semver::Version;
use serde::Deserialize;
use tokio::sync::mpsc;
use url::Url;

const LATEST_RELEASE_ENDPOINT: &str =
    "https://api.github.com/repos/Morfusee/subtracker-cli/releases/latest";

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AvailableUpdate {
    pub version: Version,
    pub release_url: Url,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, thiserror::Error)]
pub enum UpdateError {
    #[error("update request failed")]
    Network,
    #[error("GitHub returned an invalid release payload")]
    InvalidPayload,
    #[error("GitHub returned an invalid release version")]
    InvalidVersion,
}

pub type UpdateCheckResult = Result<Option<AvailableUpdate>, UpdateError>;

#[derive(Clone)]
pub struct UpdateChecker {
    client: reqwest::Client,
    endpoint: Url,
    current_version: Version,
}

#[derive(Deserialize)]
struct GithubRelease {
    tag_name: String,
    html_url: String,
}

impl UpdateChecker {
    pub fn new(client: reqwest::Client, endpoint: Url, current_version: Version) -> Self {
        Self {
            client,
            endpoint,
            current_version,
        }
    }

    pub fn production() -> Result<Self, UpdateError> {
        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(5))
            .build()
            .map_err(|_| UpdateError::Network)?;
        let endpoint =
            Url::parse(LATEST_RELEASE_ENDPOINT).map_err(|_| UpdateError::InvalidPayload)?;
        let current_version =
            Version::parse(env!("CARGO_PKG_VERSION")).map_err(|_| UpdateError::InvalidVersion)?;
        Ok(Self::new(client, endpoint, current_version))
    }

    pub async fn check(&self) -> UpdateCheckResult {
        let response = self
            .client
            .get(self.endpoint.clone())
            .header("User-Agent", format!("subtracker/{}", self.current_version))
            .header("Accept", "application/vnd.github+json")
            .send()
            .await
            .map_err(|_| UpdateError::Network)?
            .error_for_status()
            .map_err(|_| UpdateError::Network)?;
        let release: GithubRelease = response
            .json()
            .await
            .map_err(|_| UpdateError::InvalidPayload)?;
        let version = Version::parse(
            release
                .tag_name
                .strip_prefix('v')
                .unwrap_or(&release.tag_name),
        )
        .map_err(|_| UpdateError::InvalidVersion)?;
        let release_url = Url::parse(&release.html_url).map_err(|_| UpdateError::InvalidPayload)?;
        Ok((version > self.current_version).then_some(AvailableUpdate {
            version,
            release_url,
        }))
    }
}

pub fn spawn_update_check(
    checker: UpdateChecker,
    sender: mpsc::UnboundedSender<UpdateCheckResult>,
) {
    tokio::spawn(async move {
        let _ = sender.send(checker.check().await);
    });
}

pub fn open_release_notes(url: &Url) -> io::Result<()> {
    #[cfg(target_os = "windows")]
    let mut command = {
        let mut c = Command::new("explorer.exe");
        c.arg(url.as_str());
        c
    };
    #[cfg(target_os = "macos")]
    let mut command = {
        let mut c = Command::new("open");
        c.arg(url.as_str());
        c
    };
    #[cfg(all(unix, not(target_os = "macos")))]
    let mut command = {
        let mut c = Command::new("xdg-open");
        c.arg(url.as_str());
        c
    };

    let status = command
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()?;
    if status.success() {
        Ok(())
    } else {
        Err(io::Error::other(format!(
            "browser opener exited with {status}"
        )))
    }
}

pub fn install_update() -> io::Result<bool> {
    Ok(Command::new("cargo")
        .args(["install", "subtracker"])
        .status()?
        .success())
}
