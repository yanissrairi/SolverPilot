# Story 2.6: Job State Reconciliation - Triple Redundancy Validation

Status: ready-for-dev

<!-- Note: Validation is optional. Run validate-create-story for quality check before dev-story. -->

## Story

As a system architect,
I want to validate job state using triple redundancy (SQLite → State File → tmux check),
So that job status is accurate even if one source fails or is corrupted.

## Acceptance Criteria

**Given** a job completed on the remote server
**When** the backend queries job status
**Then** the reconciliation follows this priority chain:

1. **PRIMARY:** Query server SQLite DB via SSH
2. **FALLBACK:** Parse state file if SQLite unavailable
3. **INFERENCE:** Check tmux session existence if both fail
4. **ERROR:** Mark as "state lost" if all sources fail

**Given** the server SQLite database is healthy
**When** I query job status
**Then** the backend executes:

```bash
ssh user@host "sqlite3 ~/.solverpilot-server/server.db \
  'SELECT status, exit_code, completed_at FROM jobs WHERE id=<job_id>'"
```

**And** the returned status is trusted (primary source)
**And** state file and tmux checks are skipped (optimization)

**Given** the SQLite query fails (database locked, corrupted, or missing)
**When** the FALLBACK activates
**Then** the backend reads the state file via SSH:

```bash
ssh user@host "cat ~/.solverpilot-server/jobs/<job_id>.status"
```

**And** the JSON is parsed: `{"status": "completed", "exit_code": 0, ...}`
**And** the parsed status is used (fallback source)

**Given** both SQLite and state file are unavailable
**When** the INFERENCE activates
**Then** the backend checks tmux session existence:

```bash
ssh user@host "tmux has-session -t solverpilot_<user>_<job_id_short> 2>/dev/null && echo 'exists' || echo 'missing'"
```

**And** if session exists → infer status = "running"
**And** if session missing → infer status = "unknown" (completed or crashed)

**Given** all three sources fail (server unreachable, network timeout)
**When** the reconciliation reaches ERROR state
**Then** the backend returns error: "State lost - unable to determine job status"
**And** the UI shows clear indicator: "⚠ State unavailable - server unreachable"
**And** the job retains last known status in local DB (honest about stale data)

**Given** a job was marked "running" when I closed the app
**When** reconciliation queries the server and finds status = "completed"
**Then** the local DB is updated:

```sql
UPDATE jobs
SET status='completed',
    completed_at=<remote_completed_at>,
    exit_code=<remote_exit_code>
WHERE id=?
```

**Given** a job was "running" but the tmux session is missing and no exit_code
**When** reconciliation detects this conflict
**Then** the status is inferred as "failed" with error_message: "Job terminated unexpectedly (possible SIGKILL)"

**And** reconciliation logic is reused in Epic 3 (startup reconciliation)
**And** all reconciliation queries have 5-second timeout (avoid hanging on unresponsive server)
**And** reconciliation results are logged for debugging: `tracing::info!("Reconciliation for job {}: SQLite={}, StateFile={}, Tmux={}", ...)`

## Tasks / Subtasks

- [ ] Task 1: Create reconciliation.rs module with priority chain (AC: SQLite → File → tmux → Error)
  - [ ] Subtask 1.1: Create `src-tauri/src/reconciliation.rs` file
  - [ ] Subtask 1.2: Define `ReconciliationResult` enum (SqliteSource, StateFileSource, TmuxInference, StateLost)
  - [ ] Subtask 1.3: Implement `query_server_sqlite()` - PRIMARY source with 5s timeout
  - [ ] Subtask 1.4: Implement `parse_state_file()` - FALLBACK source
  - [ ] Subtask 1.5: Implement `check_tmux_session()` - INFERENCE source
  - [ ] Subtask 1.6: Implement priority chain logic - try each source in order
  - [ ] Subtask 1.7: Add `mod reconciliation;` to lib.rs

