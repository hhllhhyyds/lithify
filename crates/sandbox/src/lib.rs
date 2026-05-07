// Sandbox execution environment — subprocess isolation for tool execution.
// Implements the Sandbox trait from lithify-core.

use std::path::Path;
use std::process::Stdio;

use async_trait::async_trait;
use tokio::time::Duration;

use lithify_core::{ExecutionResult, Sandbox, SandboxError};

/// Default timeout for sandboxed commands (5 minutes).
pub const DEFAULT_TIMEOUT: Duration = Duration::from_secs(300);

/// Executes shell commands in an isolated subprocess.
///
/// Wraps `tokio::process::Command` with timeout control and output capture.
pub struct SandboxImpl {
    pub(crate) default_timeout: Duration,
}

impl SandboxImpl {
    /// Creates a new sandbox with the given default timeout.
    pub fn new(default_timeout: Duration) -> Self {
        Self { default_timeout }
    }
}

impl Default for SandboxImpl {
    fn default() -> Self {
        Self {
            default_timeout: DEFAULT_TIMEOUT,
        }
    }
}

#[async_trait]
impl Sandbox for SandboxImpl {
    async fn run(
        &self,
        command: &str,
        args: &[String],
        working_dir: Option<&Path>,
    ) -> Result<ExecutionResult, SandboxError> {
        let mut cmd = tokio::process::Command::new(command);
        cmd.args(args)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .stdin(Stdio::null())
            .kill_on_drop(true);

        if let Some(dir) = working_dir {
            cmd.current_dir(dir);
        }

        let child = cmd
            .spawn()
            .map_err(|e| SandboxError::Command(e.to_string()))?;

        let timeout_ms = self.default_timeout.as_millis() as u64;
        match tokio::time::timeout(self.default_timeout, child.wait_with_output()).await {
            Ok(Ok(output)) => {
                let exit_code = output.status.code().unwrap_or(-1);
                let stdout = String::from_utf8_lossy(&output.stdout).into();
                let stderr = String::from_utf8_lossy(&output.stderr).into();
                Ok(ExecutionResult {
                    exit_code,
                    stdout,
                    stderr,
                })
            }
            Ok(Err(e)) => Err(SandboxError::Command(e.to_string())),
            Err(_elapsed) => Err(SandboxError::Timeout(timeout_ms)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use lithify_core::Sandbox;

    /// Helper: create a sandbox with a short timeout for testing.
    fn test_sandbox() -> SandboxImpl {
        SandboxImpl::new(Duration::from_secs(5))
    }

    // ---- T1: echo hello - normal path ----

    #[tokio::test]
    async fn t1_echo_hello() {
        let sb = test_sandbox();
        let result = sb.run("echo", &["hello".into()], None).await.unwrap();

        assert_eq!(result.exit_code, 0);
        assert!(result.stdout.contains("hello"));
        assert_eq!(result.stderr, "");
    }

    // ---- T2: non-zero exit code ----

    #[tokio::test]
    async fn t2_non_zero_exit() {
        let sb = test_sandbox();
        let result = sb
            .run("bash", &["-c".into(), "exit 3".into()], None)
            .await
            .unwrap();

        assert_eq!(result.exit_code, 3);
    }

    // ---- T3: timeout ----

    #[tokio::test]
    async fn t3_timeout() {
        let sb = SandboxImpl::new(Duration::from_secs(1));
        let result = sb.run("sleep", &["10".into()], None).await;

        assert!(matches!(result, Err(SandboxError::Timeout(_))));
    }

    // ---- T4: nonexistent command ----

    #[tokio::test]
    async fn t4_nonexistent_command() {
        let sb = test_sandbox();
        let result = sb.run("nonexistent_command_xyz", &[], None).await;

        assert!(matches!(result, Err(SandboxError::Command(_))));
    }

    // ---- T5: working directory ----

    #[tokio::test]
    async fn t5_working_directory() {
        let sb = test_sandbox();
        let result = sb.run("pwd", &[], Some(Path::new("/tmp"))).await.unwrap();

        assert_eq!(result.exit_code, 0);
        assert!(result.stdout.contains("/tmp"));
    }

    // ---- T6: stderr capture ----

    #[tokio::test]
    async fn t6_stderr_capture() {
        let sb = test_sandbox();
        let result = sb
            .run("bash", &["-c".into(), "echo error >&2".into()], None)
            .await
            .unwrap();

        assert_eq!(result.exit_code, 0);
        assert!(result.stderr.contains("error"));
        assert_eq!(result.stdout, "");
    }
}
