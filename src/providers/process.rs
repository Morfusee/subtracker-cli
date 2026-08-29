use async_trait::async_trait;
use std::{future::Future, time::Duration};
use tokio::{process::Command, time::timeout};

use super::ProviderError;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CommandSpec {
    pub program: String,
    pub args: Vec<String>,
}

impl CommandSpec {
    pub fn new(
        program: impl Into<String>,
        args: impl IntoIterator<Item = impl Into<String>>,
    ) -> Self {
        Self {
            program: program.into(),
            args: args.into_iter().map(Into::into).collect(),
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CommandOutput {
    pub success: bool,
    pub stdout: String,
    pub stderr: String,
}

#[async_trait]
pub trait ProcessRunner: Send + Sync {
    async fn run(
        &self,
        spec: &CommandSpec,
        timeout_duration: Duration,
    ) -> Result<CommandOutput, ProviderError>;
}

#[derive(Clone, Copy, Debug, Default)]
pub struct SystemProcessRunner;

#[async_trait]
impl ProcessRunner for SystemProcessRunner {
    async fn run(
        &self,
        spec: &CommandSpec,
        timeout_duration: Duration,
    ) -> Result<CommandOutput, ProviderError> {
        let mut command = Command::new(&spec.program);
        command.args(&spec.args);
        command.kill_on_drop(true);

        let output = with_timeout(timeout_duration, async {
            command.output().await.map_err(|error| {
                if error.kind() == std::io::ErrorKind::NotFound {
                    ProviderError::CliNotFound
                } else {
                    ProviderError::CommandFailed
                }
            })
        })
        .await?;

        Ok(CommandOutput {
            success: output.status.success(),
            stdout: String::from_utf8_lossy(&output.stdout).into_owned(),
            stderr: String::from_utf8_lossy(&output.stderr).into_owned(),
        })
    }
}

pub async fn with_timeout<T, F>(timeout_duration: Duration, future: F) -> Result<T, ProviderError>
where
    F: Future<Output = Result<T, ProviderError>>,
{
    timeout(timeout_duration, future)
        .await
        .map_err(|_| ProviderError::Timeout)?
}

pub fn classify_command_output(output: &CommandOutput) -> Result<&str, ProviderError> {
    if output.success {
        return Ok(&output.stdout);
    }

    let stderr = output.stderr.to_ascii_lowercase();
    if stderr.contains("authentication required")
        || stderr.contains("not authenticated")
        || stderr.contains("sign in")
        || stderr.contains("login required")
    {
        Err(ProviderError::NotAuthenticated)
    } else {
        Err(ProviderError::CommandFailed)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{future::pending, time::Duration};

    #[tokio::test]
    async fn missing_program_is_reported_as_cli_not_found() {
        let runner = SystemProcessRunner;
        let spec = CommandSpec::new("__stc_binary_that_does_not_exist__", Vec::<String>::new());

        let error = runner
            .run(&spec, Duration::from_millis(100))
            .await
            .unwrap_err();

        assert_eq!(error, ProviderError::CliNotFound);
    }

    #[tokio::test]
    async fn pending_work_is_stopped_by_timeout() {
        let result = with_timeout(
            Duration::from_millis(10),
            pending::<Result<(), ProviderError>>(),
        )
        .await;

        assert_eq!(result, Err(ProviderError::Timeout));
    }

    #[test]
    fn authentication_failure_is_classified_without_returning_raw_stderr() {
        let output = CommandOutput {
            success: false,
            stdout: String::new(),
            stderr: "authentication required: secret detail".into(),
        };

        assert_eq!(
            classify_command_output(&output),
            Err(ProviderError::NotAuthenticated)
        );
    }
}
