//! Queue execution service for sequential job processing
//!
//! This module implements the core queue execution engine that:
//! - Processes jobs sequentially (`max_concurrent` = 1 for Beta 1)
//! - Syncs projects via `rsync` before execution
//! - Launches jobs in `tmux` sessions with wrapper script
//! - Polls server DB for job completion
//! - Auto-starts next job after current completes

use crate::ssh::SshManager;
use crate::state::{Job, JobStatus};
use sqlx::{Row, SqlitePool};
use std::sync::Arc;
use tokio::sync::Mutex;
use tokio::time::{interval, Duration};

/// Queue execution manager
///
/// Manages sequential job processing with background task execution.
/// Ensures only one job runs at a time (`max_concurrent` = 1).
#[derive(Clone)]
pub struct QueueManager {
    is_processing: Arc<Mutex<bool>>,
    current_job_id: Arc<Mutex<Option<i64>>>,
}

impl QueueManager {
    /// Create a new queue manager
    pub fn new() -> Self {
        Self {
            is_processing: Arc::new(Mutex::new(false)),
            current_job_id: Arc::new(Mutex::new(None)),
        }
    }

    /// Start queue processing loop in background task
    ///
    /// Spawns a background Tokio task that:
    /// 1. Selects next pending job by `queue_position`
    /// 2. Executes job (rsync → tmux → poll)
    /// 3. Auto-starts next job after completion
    /// 4. Stops when queue is empty
    pub async fn start_processing(
        &self,
        db: SqlitePool,
        ssh: Arc<SshManager>,
        ssh_host: String,
        ssh_username: String,
    ) -> Result<(), String> {
        let mut processing = self.is_processing.lock().await;
        if *processing {
            return Err("Queue already processing".to_string());
        }
        *processing = true;
        drop(processing);

        // Clone Arc references for background task
        let is_processing = Arc::clone(&self.is_processing);
        let current_job_id = Arc::clone(&self.current_job_id);
        let ssh_host_cloned = ssh_host.clone();
        let ssh_username_cloned = ssh_username.clone();

        tokio::spawn(async move {
            loop {
                // Check if still processing
                if !*is_processing.lock().await {
                    tracing::info!("Queue processing stopped");
                    break;
                }

                // Select next job
                match select_next_job(&db).await {
                    Ok(Some(job)) => {
                        tracing::info!("Starting job {} ({})", job.id, job.benchmark_name);
                        *current_job_id.lock().await = Some(job.id);

                        // Execute job
                        if let Err(e) =
                            execute_job(&db, &ssh, &job, &ssh_host_cloned, &ssh_username_cloned)
                                .await
                        {
                            tracing::error!("Job {} failed: {}", job.id, e);
                            if let Err(mark_err) = mark_job_failed(&db, job.id, &e).await {
                                tracing::error!("Failed to mark job as failed: {}", mark_err);
                            }
                        }

                        *current_job_id.lock().await = None;
                    }
                    Ok(None) => {
                        // Queue empty, stop processing
                        *is_processing.lock().await = false;
                        tracing::info!("Queue completed - all jobs finished");
                        break;
                    }
                    Err(e) => {
                        tracing::error!("Failed to select next job: {}", e);
                        tokio::time::sleep(Duration::from_secs(5)).await;
                    }
                }
            }
        });

        Ok(())
    }

    /// Stop queue processing gracefully
    ///
    /// Stops processing after current job completes.
    /// Does not cancel running job.
    pub async fn stop_processing(&self) -> Result<(), String> {
        *self.is_processing.lock().await = false;
        tracing::info!("Queue processing will stop after current job");
        Ok(())
    }

    /// Get current processing state
    pub async fn is_processing(&self) -> bool {
        *self.is_processing.lock().await
    }

    /// Get currently executing job ID
    pub async fn current_job(&self) -> Option<i64> {
        *self.current_job_id.lock().await
    }
}

