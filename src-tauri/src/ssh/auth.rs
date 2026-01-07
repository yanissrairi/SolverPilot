//! SSH authentication module
//!
//! Handles key loading, passphrase management, and authentication methods.
//! Supports public key authentication with optional passphrases.

use crate::config::AppConfig;
use russh::client::Handle;
use russh::keys::{load_secret_key, ssh_key, PrivateKey, PrivateKeyWithHashAlg};
use std::path::Path;
use std::sync::Arc;
use zeroize::ZeroizeOnDrop;

use super::error::{AuthMethod, Result, SshError};

/// Secure string that zeros its contents on drop
#[derive(Clone, ZeroizeOnDrop)]
pub struct SecureString(String);

impl SecureString {
    /// Create a new secure string
    pub fn new(s: impl Into<String>) -> Self {
        Self(s.into())
    }

    /// Get a reference to the inner string
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl From<String> for SecureString {
    fn from(s: String) -> Self {
        Self(s)
    }
}

impl From<&str> for SecureString {
    fn from(s: &str) -> Self {
        Self(s.to_string())
    }
}

/// SSH authentication configuration
#[derive(Clone)]
pub enum SshAuth {
    /// Public key authentication with optional passphrase
    Key {
        path: String,
        passphrase: Option<SecureString>,
    },
    /// SSH agent authentication
    Agent { key_path: String },
}

impl SshAuth {
    /// Create key-based authentication without passphrase
    pub fn key(path: impl Into<String>) -> Self {
        Self::Key {
            path: path.into(),
            passphrase: None,
        }
    }

    /// Create key-based authentication with passphrase
    pub fn key_with_passphrase(
        path: impl Into<String>,
        passphrase: impl Into<SecureString>,
    ) -> Self {
        Self::Key {
            path: path.into(),
            passphrase: Some(passphrase.into()),
        }
    }

    /// Create SSH agent authentication
    pub fn agent(key_path: impl Into<String>) -> Self {
        Self::Agent {
            key_path: key_path.into(),
        }
    }

    /// Load the private key from disk
    pub fn load_key(&self) -> Result<PrivateKey> {
        match self {
            Self::Key { path, passphrase } => {
                let expanded_path = expand_tilde(path);

                // Check if key file exists
                if !Path::new(&expanded_path).exists() {
                    return Err(SshError::key_error(path, "Key file does not exist"));
                }

                // Load key with optional passphrase
                let passphrase_str = passphrase.as_ref().map(SecureString::as_str);
                load_secret_key(&expanded_path, passphrase_str)
                    .map_err(|e| SshError::key_error(path, format!("Failed to load key: {e}")))
            }
            Self::Agent { key_path } => {
                // For agent mode, we still need to load the public key to identify it
                let expanded_path = expand_tilde(key_path);

                if !Path::new(&expanded_path).exists() {
                    return Err(SshError::key_error(key_path, "Key file does not exist"));
                }

                // Load without passphrase (agent handles it)
                load_secret_key(&expanded_path, None).map_err(|e| {
                    SshError::key_error(key_path, format!("Failed to load key for agent: {e}"))
                })
            }
        }
    }
}

/// SSH client handler for russh
pub struct SshHandler;

impl SshHandler {
    /// Create a new SSH handler
    pub const fn new() -> Self {
        Self
    }
}

impl Default for SshHandler {
    fn default() -> Self {
        Self::new()
    }
}

impl russh::client::Handler for SshHandler {
    type Error = russh::Error;