- [ ] Task 2: Implement SQLite query via SSH (AC: query server DB, 5s timeout, parse JSON)
  - [ ] Subtask 2.1: Build query: `SELECT status, exit_code, completed_at FROM jobs WHERE id=?`
  - [ ] Subtask 2.2: Execute via SSH: `sqlite3 -json ~/.solverpilot-server/server.db "<query>"`
  - [ ] Subtask 2.3: Add tokio::time::timeout (5 seconds) to prevent hanging
  - [ ] Subtask 2.4: Parse JSON output using serde_json
  - [ ] Subtask 2.5: Handle SQLite errors (database locked, corrupted, missing)
  - [ ] Subtask 2.6: Log SQLite query success/failure for debugging

- [ ] Task 3: Implement state file parsing (AC: read JSON file via SSH, parse status)
  - [ ] Subtask 3.1: Build SSH command: `cat ~/.solverpilot-server/jobs/<job_id>.status`
  - [ ] Subtask 3.2: Add 5-second timeout to SSH read
  - [ ] Subtask 3.3: Parse JSON: `{"status": "completed", "exit_code": 0, "completed_at": "..."}`
  - [ ] Subtask 3.4: Handle file not found (ENOENT)
  - [ ] Subtask 3.5: Handle malformed JSON (log error, continue to next source)
  - [ ] Subtask 3.6: Log state file read success/failure

- [ ] Task 4: Implement tmux session check (AC: check session existence, infer status)
  - [ ] Subtask 4.1: Generate session name: `solverpilot_{USER}_{job_id:0:8}`
  - [ ] Subtask 4.2: Execute SSH: `tmux has-session -t <name> 2>/dev/null && echo 'exists' || echo 'missing'`
  - [ ] Subtask 4.3: Add 5-second timeout
  - [ ] Subtask 4.4: Parse output: "exists" → Running, "missing" → Unknown
  - [ ] Subtask 4.5: Handle tmux not installed error
  - [ ] Subtask 4.6: Log tmux check result

- [ ] Task 5: Create reconcile_job_state() public API (AC: entry point, returns JobStatus)
  - [ ] Subtask 5.1: Define function signature: `pub async fn reconcile_job_state(job_id: &str, ssh: &SshManager) -> Result<JobStatus, String>`
  - [ ] Subtask 5.2: Execute priority chain: try SQLite → File → tmux → Error
  - [ ] Subtask 5.3: Log each source attempt with result
  - [ ] Subtask 5.4: Return JobStatus enum (Running, Completed, Failed, Unknown, StateLost)
  - [ ] Subtask 5.5: Add tracing for debugging: `tracing::info!("Reconciliation for job {}: source={}, status={}", ...)`

- [ ] Task 6: Add Tauri command for reconciliation (AC: reconcile_job_state command)
  - [ ] Subtask 6.1: Add `reconcile_job_state(job_id: String)` command in commands.rs
  - [ ] Subtask 6.2: Call reconciliation::reconcile_job_state with SSH manager
  - [ ] Subtask 6.3: Update local DB with reconciled status
  - [ ] Subtask 6.4: Register command in lib.rs invoke_handler
  - [ ] Subtask 6.5: Add `reconcileJobState(jobId: string)` to api.ts
  - [ ] Subtask 6.6: Add TypeScript types for ReconciliationResult

- [ ] Task 7: Write unit tests for priority chain (AC: test each fallback scenario)
  - [ ] Subtask 7.1: Test SQLite success (skip File and tmux checks)
  - [ ] Subtask 7.2: Test SQLite failure → State file success
  - [ ] Subtask 7.3: Test SQLite + File failure → tmux inference (exists → Running)
  - [ ] Subtask 7.4: Test all sources fail → StateLost error
  - [ ] Subtask 7.5: Test timeout handling (5-second limit)
  - [ ] Subtask 7.6: Mock SSH for predictable testing

- [ ] Task 8: Write integration test for reconciliation scenarios (AC: full priority chain flow)
  - [ ] Subtask 8.1: Test scenario: Job completed while app closed (SQLite shows completed)
  - [ ] Subtask 8.2: Test scenario: Wrapper crashed (no state file, no tmux → Failed)
  - [ ] Subtask 8.3: Test scenario: SQLite locked (fallback to state file)
  - [ ] Subtask 8.4: Test scenario: Server rebooted (no tmux → Unknown)
  - [ ] Subtask 8.5: Mock SSH responses for each scenario