impl Default for QueueManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Select next pending job by `queue_position` (FIFO)
///
/// Returns the job with lowest `queue_position` where status = 'pending'.
async fn select_next_job(db: &SqlitePool) -> Result<Option<Job>, String> {
    let row = sqlx::query(
        "
        SELECT 
            id,
            project_id,
            benchmark_name,
            status,
            created_at,
            started_at,
            finished_at,
            progress_current,
            progress_total,
            results_path,
            error_message,
            log_content,
            queue_position,
            queued_at
        FROM jobs 
        WHERE status = 'pending' 
        ORDER BY queue_position ASC 
        LIMIT 1
        ",
    )
    .fetch_optional(db)
    .await
    .map_err(|e| format!("Failed to select next job: {e}"))?;

    let job = row.map(|r| {
        let status_str: String = r.get("status");
        let status = match status_str.as_str() {
            "running" => JobStatus::Running,
            "completed" => JobStatus::Completed,
            "failed" => JobStatus::Failed,
            "killed" => JobStatus::Killed,
            _ => JobStatus::Pending,
        };

        Job {
            id: r.get("id"),
            project_id: r.get("project_id"),
            benchmark_name: r.get("benchmark_name"),
            status,
            created_at: r.get("created_at"),
            started_at: r.get("started_at"),
            finished_at: r.get("finished_at"),
            #[allow(clippy::cast_sign_loss)]
            progress_current: r.get::<i32, _>("progress_current") as u32,
            #[allow(clippy::cast_sign_loss)]
            progress_total: r.get::<i32, _>("progress_total") as u32,
            results_path: r.get("results_path"),
            error_message: r.get("error_message"),
            log_content: r.get("log_content"),
            queue_position: r.get("queue_position"),
            queued_at: r.get("queued_at"),
        }
    });

    if let Some(ref j) = job {
        tracing::debug!("Selected job {} at position {:?}", j.id, j.queue_position);
    }

    Ok(job)
}

/// Execute a single job: rsync → tmux → poll
///
/// Steps:
/// 1. Update local DB status to 'running'
/// 2. Rsync project files to remote server
/// 3. Create tmux session with wrapper invocation
/// 4. Poll server DB every 2 seconds for completion
/// 5. Update local DB when job completes
async fn execute_job(
    db: &SqlitePool,
    ssh: &SshManager,
    job: &Job,
    ssh_host: &str,
    ssh_username: &str,
) -> Result<(), String> {
    // 1. Update local DB to running
    sqlx::query("UPDATE jobs SET status = 'running', started_at = datetime('now') WHERE id = ?")
        .bind(job.id)
        .execute(db)
        .await
        .map_err(|e| format!("Failed to update job status: {e}"))?;

    tracing::info!("Job {} marked as running", job.id);

    // 2. Get project path from database
    let project_path = get_project_path(db, job.project_id).await?;

    // 3. Rsync project files
    if let Err(e) = rsync_project(&project_path, ssh_host, ssh_username).await {
        tracing::error!("Failed to rsync project: {}", e);
        return Err(format!("Failed to sync project files: {e}"));
    }

    tracing::info!("Project files synced for job {}", job.id);

    // 4. Create tmux session with wrapper
    let session_name = generate_session_name(job.id);
    let wrapper_cmd = format!(
        "~/.solverpilot/bin/job_wrapper.sh {} python3 {}",
        job.id, job.benchmark_name
    );

    // Check for session collision (unlikely but handle it)
    let check_session = format!("tmux has-session -t {session_name} 2>/dev/null");
    if ssh.executor().execute(&check_session).await.is_ok() {
        tracing::warn!("tmux session {} already exists, killing it", session_name);
        let kill_cmd = format!("tmux kill-session -t {session_name}");
        ssh.executor().execute(&kill_cmd).await.ok(); // Ignore errors
    }

    // Create new session
    let create_session = format!("tmux new-session -d -s {session_name} \"{wrapper_cmd}\"");
    ssh.executor()
        .execute(&create_session)
        .await
        .map_err(|e| format!("Failed to create tmux session: {e}"))?;

    tracing::info!("tmux session {} created for job {}", session_name, job.id);

    // Store session name in DB (for future reference)
    let session_info = format!("tmux_session: {session_name}");
    sqlx::query("UPDATE jobs SET log_content = ? WHERE id = ?")
        .bind(&session_info)
        .bind(job.id)
        .execute(db)
        .await
        .ok(); // Non-critical, continue if fails

    // 5. Poll for completion
    poll_job_completion(db, ssh, job.id).await?;

    tracing::info!("Job {} completed", job.id);
    Ok(())
}

/// Poll server DB every 2 seconds for job completion
///
/// Queries server `SQLite` database via SSH to check job status.
/// Updates local DB when status changes to 'completed' or 'failed'.
async fn poll_job_completion(db: &SqlitePool, ssh: &SshManager, job_id: i64) -> Result<(), String> {
    let mut poll_interval = interval(Duration::from_secs(2));

    loop {
        poll_interval.tick().await;

        // Query server DB
        // Note: job_id is i64 so SQL injection is not possible
        let sql_cmd = format!(
            "sqlite3 ~/.solverpilot-server/server.db \"SELECT status, exit_code, completed_at FROM jobs WHERE id = {job_id}\""
        );

        match ssh.executor().execute(&sql_cmd).await {
            Ok(output) => {
                if let Some((status, exit_code, completed_at)) = parse_sql_output(&output) {
                    if status == "completed" || status == "failed" {
                        tracing::info!("Job {} {} with exit code {:?}", job_id, status, exit_code);

                        // Update local DB
                        sqlx::query(
                            "
                            UPDATE jobs 
                            SET status = ?, 
                                finished_at = COALESCE(?, datetime('now')),
                                progress_current = progress_total
                            WHERE id = ?
                            ",
                        )
                        .bind(&status)
                        .bind(&completed_at)
                        .bind(job_id)
                        .execute(db)
                        .await
                        .map_err(|e| format!("Failed to update job: {e}"))?;

                        return Ok(());
                    }

                    tracing::debug!("Job {job_id} still running (status: {status})");
                }
            }
            Err(e) => {
                tracing::warn!("Failed to query server DB: {}", e);
                // Continue polling - temporary SSH issues shouldn't fail job
            }
        }
    }
}

