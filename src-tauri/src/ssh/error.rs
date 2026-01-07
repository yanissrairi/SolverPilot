//! Error types for SSH operations
//!
//! Provides structured error types to replace `Result<T, String>` throughout
//! the SSH module, enabling better error handling and debugging.

use std::fmt;

/// Result type alias for SSH operations
pub type Result<T> = std::result::Result<T, SshError>;

/// Structured error type for all SSH operations
#[derive(Debug)]
pub enum SshError {
    /// Connection failed (network, timeout, etc.)
    ConnectionFailed {
        host: String,
        port: u16,
        reason: String,
    },

    /// Authentication failed
    AuthenticationFailed {
        username: String,
        method: AuthMethod,
        reason: String,
    },

    /// Key loading/parsing error
    KeyError { path: String, reason: String },

    /// Channel operation failed (exec, shell, etc.)
    ChannelError { operation: String, reason: String },

    /// Command execution failed
    CommandFailed {
        command: String,
        exit_code: Option<u32>,
        stderr: String,
    },

    /// File transfer error
    TransferError {
        source: String,
        destination: String,
        reason: String,
    },

    /// Connection pool error
    PoolError { reason: String },

    /// Configuration error
    ConfigError { field: String, reason: String },

    /// Timeout error
    Timeout {
        operation: String,
        duration_secs: u64,
    },

    /// Underlying russh error
    RusshError(russh::Error),

    /// I/O error
    IoError(std::io::Error),

    /// Generic error with context
    Other {
        context: String,
        source: Option<Box<dyn std::error::Error + Send + Sync>>,
    },
}

/// Authentication method enum for error reporting
#[derive(Debug, Clone, Copy)]
pub enum AuthMethod {
    PublicKey,
    Password,
    Agent,
    Certificate,
}

impl fmt::Display for AuthMethod {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::PublicKey => write!(f, "public key"),
            Self::Password => write!(f, "password"),
            Self::Agent => write!(f, "ssh-agent"),
            Self::Certificate => write!(f, "certificate"),
        }
    }
}

impl fmt::Display for SshError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::ConnectionFailed { host, port, reason } => {
                write!(f, "Connection to {}:{} failed: {}", host, port, reason)
            }
            Self::AuthenticationFailed {
                username,
                method,
                reason,
            } => {
                write!(
                    f,
                    "Authentication failed for user '{}' with {}: {}",
                    username, method, reason
                )
            }
            Self::KeyError { path, reason } => {
                write!(f, "Key error for '{}': {}", path, reason)
            }
            Self::ChannelError { operation, reason } => {
                write!(f, "Channel operation '{}' failed: {}", operation, reason)
            }
            Self::CommandFailed {
                command,
                exit_code,
                stderr,
            } => {
                if let Some(code) = exit_code {
                    write!(
                        f,
                        "Command '{}' failed with exit code {}: {}",
                        command, code, stderr
                    )
                } else {
                    write!(f, "Command '{}' failed: {}", command, stderr)
                }
            }
            Self::TransferError {
                source,
                destination,
                reason,
            } => {
                write!(
                    f,
                    "Transfer from '{}' to '{}' failed: {}",
                    source, destination, reason
                )
            }
            Self::PoolError { reason } => {
                write!(f, "Connection pool error: {}", reason)
            }
            Self::ConfigError { field, reason } => {
                write!(f, "Configuration error in '{}': {}", field, reason)
            }
            Self::Timeout {
                operation,
                duration_secs,
            } => {
                write!(
                    f,
                    "Operation '{}' timed out after {} seconds",
                    operation, duration_secs
                )
            }
            Self::RusshError(e) => write!(f, "SSH protocol error: {}", e),
            Self::IoError(e) => write!(f, "I/O error: {}", e),
            Self::Other { context, source } => {
                if let Some(src) = source {
                    write!(f, "{}: {}", context, src)
                } else {
                    write!(f, "{}", context)
                }
            }
        }
    }
}

impl std::error::Error for SshError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::RusshError(e) => Some(e),
            Self::IoError(e) => Some(e),
            Self::Other {
                source: Some(src), ..
            } => Some(src.as_ref()),
            _ => None,
        }
    }
}

// Conversions from underlying errors
impl From<russh::Error> for SshError {
    fn from(err: russh::Error) -> Self {
        Self::RusshError(err)
    }
}

impl From<std::io::Error> for SshError {
    fn from(err: std::io::Error) -> Self {
        Self::IoError(err)
    }
}

impl From<russh_keys::Error> for SshError {
    fn from(err: russh_keys::Error) -> Self {
        Self::KeyError {
            path: "unknown".to_string(),
            reason: err.to_string(),
        }
    }
}

// Conversion to String for Tauri commands (backwards compatibility)
impl From<SshError> for String {
    fn from(err: SshError) -> Self {
        err.to_string()
    }
}

// Helper methods for creating specific errors
impl SshError {
    /// Create a connection failed error
    pub fn connection_failed(
        host: impl Into<String>,
        port: u16,
        reason: impl Into<String>,
    ) -> Self {
        Self::ConnectionFailed {
            host: host.into(),
            port,
            reason: reason.into(),
        }
    }

    /// Create an authentication failed error
    pub fn auth_failed(
        username: impl Into<String>,
        method: AuthMethod,
        reason: impl Into<String>,
    ) -> Self {
        Self::AuthenticationFailed {
            username: username.into(),
            method,
            reason: reason.into(),
        }
    }

    /// Create a key error
    pub fn key_error(path: impl Into<String>, reason: impl Into<String>) -> Self {
        Self::KeyError {
            path: path.into(),
            reason: reason.into(),
        }
    }

    /// Create a command failed error
    pub fn command_failed(
        command: impl Into<String>,
        exit_code: Option<u32>,
        stderr: impl Into<String>,
    ) -> Self {
        Self::CommandFailed {
            command: command.into(),
            exit_code,
            stderr: stderr.into(),
        }
    }

    /// Create a timeout error
    pub fn timeout(operation: impl Into<String>, duration_secs: u64) -> Self {
        Self::Timeout {
            operation: operation.into(),
            duration_secs,
        }
    }

    /// Create a generic error with context
    pub fn other(context: impl Into<String>) -> Self {
        Self::Other {
            context: context.into(),
            source: None,
        }
    }

    /// Add context to an existing error
    pub fn with_context(self, context: impl Into<String>) -> Self {
        Self::Other {
            context: context.into(),
            source: Some(Box::new(self)),
        }
    }
}