- [ ] Task 9: Add logging and error context (AC: tracing logs, clear error messages)
  - [ ] Subtask 9.1: Add tracing::info for each reconciliation attempt
  - [ ] Subtask 9.2: Add tracing::warn for fallback activations
  - [ ] Subtask 9.3: Add tracing::error for state loss scenarios
  - [ ] Subtask 9.4: Include job_id, source, and status in all logs
  - [ ] Subtask 9.5: Use .map_err() to add context to errors: `format!("Failed to query SQLite for job {}: {}", job_id, e)`

## Dev Notes

### CRITICAL MISSION CONTEXT

**You are implementing the TRIPLE REDUNDANCY STATE VALIDATION ENGINE that ensures job status is ALWAYS accurate, even when sources fail!**

Story 2.4 created the queue execution engine. Story 2.5 added pause/resume controls. **Story 2.6 (THIS)** adds the reconciliation layer that validates job state using three independent sources with a clear priority chain.

**Impact Chain:**

- Story 2.1 ✅: Bash wrapper writes state to SQLite + JSON files
- Story 2.2 ✅: Server SQLite DB schema ready
- Story 2.3 ✅: Wrapper deployed to remote server
- Story 2.4 ✅: Queue execution engine running jobs
- Story 2.5 ✅: Pause/resume controls implemented
- **Story 2.6** (THIS): Triple redundancy reconciliation validates state
- Epic 3: Startup reconciliation uses this priority chain to sync on app restart
- Epic 4: Real-time polling uses reconciliation for status updates

**Critical Success Criteria:**

- MUST implement priority chain: SQLite → State File → tmux → Error
- MUST add 5-second timeout to all SSH queries (prevent hanging)
- MUST log each source attempt for debugging
- MUST return clear error when all sources fail (honest about state loss)
- MUST handle edge cases: crashed wrapper, rebooted server, locked DB
- MUST be reusable in Epic 3 for startup reconciliation

### Architecture Context

**Module Organization:**

This story creates a NEW isolated module for reconciliation logic:

```
src-tauri/src/
├── reconciliation.rs    # NEW: Triple redundancy priority chain (Story 2.6)
├── commands.rs          # EXTEND: Add reconcile_job_state command
├── lib.rs               # EXTEND: Register reconciliation module, register command
└── queue_service.rs     # FUTURE: Will use reconciliation in Epic 4 polling
```

**Reconciliation Priority Chain:**

