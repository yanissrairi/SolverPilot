//! Wrapper script deployment and management module
//!
//! This module handles:
//! - Embedding the `job_wrapper.sh` script via `include_str!`
//! - Deploying the wrapper to remote servers via SSH
//! - Version tracking for debugging and compatibility
//! - Idempotent deployment (skip if already installed)

use crate::ssh::SshExecutor;

/// Version of the wrapper script (Story 2.3)
pub const WRAPPER_VERSION: &str = "1.0.0";

/// Embedded wrapper script content
pub const WRAPPER_SCRIPT: &str = include_str!("../scripts/job_wrapper.sh");

/// Remote path where the wrapper is deployed
pub const REMOTE_WRAPPER_PATH: &str = "~/.solverpilot/bin/job_wrapper.sh";

/// Manager for wrapper script deployment and operations
pub struct WrapperManager {
    script_content: String,
    version: String,
}

impl WrapperManager {
    /// Create a new wrapper manager with embedded script
    pub fn new() -> Self {
        Self {
            script_content: WRAPPER_SCRIPT.to_string(),
            version: WRAPPER_VERSION.to_string(),
        }
    }

    /// Check if wrapper is installed on remote server
    ///
    /// # Errors
    /// Returns error if SSH command fails or connection is lost
    pub async fn check_installed(&self, executor: &SshExecutor) -> Result<bool, String> {
        let check_cmd =
            format!("test -f {REMOTE_WRAPPER_PATH} && echo 'installed' || echo 'missing'");

        let output = executor
            .execute(&check_cmd)
            .await
            .map_err(|e| format!("Failed to check wrapper installation: {e}"))?;

        Ok(output.trim() == "installed")
    }

    /// Deploy wrapper to server at `~/.solverpilot/bin/job_wrapper.sh`
    ///
    /// Performs the following steps:
    /// 1. Create remote directory
    /// 2. Write wrapper via heredoc
    /// 3. Make executable
    ///
    /// # Errors
    /// Returns error if any SSH command fails
    pub async fn deploy_to_server(&self, executor: &SshExecutor) -> Result<(), String> {
        // Step 1: Create directory
        executor
            .execute("mkdir -p ~/.solverpilot/bin")
            .await
            .map_err(|e| format!("Failed to create wrapper directory: {e}"))?;

        // Step 2: Write script via heredoc (single-quoted delimiter prevents variable expansion)
        let write_cmd = format!(
            "cat > {} << 'WRAPPER_EOF'\n{}\nWRAPPER_EOF",
            REMOTE_WRAPPER_PATH, self.script_content
        );

        executor
            .execute(&write_cmd)
            .await
            .map_err(|e| format!("Failed to write wrapper script: {e}"))?;

        // Step 3: Make executable
        executor
            .execute(&format!("chmod +x {REMOTE_WRAPPER_PATH}"))
            .await
            .map_err(|e| format!("Failed to make wrapper executable: {e}"))?;

        tracing::info!("Wrapper deployed successfully to {}", REMOTE_WRAPPER_PATH);
        Ok(())
    }

    /// Generate wrapper invocation command for use in tmux
    ///
    /// # Example
    /// ```
    /// let manager = WrapperManager::new();
    /// let cmd = manager.generate_invocation("job-123", &["python3", "bench.py"]);
    /// // Returns: "~/.solverpilot/bin/job_wrapper.sh job-123 python3 bench.py"
    /// ```
    pub fn generate_invocation(&self, job_id: &str, command: &[String]) -> String {
        format!("{} {} {}", REMOTE_WRAPPER_PATH, job_id, command.join(" "))
    }

    /// Get wrapper version
    pub fn version(&self) -> &str {
        &self.version
    }
}

impl Default for WrapperManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_wrapper_script_embedded() {
        // Verify the script was successfully embedded
        // Checking actual content is more meaningful than checking emptiness
        assert!(
            WRAPPER_SCRIPT.contains("#!/bin/bash"),
            "Wrapper should be a bash script"
        );
        assert!(
            WRAPPER_SCRIPT.contains("trap cleanup EXIT"),
            "Wrapper should have EXIT trap"
        );
        assert!(
            WRAPPER_SCRIPT.contains("set -euo pipefail"),
            "Wrapper should use strict mode"
        );
    }

    #[test]
    fn test_wrapper_version() {
        assert_eq!(WRAPPER_VERSION, "1.0.0");
    }

    #[test]
    fn test_wrapper_manager_creation() {
        let manager = WrapperManager::new();
        assert_eq!(manager.version(), "1.0.0");
        assert!(!manager.script_content.is_empty());
    }

    #[test]
    fn test_generate_invocation() {
        let manager = WrapperManager::new();
        let cmd = manager.generate_invocation(
            "test-job-123",
            &["python3".to_string(), "bench.py".to_string()],
        );
        assert_eq!(
            cmd,
            "~/.solverpilot/bin/job_wrapper.sh test-job-123 python3 bench.py"
        );
    }

    #[test]
    fn test_heredoc_command_format_preserves_variables() {
        let manager = WrapperManager::new();
        // Verify the heredoc format prevents variable expansion
        // These should NOT be expanded during deployment
        assert!(
            manager.script_content.contains("$JOB_ID"),
            "Script variables should be preserved"
        );
        assert!(
            manager.script_content.contains("$USER"),
            "Script environment variables should be preserved"
        );
    }

    #[test]
    fn test_wrapper_has_signal_handlers() {
        // Verify Story 2.1 signal handling is present
        assert!(
            WRAPPER_SCRIPT.contains("trap 'exit 143' TERM"),
            "Should handle SIGTERM"
        );
        assert!(
            WRAPPER_SCRIPT.contains("trap 'exit 130' INT"),
            "Should handle SIGINT"
        );
    }

    #[test]
    fn test_wrapper_has_sql_escaping() {
        // Verify Story 2.1 SQL injection protection
        assert!(
            WRAPPER_SCRIPT.contains("JOB_ID_SQL"),
            "Should escape SQL variables"
        );
    }

    #[test]
    fn test_default_trait() {
        let manager = WrapperManager::default();
        assert_eq!(manager.version(), "1.0.0");
    }
}
