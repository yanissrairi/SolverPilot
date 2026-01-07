//! Remote command execution via SSH
//!
//! Provides high-level APIs for executing commands on remote servers
//! with proper output capture, error handling, and timeout support.

use russh::ChannelMsg;
use std::time::Duration;

use super::error::{Result, SshError};
use super::pool::{SshConnection, SshPool};

/// Result of a command execution
#[derive(Debug, Clone)]
pub struct CommandResult {
    /// Standard output
    pub stdout: String,
    /// Standard error
    pub stderr: String,
    /// Exit code (0 = success)
    pub exit_code: u32,
}

impl CommandResult {
    /// Check if the command succeeded (exit code 0)
    pub fn success(&self) -> bool {
        self.exit_code == 0
    }

    /// Convert to Result, returning error if exit code is non-zero
    pub fn into_result(self) -> Result<String> {
        if self.success() {
            Ok(self.stdout)
        } else {
            Err(SshError::command_failed(
                "command",
                Some(self.exit_code),
                &self.stderr,
            ))
        }
    }
}

/// SSH command executor
#[derive(Clone)]
pub struct SshExecutor {
    pool: SshPool,
}

impl SshExecutor {
    /// Create a new executor from a connection pool
    pub fn new(pool: SshPool) -> Self {
        Self { pool }
    }

    /// Execute a command and return the result
    pub async fn execute_raw(&self, command: &str) -> Result<CommandResult> {
        let mut conn = self.pool.get().await?;
        execute_command_on_connection(&mut conn, command, Duration::from_secs(300)).await
    }

    /// Execute a command and return stdout on success
    pub async fn execute(&self, command: &str) -> Result<String> {
        self.execute_raw(command).await?.into_result()
    }

    /// Execute a command with custom timeout
    pub async fn execute_with_timeout(&self, command: &str, timeout: Duration) -> Result<String> {
        let mut conn = self.pool.get().await?;
        execute_command_on_connection(&mut conn, command, timeout)
            .await?
            .into_result()
    }

    /// Execute a command, ignoring exit status
    pub async fn execute_ignore_status(&self, command: &str) -> Result<String> {
        let result = self.execute_raw(command).await?;
        Ok(result.stdout)
    }

    /// Execute multiple commands in parallel
    pub async fn execute_parallel(&self, commands: Vec<&str>) -> Vec<Result<String>> {
        let futures: Vec<_> = commands.into_iter().map(|cmd| self.execute(cmd)).collect();

        futures::future::join_all(futures).await
    }

    /// Execute a command that starts a long-running process
    pub async fn execute_background(&self, command: &str) -> Result<()> {
        let conn = self.pool.get().await?;
        let channel = conn.channel_open_session().await?;

        channel.exec(true, command).await?;
        drop(channel);

        Ok(())
    }

    /// Check if a tmux session exists
    pub async fn tmux_session_exists(&self, session_name: &str) -> Result<bool> {
        let cmd = format!(
            "tmux has-session -t {} 2>/dev/null && echo yes || echo no",
            session_name
        );
        let output = self.execute_ignore_status(&cmd).await?;
        Ok(output.trim() == "yes")
    }

    /// Send Ctrl-C to a tmux session
    pub async fn tmux_send_ctrl_c(&self, session_name: &str) -> Result<()> {
        let cmd = format!("tmux send-keys -t {} C-c", session_name);
        self.execute_ignore_status(&cmd).await?;
        Ok(())
    }

    /// Kill a tmux session
    pub async fn tmux_kill_session(&self, session_name: &str) -> Result<()> {
        let cmd = format!("tmux kill-session -t {}", session_name);
        self.execute_ignore_status(&cmd).await?;
        Ok(())
    }

    /// Get job logs using tail
    pub async fn tail_logs(&self, log_file: &str, lines: u32) -> Result<String> {
        let cmd = format!("tail -n {} {} 2>/dev/null || echo ''", lines, log_file);
        self.execute_ignore_status(&cmd).await
    }
}

/// Execute a command on an existing connection
async fn execute_command_on_connection(
    conn: &mut SshConnection,
    command: &str,
    timeout: Duration,
) -> Result<CommandResult> {
    tokio::time::timeout(timeout, async {
        let mut channel =
            conn.channel_open_session()
                .await
                .map_err(|e| SshError::ChannelError {
                    operation: "open_session".to_string(),
                    reason: e.to_string(),
                })?;

        channel
            .exec(true, command)
            .await
            .map_err(|e| SshError::ChannelError {
                operation: "exec".to_string(),
                reason: e.to_string(),
            })?;

        let mut stdout_data = Vec::new();
        let mut stderr_data = Vec::new();
        let mut exit_code = None;

        while let Some(msg) = channel.wait().await {
            match msg {
                ChannelMsg::Data { data } => {
                    stdout_data.extend_from_slice(&data);
                }
                ChannelMsg::ExtendedData { data, ext } => {
                    if ext == 1 {
                        stderr_data.extend_from_slice(&data);
                    }
                }
                ChannelMsg::ExitStatus { exit_status } => {
                    exit_code = Some(exit_status);
                }
                ChannelMsg::Eof => {}
                ChannelMsg::Close => {
                    break;
                }
                _ => {}
            }
        }

        let stdout = String::from_utf8_lossy(&stdout_data).to_string();
        let stderr = String::from_utf8_lossy(&stderr_data).to_string();

        Ok(CommandResult {
            stdout,
            stderr,
            exit_code: exit_code.unwrap_or(255),
        })
    })
    .await
    .map_err(|_| SshError::timeout("command execution", timeout.as_secs()))?
}