```rust
// src-tauri/src/reconciliation.rs

use std::sync::Arc;
use tokio::time::{timeout, Duration};
use tracing::{info, warn, error};

/// Reconciliation result indicating which source was used
#[derive(Debug, Clone, PartialEq)]
pub enum ReconciliationSource {
    SqliteDatabase,    // PRIMARY: Server SQLite DB via SSH
    StateFile,         // FALLBACK: JSON state file via SSH
    TmuxInference,     // INFERENCE: tmux session existence check
}

/// Job status after reconciliation
#[derive(Debug, Clone, PartialEq)]
pub enum JobStatus {
    Running,
    Completed { exit_code: i32, completed_at: String },
    Failed { exit_code: i32, error_message: String },
    Unknown,    // Cannot determine (tmux missing, no exit code)
    StateLost,  // All sources failed
}

/// Reconciliation result with source information
#[derive(Debug, Clone)]
pub struct ReconciliationResult {
    pub status: JobStatus,
    pub source: ReconciliationSource,
}

/// Main reconciliation function - implements triple redundancy priority chain
pub async fn reconcile_job_state(
    job_id: &str,
    ssh: Arc<crate::ssh::SshManager>,
) -> Result<ReconciliationResult, String> {
    info!("Starting reconciliation for job: {}", job_id);

    // 1. PRIMARY: Try SQLite database
    match query_server_sqlite(job_id, ssh.clone()).await {
        Ok(status) => {
            info!("Reconciliation successful (SQLite): job={}, status={:?}", job_id, status);
            return Ok(ReconciliationResult {
                status,
                source: ReconciliationSource::SqliteDatabase,
            });
        }
        Err(e) => {
            warn!("SQLite query failed for job {}: {} - trying state file", job_id, e);
        }
    }

    // 2. FALLBACK: Try state file
    match parse_state_file(job_id, ssh.clone()).await {
        Ok(status) => {
            info!("Reconciliation successful (StateFile): job={}, status={:?}", job_id, status);
            return Ok(ReconciliationResult {
                status,
                source: ReconciliationSource::StateFile,
            });
        }
        Err(e) => {
            warn!("State file read failed for job {}: {} - trying tmux check", job_id, e);
        }
    }

    // 3. INFERENCE: Check tmux session existence
    match check_tmux_session(job_id, ssh.clone()).await {
        Ok(exists) => {
            let status = if exists {
                JobStatus::Running
            } else {
                JobStatus::Unknown
            };
            info!("Reconciliation via tmux inference: job={}, exists={}, status={:?}", job_id, exists, status);
            return Ok(ReconciliationResult {
                status,
                source: ReconciliationSource::TmuxInference,
            });
        }
        Err(e) => {
            error!("Tmux check failed for job {}: {} - state lost", job_id, e);
        }
    }

    // 4. ERROR: All sources failed
    error!("Reconciliation failed for job {}: all sources unavailable", job_id);
    Err(format!("State lost - unable to determine job status for {}", job_id))
}

/// Query server SQLite database (PRIMARY source)
async fn query_server_sqlite(
    job_id: &str,
    ssh: Arc<crate::ssh::SshManager>,
) -> Result<JobStatus, String> {
    let query = format!(
        "SELECT status, exit_code, completed_at FROM jobs WHERE id = '{}'",
        job_id.replace("'", "''")  // Escape single quotes for SQL safety
    );

    let cmd = format!(
        "sqlite3 -json ~/.solverpilot-server/server.db \"{}\"",
        query
    );

    // Execute with 5-second timeout
    let output = timeout(
        Duration::from_secs(5),
        ssh.execute(&cmd)
    )
    .await
    .map_err(|_| "SQLite query timeout (5 seconds exceeded)".to_string())?
    .map_err(|e| format!("SSH execution failed: {}", e))?;

    // Parse JSON output
    #[derive(serde::Deserialize)]
    struct DbRow {
        status: String,
        exit_code: Option<i32>,
        completed_at: Option<String>,
    }

    let rows: Vec<DbRow> = serde_json::from_str(&output)
        .map_err(|e| format!("Failed to parse SQLite JSON: {}", e))?;

    let row = rows.first()
        .ok_or_else(|| format!("Job {} not found in server database", job_id))?;

    // Convert to JobStatus
    match row.status.as_str() {
        "running" => Ok(JobStatus::Running),
        "completed" => Ok(JobStatus::Completed {
            exit_code: row.exit_code.unwrap_or(0),
            completed_at: row.completed_at.clone().unwrap_or_default(),
        }),
        "failed" => Ok(JobStatus::Failed {
            exit_code: row.exit_code.unwrap_or(1),
            error_message: "Job failed".to_string(),
        }),
        other => Err(format!("Unknown status in database: {}", other)),
    }
}

/// Parse state file (FALLBACK source)
async fn parse_state_file(
    job_id: &str,
    ssh: Arc<crate::ssh::SshManager>,
) -> Result<JobStatus, String> {
    let cmd = format!("cat ~/.solverpilot-server/jobs/{}.status", job_id);

    // Execute with 5-second timeout
    let output = timeout(
        Duration::from_secs(5),
        ssh.execute(&cmd)
    )
    .await
    .map_err(|_| "State file read timeout (5 seconds exceeded)".to_string())?
    .map_err(|e| format!("SSH execution failed: {}", e))?;

    // Parse JSON state file
    #[derive(serde::Deserialize)]
    struct StateFile {
        status: String,
        exit_code: Option<i32>,
        completed_at: Option<String>,
    }

    let state: StateFile = serde_json::from_str(&output)
        .map_err(|e| format!("Failed to parse state file JSON: {}", e))?;

    // Convert to JobStatus
    match state.status.as_str() {
        "running" => Ok(JobStatus::Running),
        "completed" => Ok(JobStatus::Completed {
            exit_code: state.exit_code.unwrap_or(0),
            completed_at: state.completed_at.unwrap_or_default(),
        }),
        "failed" => Ok(JobStatus::Failed {
            exit_code: state.exit_code.unwrap_or(1),
            error_message: "Job failed".to_string(),
        }),
        other => Err(format!("Unknown status in state file: {}", other)),
    }
}

/// Check tmux session existence (INFERENCE source)
async fn check_tmux_session(
    job_id: &str,
    ssh: Arc<crate::ssh::SshManager>,
) -> Result<bool, String> {
    // Generate session name (matches Story 2.4 convention)
    let short_id = &job_id[..8.min(job_id.len())];
    let session_name = format!("solverpilot_default_{}", short_id);

    let cmd = format!(
        "tmux has-session -t {} 2>/dev/null && echo 'exists' || echo 'missing'",
        session_name
    );

    // Execute with 5-second timeout
    let output = timeout(
        Duration::from_secs(5),
        ssh.execute(&cmd)
    )
    .await
    .map_err(|_| "Tmux check timeout (5 seconds exceeded)".to_string())?
    .map_err(|e| format!("SSH execution failed: {}", e))?;

    // Parse output
    Ok(output.trim() == "exists")
}
```

