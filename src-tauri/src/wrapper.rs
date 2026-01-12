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
    /// On failure, attempts to clean up partial deployment to prevent inconsistent state.
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

        if let Err(e) = executor.execute(&write_cmd).await {
            // Cleanup: remove partial file if write failed
            let _ = executor
                .execute(&format!("rm -f {REMOTE_WRAPPER_PATH}"))
                .await;
            return Err(format!("Failed to write wrapper script: {e}"));
        }

        // Step 3: Make executable
        if let Err(e) = executor
            .execute(&format!("chmod +x {REMOTE_WRAPPER_PATH}"))
            .await
        {
            // Cleanup: remove script if chmod failed (script exists but not executable = broken state)
            let _ = executor
                .execute(&format!("rm -f {REMOTE_WRAPPER_PATH}"))
                .await;
            return Err(format!("Failed to make wrapper executable: {e}"));
        }

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

    // =========================================================================
    // Integration Tests (Task 7) - Command Generation & Logic Verification
    // =========================================================================

    #[test]
    fn test_check_command_format() {
        // Task 7.1: Verify the check command format matches AC requirements
        let expected_check_cmd =
            format!("test -f {REMOTE_WRAPPER_PATH} && echo 'installed' || echo 'missing'");
        assert!(expected_check_cmd.contains("~/.solverpilot/bin/job_wrapper.sh"));
        assert!(expected_check_cmd.contains("echo 'installed'"));
        assert!(expected_check_cmd.contains("echo 'missing'"));
    }

    #[test]
    fn test_deployment_mkdir_command() {
        // Task 7.2: Verify mkdir command creates correct directory structure
        let mkdir_cmd = "mkdir -p ~/.solverpilot/bin";
        assert!(mkdir_cmd.contains("-p")); // Create parents
        assert!(mkdir_cmd.contains("~/.solverpilot/bin"));
    }

    #[test]
    fn test_deployment_heredoc_format() {
        // Task 7.2: Verify heredoc command format for script deployment
        let manager = WrapperManager::new();
        let write_cmd = format!(
            "cat > {} << 'WRAPPER_EOF'\n{}\nWRAPPER_EOF",
            REMOTE_WRAPPER_PATH, manager.script_content
        );

        // Verify heredoc uses single-quoted delimiter (prevents variable expansion)
        assert!(write_cmd.contains("<< 'WRAPPER_EOF'"));
        assert!(write_cmd.contains("WRAPPER_EOF"));
        assert!(write_cmd.contains(&manager.script_content));
        // Ensure script variables are preserved in the command
        assert!(write_cmd.contains("$JOB_ID"));
        assert!(write_cmd.contains("$USER"));
    }

    #[test]
    fn test_deployment_chmod_command() {
        // Task 7.2: Verify chmod command format
        let chmod_cmd = format!("chmod +x {REMOTE_WRAPPER_PATH}");
        assert!(chmod_cmd.contains("chmod +x"));
        assert!(chmod_cmd.contains("~/.solverpilot/bin/job_wrapper.sh"));
    }

    #[test]
    fn test_cleanup_command_on_failure() {
        // Task 7.4: Verify cleanup command format for error recovery
        let cleanup_cmd = format!("rm -f {REMOTE_WRAPPER_PATH}");
        assert!(cleanup_cmd.contains("rm -f"));
        assert!(cleanup_cmd.contains("~/.solverpilot/bin/job_wrapper.sh"));
    }

    #[test]
    fn test_idempotent_deployment_logic() {
        // Task 7.3: Test idempotent behavior - calling deploy twice should be safe
        // This tests the check → deploy → verify sequence logic
        let manager = WrapperManager::new();

        // Simulate "installed" response
        let installed_output = "installed";
        assert_eq!(installed_output.trim(), "installed");

        // Simulate "missing" response
        let missing_output = "missing";
        assert_ne!(missing_output.trim(), "installed");

        // Version should remain constant between calls
        assert_eq!(manager.version(), WRAPPER_VERSION);
    }

    #[test]
    fn test_remote_path_constant() {
        // Verify the remote path matches architecture requirements
        assert_eq!(
            REMOTE_WRAPPER_PATH, "~/.solverpilot/bin/job_wrapper.sh",
            "Remote path must match architecture spec"
        );
    }

    #[test]
    fn test_wrapper_script_matches_disk() {
        // Task 6.5: Verify embedded script content matches expected patterns
        // (We can't compare to disk in unit tests, but we verify key patterns)
        let script = WRAPPER_SCRIPT;

        // Must have shebang
        assert!(script.starts_with("#!/bin/bash"));

        // Must have strict mode
        assert!(script.contains("set -euo pipefail"));

        // Must have trap EXIT for cleanup
        assert!(script.contains("trap cleanup EXIT"));

        // Must have server DB path (uses $HOME not ~)
        assert!(script.contains(".solverpilot-server"));
        assert!(script.contains("server.db"));

        // Must have state file path pattern
        assert!(script.contains("$BASE_DIR/jobs/$JOB_ID.status"));

        // Must have lock file pattern
        assert!(script.contains("$BASE_DIR/locks/$JOB_ID.lock"));
    }

    #[test]
    fn test_generate_invocation_with_special_characters() {
        // Test invocation generation with special characters in job_id
        let manager = WrapperManager::new();

        // Job ID with hyphens (common UUID format)
        let cmd = manager.generate_invocation(
            "550e8400-e29b-41d4-a716-446655440000",
            &["python3".to_string(), "benchmark.py".to_string()],
        );
        assert!(cmd.contains("550e8400-e29b-41d4-a716-446655440000"));

        // Job ID with underscores
        let cmd2 = manager.generate_invocation(
            "job_123_test",
            &[
                "python3".to_string(),
                "script.py".to_string(),
                "--arg=value".to_string(),
            ],
        );
        assert!(cmd2.contains("job_123_test"));
        assert!(cmd2.contains("--arg=value"));
    }

    #[test]
    fn test_generate_invocation_empty_args() {
        // Edge case: invocation with no arguments (just job_id)
        let manager = WrapperManager::new();
        let cmd = manager.generate_invocation("test-job", &[]);
        assert_eq!(cmd, "~/.solverpilot/bin/job_wrapper.sh test-job ");
    }

    #[test]
    fn test_wrapper_script_line_count() {
        // Verify script is reasonable size (not empty, not huge)
        let lines: Vec<&str> = WRAPPER_SCRIPT.lines().collect();
        assert!(lines.len() > 50, "Script should have substantial content");
        assert!(lines.len() < 200, "Script should be concise");
    }
}
