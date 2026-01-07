//! File transfer via rsync over SSH
//!
//! For now, this uses rsync CLI for reliability.
//! Future: Migrate to russh-sftp for pure Rust implementation.

use crate::config::AppConfig;
use std::path::Path;
use tokio::process::Command;

use super::error::{Result, SshError};

/// SSH file transfer manager
#[derive(Clone)]
pub struct SshTransfer {
    config: AppConfig,
}

impl SshTransfer {
    /// Create a new file transfer manager
    pub fn new(config: AppConfig) -> Self {
        Self { config }
    }

    /// Build rsync SSH command string
    fn rsync_ssh_command(&self) -> String {
        let mut ssh_args = vec![
            "ssh".to_string(),
            "-o".to_string(),
            "StrictHostKeyChecking=accept-new".to_string(),
        ];

        if self.config.ssh.port != 22 {
            ssh_args.push("-p".to_string());
            ssh_args.push(self.config.ssh.port.to_string());
        }

        ssh_args.join(" ")
    }

    /// Sync project files to remote server
    pub async fn rsync_project(&self, project_name: &str, project_dir: &Path) -> Result<()> {
        let remote_path = format!(
            "{}@{}:{}/projects/{}/",
            self.config.ssh.user,
            self.config.ssh.host,
            self.config.remote.remote_base,
            project_name
        );

        let local_path = format!("{}/", project_dir.display());

        self.rsync_files(
            &local_path,
            &remote_path,
            vec!["pyproject.toml", "uv.lock", ".python-version"],
        )
        .await
    }

    /// Sync benchmark files to remote server
    pub async fn rsync_benchmarks(
        &self,
        project_name: &str,
        benchmark_dir: &Path,
        files: Vec<String>,
    ) -> Result<()> {
        let remote_path = format!(
            "{}@{}:{}/projects/{}/code/",
            self.config.ssh.user,
            self.config.ssh.host,
            self.config.remote.remote_base,
            project_name
        );

        // Create temporary file list for rsync --files-from
        let temp_dir = std::env::temp_dir();
        let files_list_path = temp_dir.join(format!("rsync_files_{}.txt", std::process::id()));

        // Convert to relative paths
        let mut relative_files = Vec::new();
        for file in &files {
            let file_path = Path::new(file);
            if let Ok(rel) = file_path.strip_prefix(benchmark_dir) {
                relative_files.push(rel.display().to_string());
            }
        }

        // Write file list
        std::fs::write(&files_list_path, relative_files.join("\n")).map_err(|e| {
            SshError::TransferError {
                source: "file list".to_string(),
                destination: files_list_path.display().to_string(),
                reason: format!("Failed to write file list: {}", e),
            }
        })?;

        let local_path = format!("{}/", benchmark_dir.display());

        let result = self
            .rsync_from_file_list(
                &local_path,
                &remote_path,
                &files_list_path.display().to_string(),
            )
            .await;

        // Cleanup
        let _ = std::fs::remove_file(&files_list_path);

        result
    }

    /// Download files from remote server
    pub async fn rsync_from_server(&self, remote_path: &str, local_path: &str) -> Result<()> {
        let remote = format!(
            "{}@{}:{}",
            self.config.ssh.user, self.config.ssh.host, remote_path
        );

        let ssh_cmd = self.rsync_ssh_command();

        let mut command = Command::new("rsync");
        command.args(["-avz", "-e", &ssh_cmd, &remote, local_path]);

        let output = command
            .output()
            .await
            .map_err(|e| SshError::TransferError {
                source: remote_path.to_string(),
                destination: local_path.to_string(),
                reason: format!("Failed to execute rsync: {}", e),
            })?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(SshError::TransferError {
                source: remote_path.to_string(),
                destination: local_path.to_string(),
                reason: format!("rsync failed: {}", stderr),
            });
        }

        Ok(())
    }

    /// Internal: rsync with specific includes
    async fn rsync_files(
        &self,
        local_path: &str,
        remote_path: &str,
        includes: Vec<&str>,
    ) -> Result<()> {
        let ssh_cmd = self.rsync_ssh_command();

        let mut command = Command::new("rsync");
        command.args(["-avz"]);

        // Add include patterns
        for include in &includes {
            command.arg("--include").arg(include);
        }
        command.arg("--exclude").arg("*");

        command.args(["-e", &ssh_cmd, local_path, remote_path]);

        let output = command
            .output()
            .await
            .map_err(|e| SshError::TransferError {
                source: local_path.to_string(),
                destination: remote_path.to_string(),
                reason: format!("Failed to execute rsync: {}", e),
            })?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(SshError::TransferError {
                source: local_path.to_string(),
                destination: remote_path.to_string(),
                reason: format!("rsync failed: {}", stderr),
            });
        }

        Ok(())
    }

    /// Internal: rsync using --files-from
    async fn rsync_from_file_list(
        &self,
        local_path: &str,
        remote_path: &str,
        files_list: &str,
    ) -> Result<()> {
        let ssh_cmd = self.rsync_ssh_command();

        let mut command = Command::new("rsync");
        command.args([
            "-avz",
            "--files-from",
            files_list,
            "-e",
            &ssh_cmd,
            local_path,
            remote_path,
        ]);

        let output = command
            .output()
            .await
            .map_err(|e| SshError::TransferError {
                source: local_path.to_string(),
                destination: remote_path.to_string(),
                reason: format!("Failed to execute rsync: {}", e),
            })?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(SshError::TransferError {
                source: local_path.to_string(),
                destination: remote_path.to_string(),
                reason: format!("rsync failed: {}", stderr),
            });
        }

        Ok(())
    }

    /// Dry-run to check which files would be transferred
    pub async fn dry_run_project(
        &self,
        project_name: &str,
        project_dir: &Path,
    ) -> Result<Vec<String>> {
        let remote_path = format!(
            "{}@{}:{}/projects/{}/",
            self.config.ssh.user,
            self.config.ssh.host,
            self.config.remote.remote_base,
            project_name
        );

        let local_path = format!("{}/", project_dir.display());
        let ssh_cmd = self.rsync_ssh_command();

        let mut command = Command::new("rsync");
        command.args([
            "-avzn",
            "--include=pyproject.toml",
            "--include=uv.lock",
            "--include=.python-version",
            "--exclude=*",
            "--out-format=%n",
            "-e",
            &ssh_cmd,
            &local_path,
            &remote_path,
        ]);

        let output = command
            .output()
            .await
            .map_err(|e| SshError::TransferError {
                source: local_path.clone(),
                destination: remote_path.clone(),
                reason: format!("Failed to execute rsync dry-run: {}", e),
            })?;

        if !output.status.success() {
            return Err(SshError::TransferError {
                source: local_path,
                destination: remote_path,
                reason: "rsync dry-run failed".to_string(),
            });
        }

        let stdout = String::from_utf8_lossy(&output.stdout);
        let files: Vec<String> = stdout
            .lines()
            .filter(|line| !line.is_empty() && !line.ends_with('/'))
            .map(|line| line.to_string())
            .collect();

        Ok(files)
    }
}