**Rationale:**

- ✅ **Priority Chain** - SQLite first (most reliable), state file second (backup), tmux last (inference)
- ✅ **5-Second Timeouts** - All SSH queries timeout to prevent hanging on unresponsive servers
- ✅ **Comprehensive Logging** - Every source attempt logged with result for debugging
- ✅ **Clear Error States** - StateLost indicates all sources failed (honest about state)
- ✅ **Reusable** - Epic 3 will use this same logic for startup reconciliation

### Previous Story Learnings

**From Story 2.4 (Queue Execution Backend):**

- Queue execution engine created with sequential job processing
- Jobs start via tmux with wrapper: `tmux new-session -d -s <name> "~/.solverpilot/bin/job_wrapper.sh <job_id> ..."`
- Session naming convention: `solverpilot_default_{job_id:0:8}`
- Server SQLite DB tracks job status (running, completed, failed)

**Integration Pattern:**

Story 2.6 ADDS reconciliation validation to complement Story 2.4's execution:

```
Story 2.4: Start job → Update server DB (status=running) → Poll for completion
Story 2.6: Reconcile job → Query SQLite → Fallback to state file → Infer from tmux
```

**Key Integration Points:**

- Story 2.4 creates tmux sessions - Story 2.6 checks their existence
- Story 2.4 updates server SQLite - Story 2.6 queries it as primary source
- Story 2.1 creates state files - Story 2.6 uses them as fallback

**From Story 2.5 (Pause/Resume Controls):**

- Queue can be paused mid-execution
- Running jobs complete naturally, new jobs don't start
- Queue state persists to metadata table

**Reconciliation Use Case:**

- After pausing queue and closing app, reconciliation validates which jobs completed while app was closed
- Epic 3 will use reconciliation to show "2 jobs completed, 1 failed while you were away"

### Latest Technical Specifics (Rust Async Error Handling)

**Timeout Implementation (tokio 1.x):**

```rust
use tokio::time::{timeout, Duration};

// ✅ CORRECT: Wrap async operation with timeout
let result = timeout(
    Duration::from_secs(5),
    ssh.execute(&cmd)
)
.await
.map_err(|_| "Operation timeout (5 seconds exceeded)".to_string())?  // Handle timeout error
.map_err(|e| format!("SSH execution failed: {}", e))?;               // Handle SSH error
```

**Exponential Backoff for Retries (Future - Epic 6):**

```rust
use tokio_retry::strategy::{ExponentialBackoff, jitter};
use tokio_retry::Retry;

// For Epic 6 auto-reconnect feature
let retry_strategy = ExponentialBackoff::from_millis(10)
    .map(jitter)  // Add randomness to delays
    .take(3);     // Max 3 retries

let result = Retry::spawn(retry_strategy, || {
    query_server_sqlite(job_id, ssh.clone())
})
.await?;
```

