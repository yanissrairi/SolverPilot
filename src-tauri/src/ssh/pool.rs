//! SSH connection pool using bb8
//!
//! Provides a connection pool for SSH sessions with automatic health checks,
//! graceful connection recovery, and parallel operation support.

use crate::config::AppConfig;
use russh::client::{Config, Handle};
use std::sync::Arc;
use std::time::Duration;

use super::auth::{authenticate_session, SshAuth, SshHandler};
use super::error::{Result, SshError};

/// SSH connection type (Handle to an authenticated session)
pub type SshConnection = Handle<SshHandler>;

/// Connection manager for bb8 pool
pub struct SshConnectionManager {
    config: AppConfig,
    client_config: Arc<Config>,
    auth: SshAuth,
}

impl SshConnectionManager {
    /// Create a new connection manager
    pub fn new(config: AppConfig, auth: SshAuth) -> Result<Self> {
        let client_config = Arc::new(Self::create_client_config());
        Ok(Self {
            config,
            client_config,
            auth,
        })
    }

    /// Create russh client configuration with optimal settings
    fn create_client_config() -> Config {
        Config {
            // Inactivity timeout (5 minutes)
            inactivity_timeout: Some(Duration::from_secs(300)),

            // Keepalive interval (30 seconds)
            keepalive_interval: Some(Duration::from_secs(30)),

            // Use default secure algorithms (Ed25519, Curve25519, ChaCha20-Poly1305, AES-GCM)
            preferred: Default::default(),

            // Flow control settings
            window_size: 2_097_152,      // 2MB window
            maximum_packet_size: 32_768, // 32KB packets

            // Buffer sizes
            channel_buffer_size: 100,

            // Enable TCP_NODELAY for lower latency
            nodelay: true,

            ..Default::default()
        }
    }

    /// Get the connection address
    fn get_address(&self) -> (String, u16) {
        (self.config.ssh.host.clone(), self.config.ssh.port)
    }
}

impl bb8::ManageConnection for SshConnectionManager {
    type Connection = SshConnection;
    type Error = SshError;

    /// Create a new SSH connection
    async fn connect(&self) -> std::result::Result<Self::Connection, Self::Error> {
        let addr = self.get_address();
        let handler = SshHandler::new();

        // Connect to SSH server with timeout
        let mut session = tokio::time::timeout(
            Duration::from_secs(10),
            russh::client::connect(self.client_config.clone(), &addr, handler),
        )
        .await
        .map_err(|_| SshError::timeout("SSH connection", 10))?
        .map_err(|e| {
            SshError::connection_failed(&self.config.ssh.host, self.config.ssh.port, e.to_string())
        })?;

        // Authenticate
        authenticate_session(&mut session, &self.config, &self.auth).await?;

        tracing::debug!(
            "SSH connection established to {}:{}",
            self.config.ssh.host,
            self.config.ssh.port
        );

        Ok(session)
    }

    /// Check if a connection is still valid
    async fn is_valid(&self, conn: &mut Self::Connection) -> std::result::Result<(), Self::Error> {
        // Try to open a channel as a health check
        let result = tokio::time::timeout(Duration::from_secs(5), async {
            let channel = conn.channel_open_session().await?;
            drop(channel); // Close immediately
            Ok::<_, russh::Error>(())
        })
        .await;

        match result {
            Ok(Ok(())) => Ok(()),
            Ok(Err(e)) => {
                tracing::warn!("Connection health check failed: {}", e);
                Err(SshError::other(format!("Health check failed: {}", e)))
            }
            Err(_) => {
                tracing::warn!("Connection health check timed out");
                Err(SshError::timeout("Health check", 5))
            }
        }
    }

    /// Called when a connection has an error
    fn has_broken(&self, conn: &mut Self::Connection) -> bool {
        // Check if the connection is disconnected
        conn.is_closed()
    }
}

/// SSH connection pool
#[derive(Clone)]
pub struct SshPool {
    pool: bb8::Pool<SshConnectionManager>,
}

impl SshPool {
    /// Create a new SSH connection pool
    pub async fn new(config: AppConfig, auth: SshAuth, max_size: u32) -> Result<Self> {
        let manager = SshConnectionManager::new(config.clone(), auth)?;

        let pool = bb8::Pool::builder()
            .max_size(max_size)
            .connection_timeout(Duration::from_secs(10))
            .idle_timeout(Some(Duration::from_secs(300))) // 5 minutes idle timeout
            .max_lifetime(Some(Duration::from_secs(3600))) // 1 hour max lifetime
            .build(manager)
            .await
            .map_err(|e| SshError::PoolError {
                reason: format!("Failed to create pool: {}", e),
            })?;

        tracing::info!(
            "SSH connection pool created for {}:{} (max_size: {})",
            config.ssh.host,
            config.ssh.port,
            max_size
        );

        Ok(Self { pool })
    }

    /// Get a connection from the pool
    pub async fn get(&self) -> Result<bb8::PooledConnection<'_, SshConnectionManager>> {
        self.pool.get().await.map_err(|e| SshError::PoolError {
            reason: format!("Failed to get connection from pool: {}", e),
        })
    }

    /// Get pool state for monitoring
    pub fn state(&self) -> bb8::State {
        self.pool.state()
    }

    /// Dedicated connection for operations (convenience wrapper)
    pub async fn dedicated(&self) -> Result<bb8::PooledConnection<'_, SshConnectionManager>> {
        self.get().await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_client_config_creation() {
        let config = SshConnectionManager::create_client_config();

        // Verify timeouts are set
        assert!(config.inactivity_timeout.is_some());
        assert!(config.keepalive_interval.is_some());

        // Verify modern algorithms are preferred
        assert!(!config.preferred.kex.is_empty());
        assert!(!config.preferred.key.is_empty());
        assert!(!config.preferred.cipher.is_empty());

        // Verify TCP_NODELAY is enabled
        assert!(config.nodelay);
    }
}