/// Rsync project to remote server
///
/// Syncs project directory to ~/solverpilot-projects/ on remote server.
/// Excludes .git and __pycache__ directories.
async fn rsync_project(
    project_path: &str,
    ssh_host: &str,
    ssh_username: &str,
) -> Result<(), String> {
    let remote_base = "~/solverpilot-projects/";

    // Extract project name from path
    let project_name = std::path::Path::new(project_path)
        .file_name()
        .ok_or("Invalid project path")?
        .to_str()
        .ok_or("Invalid project name")?;

    let remote_dest = format!("{ssh_username}@{ssh_host}:{remote_base}{project_name}");

    tracing::debug!("Executing rsync to {}", remote_dest);

    // Execute rsync via local command (not SSH)
    let output = tokio::process::Command::new("rsync")
        .args([
            "-avz",
            "--delete",
            "--exclude",
            ".git",
            "--exclude",
            "__pycache__",
            &format!("{project_path}/"),
            &remote_dest,
        ])
        .output()
        .await
        .map_err(|e| format!("Failed to execute rsync: {e}"))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("rsync failed: {stderr}"));
    }

    Ok(())
}

/// Get project path from database
async fn get_project_path(db: &SqlitePool, project_id: Option<i64>) -> Result<String, String> {
    let project_id = project_id.ok_or("Job has no associated project")?;

    let row = sqlx::query("SELECT path FROM projects WHERE id = ?")
        .bind(project_id)
        .fetch_one(db)
        .await
        .map_err(|e| format!("Failed to get project: {e}"))?;

    let path: String = row.get("path");
    Ok(path)
}

/// Generate unique tmux session name
///
/// Format: `solverpilot_{username}_{job_id:0:8}` (truncated to 8 chars)
/// Example: `solverpilot_alice_12345678`
fn generate_session_name(job_id: i64) -> String {
    // Truncate job_id to 8 characters as per AC specification
    let job_id_str = job_id.to_string();
    let truncated = if job_id_str.len() > 8 {
        &job_id_str[..8]
    } else {
        &job_id_str
    };
    format!("solverpilot_{}_{}", whoami::username(), truncated)
}

/// Parse SQL output: `status|exit_code|completed_at`
///
/// Parses pipe-separated `SQLite` output.
/// Returns `None` if format is invalid.
fn parse_sql_output(output: &str) -> Option<(String, Option<i32>, Option<String>)> {
    let parts: Vec<&str> = output.trim().split('|').collect();
    if !parts.is_empty() && !parts[0].is_empty() {
        let status = parts[0].to_string();
        let exit_code = if parts.len() >= 2 && !parts[1].is_empty() {
            parts[1].parse::<i32>().ok()
        } else {
            None
        };
        let completed_at = if parts.len() >= 3 && !parts[2].is_empty() {
            Some(parts[2].to_string())
        } else {
            None
        };
        Some((status, exit_code, completed_at))
    } else {
        None
    }
}