**Error Context with map_err():**

```rust
// ✅ CORRECT: Add context to errors for debugging
let output = ssh.execute(&cmd)
    .await
    .map_err(|e| format!("Failed to query SQLite for job {}: {}", job_id, e))?;

// ✅ CORRECT: Layer error context through call chain
serde_json::from_str(&output)
    .map_err(|e| format!("Failed to parse SQLite JSON for job {}: {}", job_id, e))?;
```

**tracing for Structured Logging:**

```rust
use tracing::{info, warn, error};

// ✅ CORRECT: Structured logging with context
info!("Starting reconciliation for job: {}", job_id);
warn!("SQLite query failed for job {}: {} - trying state file", job_id, e);
error!("Reconciliation failed for job {}: all sources unavailable", job_id);
```

**Sources:**

- [Handling Timeouts in Rust with async and tokio Timers - Sling Academy](https://www.slingacademy.com/article/handling-timeouts-in-rust-with-async-and-tokio-timers/)
- [Error Handling in Async Rust: From a Simple Function to a Real Health Monitor](https://drunkleen.com/posts/rust-async-error-handling)
- [Error Types and Propagation - Mastering Asynchronous Programming with Tokio in Rust](https://app.studyraid.com/en/read/10838/332174/error-types-and-propagation)
- [Network Timeouts and Cancellation - Mastering Asynchronous Programming with Tokio in Rust](https://app.studyraid.com/en/read/10838/332165/network-timeouts-and-cancellation)

### Architecture Compliance

**Module Isolation (CRITICAL):**

- ✅ CREATE `reconciliation.rs` (NEW isolated module for Story 2.6)
- ✅ EXTEND `commands.rs` (add reconcile_job_state command)
- ✅ EXTEND `lib.rs` (register reconciliation module, register command)
- ❌ DO NOT modify Alpha modules (db.rs, job.rs, ssh/)

**Reconciliation Priority Chain (CRITICAL):**

```
Priority Chain (MUST follow this order):
1. SQLite Database (PRIMARY) - Most reliable, wrapper writes here first
2. State File (FALLBACK) - Backup, wrapper writes here second
3. tmux Check (INFERENCE) - Last resort, session existence only
4. Error State (STATE LOST) - Honest about failure when all sources fail
```

**API Contract:**

```rust
// New Tauri command for Story 2.6
#[tauri::command]
pub async fn reconcile_job_state(
    job_id: String,
    state: State<'_, AppState>
) -> Result<ReconciliationResult, String> {
    let ssh = state.ssh.lock().await
        .as_ref()
        .ok_or("SSH manager not initialized")?
        .clone();

    reconciliation::reconcile_job_state(&job_id, ssh).await
}
```

```typescript
// New api.ts wrapper (frontend)
export interface ReconciliationResult {
  status: 'running' | 'completed' | 'failed' | 'unknown' | 'state_lost';
  source: 'sqlite' | 'state_file' | 'tmux' | 'error';
  exitCode?: number;
  completedAt?: string;
  errorMessage?: string;
}

export async function reconcileJobState(jobId: string): Promise<ReconciliationResult> {
  return invoke<ReconciliationResult>('reconcile_job_state', { jobId });
}
```

### Testing Requirements

**Unit Tests (in reconciliation.rs):**

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_reconcile_sqlite_success() -> Result<(), Box<dyn std::error::Error>> {
        let mock_ssh = MockSshManager::new();
        mock_ssh.add_response(
            "sqlite3 -json ~/.solverpilot-server/server.db",
            r#"[{"status":"completed","exit_code":0,"completed_at":"2026-01-13T10:30:00Z"}]"#
        );

        let result = reconcile_job_state("test-job-id", Arc::new(mock_ssh)).await?;

        assert_eq!(result.source, ReconciliationSource::SqliteDatabase);
        assert!(matches!(result.status, JobStatus::Completed { exit_code: 0, .. }));

        Ok(())
    }

    #[tokio::test]
    async fn test_reconcile_fallback_to_state_file() -> Result<(), Box<dyn std::error::Error>> {
        let mock_ssh = MockSshManager::new();

        // SQLite fails
        mock_ssh.add_error("sqlite3", "database locked");

        // State file succeeds
        mock_ssh.add_response(
            "cat ~/.solverpilot-server/jobs/test-job-id.status",
            r#"{"status":"completed","exit_code":0,"completed_at":"2026-01-13T10:30:00Z"}"#
        );

        let result = reconcile_job_state("test-job-id", Arc::new(mock_ssh)).await?;

        assert_eq!(result.source, ReconciliationSource::StateFile);
        assert!(matches!(result.status, JobStatus::Completed { exit_code: 0, .. }));

        Ok(())
    }

    #[tokio::test]
    async fn test_reconcile_tmux_inference() -> Result<(), Box<dyn std::error::Error>> {
        let mock_ssh = MockSshManager::new();

        // SQLite fails
        mock_ssh.add_error("sqlite3", "database not found");

        // State file fails
        mock_ssh.add_error("cat", "file not found");

        // tmux exists
        mock_ssh.add_response("tmux has-session", "exists");

        let result = reconcile_job_state("test-job-id", Arc::new(mock_ssh)).await?;

        assert_eq!(result.source, ReconciliationSource::TmuxInference);
        assert_eq!(result.status, JobStatus::Running);

        Ok(())
    }

    #[tokio::test]
    async fn test_reconcile_all_sources_fail() {
        let mock_ssh = MockSshManager::new();

        // All sources fail
        mock_ssh.add_error("sqlite3", "database locked");
        mock_ssh.add_error("cat", "file not found");
        mock_ssh.add_error("tmux", "server unreachable");

        let result = reconcile_job_state("test-job-id", Arc::new(mock_ssh)).await;

        assert!(result.is_err());
        assert!(result.unwrap_err().contains("State lost"));
    }

    #[tokio::test]
    async fn test_reconcile_timeout_handling() -> Result<(), Box<dyn std::error::Error>> {
        let mock_ssh = MockSshManager::new();

        // Simulate slow SSH (7 seconds - exceeds 5s timeout)
        mock_ssh.add_delayed_response("sqlite3", 7000, "...");

        let start = std::time::Instant::now();
        let result = reconcile_job_state("test-job-id", Arc::new(mock_ssh)).await;
        let elapsed = start.elapsed();

        // Should timeout after ~5 seconds, not wait full 7 seconds
        assert!(elapsed.as_secs() <= 6);
        assert!(result.is_err());

        Ok(())
    }
}
```

**Integration Tests (manual validation):**

```bash
# After implementing, test manually:

# 1. Start a job, close app, let job complete
# Verify: reconcile_job_state() returns Completed (from SQLite)

# 2. Manually lock server DB with another process
# Verify: reconcile_job_state() falls back to state file

# 3. Delete state file while job running
# Verify: reconcile_job_state() falls back to tmux check → Running

# 4. Disconnect from server (network down)
# Verify: reconcile_job_state() returns "State lost" error

# 5. Kill tmux session mid-job (simulated crash)
# Verify: reconcile_job_state() returns Failed (tmux missing, no exit code)
```

### Project Structure Notes

**Files to Create:**

```
src-tauri/src/reconciliation.rs    # NEW (Story 2.6) - Triple redundancy priority chain
```

**Files to Modify:**

```
src-tauri/src/commands.rs           # EXTEND: Add reconcile_job_state command
src-tauri/src/lib.rs                # EXTEND: Add mod reconciliation; register command
src/lib/api.ts                      # EXTEND: Add reconcileJobState() wrapper
src/lib/types.ts                    # EXTEND: Add ReconciliationResult interface
```

**Files NOT to Modify:**

```
src-tauri/src/job.rs                # EXISTS - Alpha (DO NOT MODIFY)
src-tauri/src/db.rs                 # EXISTS - Alpha (DO NOT MODIFY)
src-tauri/src/ssh/executor.rs       # EXISTS - Alpha (DO NOT MODIFY)
src-tauri/src/queue_service.rs      # EXISTS - Story 2.4/2.5 (Epic 4 will extend for polling)
```

### References

**Epic 2 Overview:**
[Source: _bmad-output/planning-artifacts/epics.md#Epic 2, lines 1510-2027]

- User Outcome: Queue execution with reliability and state validation
- FRs Covered: FR161-FR163 (crash recovery, detect partially-completed queues, recovery status indicators)

**Story 2.6 Requirements:**
[Source: _bmad-output/planning-artifacts/epics.md#Story 2.6, lines 1901-2003]

- Triple redundancy priority chain (SQLite → State File → tmux → Error)
- 5-second timeout on all SSH queries
- Comprehensive logging for debugging
- Reusable for Epic 3 startup reconciliation

**Architecture - Reconciliation Protocol:**
[Source: _bmad-output/planning-artifacts/architecture.md#Decision 2, lines 974-1078]

- Hybrid reconciliation strategy (SQLite intent, tmux reality)
- Clear reconciliation rules for conflicts
- Startup reconciliation and reconnect patterns
- Race condition prevention during sync window

**Architecture - Reconciliation Module:**
[Source: _bmad-output/planning-artifacts/architecture.md, lines 646-647]

- Backend module: `src-tauri/src/reconciliation.rs`
- Quality focus: Unit tests mandatory for reconciliation scenarios

**Previous Story (Story 2.5):**
[Source: _bmad-output/implementation-artifacts/2-5-start-pause-resume-queue-controls-frontend-backend.md]

- Pause/resume controls implemented
- Queue state persists to metadata table
- Graceful pause (running jobs complete, new jobs don't start)

**Previous Story (Story 2.4):**
[Source: _bmad-output/implementation-artifacts/2-4-queue-execution-backend-sequential-job-processing.md]

- Queue execution engine ready
- Sequential job processing (FIFO)
- Tmux session creation with wrapper
- Server SQLite DB updated by wrapper

**Previous Story (Story 2.1):**
[Source: _bmad-output/implementation-artifacts/2-1-bash-wrapper-script-state-capture-foundation.md]

- Bash wrapper script with trap EXIT
- Writes to server SQLite DB (primary)
- Writes to JSON state files (backup)
- Atomic status updates on job completion

**Project Context:**
[Source: _bmad-output/project-context.md]

- Rust error handling: Result<T, String>, never unwrap/expect
- Module isolation: CREATE new reconciliation.rs, EXTEND commands.rs/lib.rs
- Reconciliation priority: SQLite → State File → tmux → Error (MUST follow)
- Quality checks: cargo clippy (zero warnings), cargo test (all pass)

**Latest Rust Async Patterns:**
[Source: Web research - Tokio async patterns, timeout handling, error propagation]

- Use tokio::time::timeout() for 5-second limits on SSH queries
- Use .map_err() to add context to errors for debugging
- Use tracing::info/warn/error for structured logging
- Handle both timeout errors and underlying operation errors

### FRs Fulfilled

**From Epic 2 Requirements:**

This story fulfills the **state validation and crash recovery** requirements:

- FR161: Crash recovery (detect crashed jobs via reconciliation)
- FR162: Detect partially-completed queues (validate queue state on reconnect)
- FR163: Recovery status indicators (show accurate status after disconnect)

**Story Dependency Chain:**

- Story 2.1 ✅: Wrapper writes to SQLite + state files (provides data sources)
- Story 2.2 ✅: Server DB schema ready (provides primary source)
- Story 2.3 ✅: Wrapper deployed to remote (state files available)
- Story 2.4 ✅: Queue execution creates tmux sessions (provides inference source)
- Story 2.5 ✅: Pause/resume persists queue state
- **Story 2.6**: Triple redundancy reconciliation (THIS STORY)
- Epic 3: Startup reconciliation extends this priority chain
- Epic 4: Real-time polling uses reconciliation for status updates

## Dev Agent Record

### Agent Model Used

Claude Sonnet 4.5 (claude-sonnet-4-5-20250929)

### Debug Log References

### Completion Notes List

### File List
