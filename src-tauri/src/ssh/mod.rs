//! SSH client module with connection pooling
//!
//! This module provides a complete SSH client implementation using:
//! - russh: Pure Rust SSH2 protocol
//! - bb8: Connection pooling for parallel operations
//! - Modern security: Ed25519, ChaCha20-Poly1305

mod auth;
mod error;
mod executor;
mod pool;
mod transfer;

// Public exports
pub use auth::{get_ssh_key_path, SecureString, SshAuth};
pub use error::{AuthMethod, Result, SshError};
pub use executor::{CommandResult, SshExecutor};
pub use pool::SshPool;
pub use transfer::SshTransfer;

use crate::config::AppConfig;

/// Main SSH manager that coordinates all SSH operations
#[derive(Clone)]
pub struct SshManager {
    executor: SshExecutor,
    transfer: SshTransfer,
    pool: SshPool,
}

impl SshManager {
    /// Create and initialize a new SSH manager
    ///
    /// This creates a connection pool with the specified size and
    /// tests the initial connection.
    pub async fn new(config: AppConfig, auth: SshAuth, pool_size: u32) -> Result<Self> {
        tracing::info!(
            "Initializing SSH manager for {}:{} (pool size: {})",
            config.ssh.host,
            config.ssh.port,
            pool_size
        );

        // Create connection pool
        let pool = SshPool::new(config.clone(), auth, pool_size).await?;

        // Test connection immediately
        let conn = pool.get().await?;
        drop(conn); // Return to pool

        tracing::info!("SSH manager initialized successfully");

        Ok(Self {
            executor: SshExecutor::new(pool.clone()),
            transfer: SshTransfer::new(config.clone()),
            pool: pool.clone(),
        })
    }

    /// Get the command executor
    pub const fn executor(&self) -> &SshExecutor {
        &self.executor
    }

    /// Get the file transfer manager
    pub const fn transfer(&self) -> &SshTransfer {
        &self.transfer
    }

    /// Get pool state for monitoring
    pub fn pool_state(&self) -> bb8::State {
        self.pool.state()
    }

    /// Test SSH connection
    pub async fn test_connection(&self) -> Result<()> {
        self.executor.execute("echo ok").await?;
        Ok(())
    }

    /// Convenience: Execute a command
    pub async fn execute(&self, cmd: &str) -> Result<String> {
        self.executor.execute(cmd).await
    }

    /// Convenience: Execute and ignore exit status
    pub async fn execute_ignore_status(&self, cmd: &str) -> Result<String> {
        self.executor.execute_ignore_status(cmd).await
    }
}

/// Simple SSH connection test (for setup wizard)
///
/// This creates a temporary single connection without a pool,
/// authenticates, and executes a simple command to verify connectivity.
pub async fn test_connection_direct(config: &AppConfig, auth: &SshAuth) -> Result<()> {
    use russh::client::Config;
    use std::sync::Arc;
    use std::time::Duration;

    tracing::info!(
        "Testing direct SSH connection to {}:{} (timeout: 30s)",
        config.ssh.host,
        config.ssh.port
    );

    // Create minimal client config
    let client_config = Arc::new(Config {
        inactivity_timeout: Some(Duration::from_secs(30)),
        ..Default::default()
    });

    // Connect
    let handler = auth::SshHandler::new();
    let addr = (config.ssh.host.as_str(), config.ssh.port);

    tracing::debug!(
        "Starting TCP connection to {}:{}",
        config.ssh.host,
        config.ssh.port
    );

    let mut session = tokio::time::timeout(
        Duration::from_secs(30),
        russh::client::connect(client_config, &addr, handler),
    )
    .await
    .map_err(|_| {
        tracing::error!(
            "SSH connection to {}:{} timed out after 30 seconds",
            config.ssh.host,
            config.ssh.port
        );
        SshError::timeout("connection", 30)
    })?
    .map_err(|e| {
        tracing::error!(
            "SSH connection to {}:{} failed: {}",
            config.ssh.host,
            config.ssh.port,
            e
        );
        SshError::connection_failed(&config.ssh.host, config.ssh.port, e.to_string())
    })?;

    // Authenticate
    auth::authenticate_session(&mut session, config, auth).await?;

    // Test with simple command
    let mut channel = session.channel_open_session().await?;
    channel.exec(true, "echo ok").await?;

    let mut output = Vec::new();
    while let Some(msg) = channel.wait().await {
        if let russh::ChannelMsg::Data { data } = msg {
            output.extend_from_slice(&data);
        }
    }

    let result = String::from_utf8_lossy(&output);
    if result.trim() != "ok" {
        return Err(SshError::other("Connection test failed"));
    }

    // Disconnect cleanly
    session
        .disconnect(russh::Disconnect::ByApplication, "", "English")
        .await?;

    tracing::info!("Direct SSH connection test successful");
    Ok(())
}

/// Check SSH key status (for migration compatibility)
///
/// In the new architecture, we don't rely on ssh-agent the same way.
/// This function is kept for backward compatibility during migration.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(tag = "type", content = "data")]
pub enum SshKeyStatus {
    /// Key is ready to use
    Ready,
    /// Key needs passphrase
    NeedsPassphrase { key_path: String },
    /// Key file not found
    NoKey { expected_path: String },
}

/// Check if SSH key is available and ready
pub fn check_key_status(_config: &AppConfig, auth: &SshAuth) -> Result<SshKeyStatus> {
    match auth {
        SshAuth::Key { path, passphrase } => {
            let expanded = if path.starts_with("~/") {
                path.replacen(
                    '~',
                    &std::env::var("HOME").unwrap_or_else(|_| ".".to_string()),
                    1,
                )
            } else {
                path.clone()
            };

            if !std::path::Path::new(&expanded).exists() {
                return Ok(SshKeyStatus::NoKey {
                    expected_path: expanded,
                });
            }

            if passphrase.is_none() {
                // Try to load key to see if it needs passphrase
                match auth.load_key() {
                    Ok(_) => Ok(SshKeyStatus::Ready),
                    Err(_) => Ok(SshKeyStatus::NeedsPassphrase {
                        key_path: path.clone(),
                    }),
                }
            } else {
                Ok(SshKeyStatus::Ready)
            }
        }
        SshAuth::Agent { key_path } => {
            let expanded = if key_path.starts_with("~/") {
                key_path.replacen(
                    '~',
                    &std::env::var("HOME").unwrap_or_else(|_| ".".to_string()),
                    1,
                )
            } else {
                key_path.clone()
            };

            if !std::path::Path::new(&expanded).exists() {
                return Ok(SshKeyStatus::NoKey {
                    expected_path: expanded,
                });
            }
            Ok(SshKeyStatus::Ready)
        }
    }
}