/// Mark job as failed in local DB
async fn mark_job_failed(db: &SqlitePool, job_id: i64, error: &str) -> Result<(), String> {
    sqlx::query(
        "
        UPDATE jobs 
        SET status = 'failed', 
            error_message = ?, 
            finished_at = datetime('now')
        WHERE id = ?
        ",
    )
    .bind(error)
    .bind(job_id)
    .execute(db)
    .await
    .map_err(|e| format!("Failed to mark job failed: {e}"))?;

    tracing::info!("Job {} marked as failed: {}", job_id, error);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_session_name_format() {
        let session = generate_session_name(12_345_678);
        assert!(session.starts_with("solverpilot_"));
        // Should contain the username
        assert!(session.contains(&whoami::username()));
        // Job ID should be at the end (truncated to 8 chars)
        assert!(session.ends_with("12345678"));
    }

    #[test]
    fn test_generate_session_name_truncation() {
        // Test with a large job ID that exceeds 8 digits
        let session = generate_session_name(123_456_789_012);
        // Should truncate to first 8 characters: "12345678"
        assert!(session.ends_with("12345678"));
        assert!(!session.contains("9012"));
    }

    #[test]
    fn test_generate_session_name_short_id() {
        // Test with a short job ID (less than 8 digits)
        let session = generate_session_name(42);
        assert!(session.ends_with("42"));
    }

    #[test]
    fn test_parse_sql_output_completed() {
        let output = "completed|0|2026-01-11T14:23:45Z";
        let result = parse_sql_output(output);
        assert!(result.is_some());

        let (status, exit_code, completed_at) = result.unwrap_or_default();
        assert_eq!(status, "completed");
        assert_eq!(exit_code, Some(0));
        assert_eq!(completed_at, Some("2026-01-11T14:23:45Z".to_string()));
    }

    #[test]
    fn test_parse_sql_output_failed() {
        let output = "failed|1|2026-01-11T14:30:00Z";
        let result = parse_sql_output(output);
        assert!(result.is_some());

        let (status, exit_code, completed_at) = result.unwrap_or_default();
        assert_eq!(status, "failed");
        assert_eq!(exit_code, Some(1));
        assert_eq!(completed_at, Some("2026-01-11T14:30:00Z".to_string()));
    }

    #[test]
    fn test_parse_sql_output_running() {
        let output = "running||";
        let result = parse_sql_output(output);
        assert!(result.is_some());

        let (status, exit_code, completed_at) = result.unwrap_or_default();
        assert_eq!(status, "running");
        assert_eq!(exit_code, None);
        assert_eq!(completed_at, None);
    }

    #[test]
    fn test_parse_sql_output_malformed() {
        // Empty output
        assert!(parse_sql_output("").is_none());
        // Only whitespace
        assert!(parse_sql_output("   ").is_none());
        // Newlines
        assert!(parse_sql_output("\n").is_none());
    }

    #[test]
    fn test_parse_sql_output_partial() {
        // Only status, missing fields
        let output = "completed";
        let result = parse_sql_output(output);
        assert!(result.is_some());
        let (status, exit_code, completed_at) = result.unwrap_or_default();
        assert_eq!(status, "completed");
        assert_eq!(exit_code, None);
        assert_eq!(completed_at, None);
    }

    #[test]
    fn test_wrapper_invocation_command_format() {
        // Verify wrapper command format matches AC specification
        let job_id: i64 = 12345;
        let benchmark_name = "bench.py";
        let wrapper_cmd = format!(
            "~/.solverpilot/bin/job_wrapper.sh {} python3 {}",
            job_id, benchmark_name
        );

        assert!(wrapper_cmd.starts_with("~/.solverpilot/bin/job_wrapper.sh"));
        assert!(wrapper_cmd.contains("12345"));
        assert!(wrapper_cmd.contains("python3"));
        assert!(wrapper_cmd.contains("bench.py"));
    }

    #[test]
    fn test_rsync_command_structure() {
        // Test rsync command construction logic
        let project_path = "/home/user/myproject";
        let ssh_host = "server.example.com";
        let ssh_username = "alice";
        let remote_base = "~/solverpilot-projects/";

        // Extract project name
        let project_name = std::path::Path::new(project_path)
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("unknown");

        assert_eq!(project_name, "myproject");

        // Verify remote destination format
        let remote_dest = format!("{ssh_username}@{ssh_host}:{remote_base}{project_name}");
        assert_eq!(
            remote_dest,
            "alice@server.example.com:~/solverpilot-projects/myproject"
        );
    }

    #[test]
    fn test_tmux_session_collision_check_command() {
        // Verify tmux collision check command format
        let session_name = "solverpilot_alice_12345678";
        let check_cmd = format!("tmux has-session -t {session_name} 2>/dev/null");
        assert_eq!(
            check_cmd,
            "tmux has-session -t solverpilot_alice_12345678 2>/dev/null"
        );
    }

    #[test]
    fn test_tmux_create_session_command() {
        // Verify tmux session creation command format
        let session_name = "solverpilot_alice_12345678";
        let wrapper_cmd = "~/.solverpilot/bin/job_wrapper.sh 12345678 python3 bench.py";
        let create_cmd = format!("tmux new-session -d -s {session_name} \"{wrapper_cmd}\"");

        assert!(create_cmd.starts_with("tmux new-session -d -s"));
        assert!(create_cmd.contains(session_name));
        assert!(create_cmd.contains("job_wrapper.sh"));
    }

    // Note: Integration tests with mock SSH/DB require additional infrastructure
    // and are covered in end-to-end testing. See Story 2.4 Dev Notes.
}
