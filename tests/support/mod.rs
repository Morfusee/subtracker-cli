use async_trait::async_trait;
use std::{sync::Mutex, time::Duration};

use subtracker::providers::{
    ProviderError,
    process::{CommandOutput, CommandSpec, ProcessRunner},
};

pub struct FixtureRunner {
    expected: CommandSpec,
    output: Mutex<Option<Result<CommandOutput, ProviderError>>>,
}

impl FixtureRunner {
    pub fn success(expected: CommandSpec, stdout: impl Into<String>) -> Self {
        Self {
            expected,
            output: Mutex::new(Some(Ok(CommandOutput {
                success: true,
                stdout: stdout.into(),
                stderr: String::new(),
            }))),
        }
    }
}

#[async_trait]
impl ProcessRunner for FixtureRunner {
    async fn run(
        &self,
        spec: &CommandSpec,
        _timeout_duration: Duration,
    ) -> Result<CommandOutput, ProviderError> {
        assert_eq!(spec, &self.expected);
        self.output
            .lock()
            .unwrap()
            .take()
            .expect("fixture runner called more than once")
    }
}