    async fn check_server_key(
        &mut self,
        _server_public_key: &ssh_key::PublicKey,
    ) -> std::result::Result<bool, Self::Error> {
        // TODO: Implement proper known_hosts verification
        // For now, accept all keys (same as StrictHostKeyChecking=accept-new)
        Ok(true)
    }
}

/// Authenticate a session with the configured method
pub async fn authenticate_session<H: russh::client::Handler>(
    session: &mut Handle<H>,
    config: &AppConfig,
    auth: &SshAuth,
) -> Result<()> {
    tracing::info!("Authenticating as user '{}'", config.ssh.user);

    // Load the private key
    tracing::debug!("Loading SSH private key");
    let key_pair = auth.load_key().map_err(|e| {
        tracing::error!("Failed to load SSH key: {}", e);
        e
    })?;

    tracing::debug!("SSH key loaded successfully");

    // Get the best supported RSA hash algorithm if using RSA
    let rsa_hash = session.best_supported_rsa_hash().await?.flatten();

    // Create key with hash algorithm
    let key_with_hash = PrivateKeyWithHashAlg::new(Arc::new(key_pair), rsa_hash);

    // Attempt authentication
    tracing::debug!("Attempting public key authentication");
    let auth_result = session
        .authenticate_publickey(&config.ssh.user, key_with_hash)
        .await?;

    if !auth_result.success() {
        tracing::error!("SSH authentication failed for user '{}'", config.ssh.user);
        return Err(SshError::auth_failed(
            &config.ssh.user,
            AuthMethod::PublicKey,
            "Server rejected authentication",
        ));
    }

    tracing::info!("SSH authentication successful");
    Ok(())
}

/// Get SSH key path from config, with auto-detection from ~/.ssh/config
pub fn get_ssh_key_path(config: &AppConfig) -> String {
    // If explicitly configured and not default, use it
    let default_key = "~/.ssh/id_rsa";
    if config.ssh.key_path != default_key && !config.ssh.key_path.is_empty() {
        return expand_tilde(&config.ssh.key_path);
    }

    // Try to auto-detect from SSH config
    if let Some(identity_file) = get_identity_file_from_ssh_config(&config.ssh.host) {
        tracing::info!(
            "Auto-detected SSH key from ~/.ssh/config: {}",
            identity_file
        );
        return identity_file;
    }

    // Fallback to configured or default
    expand_tilde(&config.ssh.key_path)
}

/// Parse ~/.ssh/config to find `IdentityFile` for a host
fn get_identity_file_from_ssh_config(host: &str) -> Option<String> {
    let home = std::env::var("HOME").ok()?;
    let config_path = format!("{home}/.ssh/config");
    let content = std::fs::read_to_string(&config_path).ok()?;

    let mut in_matching_host = false;

    for line in content.lines() {
        let trimmed = line.trim();

        // Skip empty lines and comments
        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }

        // Check for Host directives
        if let Some(hosts_str) = trimmed.strip_prefix("Host ") {
            let hosts: Vec<&str> = hosts_str.split_whitespace().collect();
            in_matching_host = hosts.iter().any(|h| *h == host || *h == "*");
            continue;
        }

        // Extract IdentityFile if in matching host block
        if in_matching_host {
            if let Some(identity_file) = trimmed.strip_prefix("IdentityFile ") {
                return Some(expand_tilde(identity_file.trim()));
            }
        }
    }

    None
}

/// Expand tilde (~) in paths to home directory
fn expand_tilde(path: &str) -> String {
    if path.starts_with("~/") {
        if let Ok(home) = std::env::var("HOME") {
            return path.replacen('~', &home, 1);
        }
    } else if path.starts_with('~') && path.len() > 1 {
        // Handle ~user/ paths (basic support)
        if let Some(slash_pos) = path.find('/') {
            let username = &path[1..slash_pos];
            // This is a simplification - proper implementation would use getpwnam
            if let Ok(home_base) = std::env::var("HOME") {
                if let Some(parent) = Path::new(&home_base).parent() {
                    let user_home = parent.join(username);
                    let user_home_display = user_home.display();
                    let path_suffix = &path[slash_pos..];
                    return format!("{user_home_display}{path_suffix}");
                }
            }
        }
    }

    path.to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_secure_string_zeroizes() {
        let original = "super_secret_passphrase";
        let secure = SecureString::new(original.to_string());
        assert_eq!(secure.as_str(), original);

        // After drop, the string should be zeroed
        drop(secure);
        // Note: We can't actually verify zeroing without unsafe code,
        // but the ZeroizeOnDrop trait ensures it happens
    }

    #[test]
    fn test_expand_tilde() {
        // Test basic tilde expansion
        if let Ok(home) = std::env::var("HOME") {
            assert_eq!(expand_tilde("~/test"), format!("{home}/test"));
            assert_eq!(expand_tilde(&home), home);
        }

        // Test non-tilde paths pass through
        assert_eq!(expand_tilde("/absolute/path"), "/absolute/path");
    }

    #[test]
    fn test_ssh_auth_creation() {
        let auth = SshAuth::key("/path/to/key");
        assert!(matches!(
            auth,
            SshAuth::Key {
                passphrase: None,
                ..
            }
        ));

        let auth = SshAuth::key_with_passphrase("/path/to/key", "passphrase");
        assert!(matches!(
            auth,
            SshAuth::Key {
                passphrase: Some(_),
                ..
            }
        ));

        let auth = SshAuth::agent("/path/to/key");
        assert!(matches!(auth, SshAuth::Agent { .. }));
    }
}
