# Story 2.4: Queue Execution Backend - Sequential Job Processing

Status: ready-for-dev

<!-- Note: Validation is optional. Run validate-create-story for quality check before dev-story. -->

## Story

As a researcher,
I want the backend to process queued jobs sequentially (one at a time),
So that jobs execute in order without overloading the remote server.

## Acceptance Criteria

**Given** I have 5 jobs in the queue with status = 'pending'
**When** the queue execution starts
**Then** the backend selects the job with the lowest queue_position (job #1)
**And** only ONE job is executed at a time (max_concurrent = 1 for Beta 1)

**Given** job #1 is selected for execution
**When** the execution process begins
**Then** the following steps occur in sequence:

1. Update local database: `UPDATE jobs SET status='running', started_at=datetime('now') WHERE id=?`
2. Rsync project files to remote server: `rsync -avz --delete <local_project_path> user@host:<remote_base_dir>/`
3. Create tmux session name: `solverpilot_${USER}_${JOB_ID:0:8}` (unique, collision-resistant)
4. Start job in tmux with wrapper:

```bash
tmux new-session -d -s <session_name> \
  "~/.solverpilot/bin/job_wrapper.sh <job_id> python3 <benchmark_path>"
```

5. Return immediately (non-blocking - job continues on server)

**Given** job #1 is now running on the remote server
**When** I query the local database
**Then** the job status is 'running'
**And** the tmux_session_name is stored in the local database

**Given** job #1 completes (wrapper writes to server DB)
**When** the backend polls for job status (2-second interval - Epic 4)
**Then** the backend detects completion by querying server DB via SSH:

- `ssh user@host "sqlite3 ~/.solverpilot-server/server.db 'SELECT status, exit_code FROM jobs WHERE id=<job_id>'"`

**Given** the backend detects job #1 is completed
**When** the status update occurs
**Then** the local database is updated:

- `UPDATE jobs SET status='completed', completed_at=<remote_completed_at>, exit_code=<exit_code> WHERE id=?`
  **And** the backend immediately selects the next pending job (job #2) and starts execution

**Given** I have jobs #1, #2, #3 pending
**When** job #1 completes
**Then** job #2 starts automatically (no user intervention required)
**And** when job #2 completes, job #3 starts automatically (continuous processing)

**Given** the queue is empty (no pending jobs)
**When** the backend checks for next job
**Then** the execution loop stops gracefully
**And** a toast notification shows: "Queue completed - all jobs finished"

**Given** the rsync operation fails (network error, permission denied)
**When** the failure is detected
**Then** the job status is set to 'failed'
**And** error_message is populated: "Failed to sync project files: <error_details>"
**And** the queue continues to the next job (failed jobs don't block queue - Epic 5)

**And** all SSH operations use bb8 connection pool for performance
**And** tmux session creation includes error handling (check if session name conflicts)
**And** all database operations return Result<T, String> (no unwrap/expect - clippy enforced)

## Tasks / Subtasks

- [ ] Task 1: Create queue_service.rs module with QueueManager (AC: sequential execution, max_concurrent=1)
  - [ ] Subtask 1.1: Create `src-tauri/src/queue_service.rs` file
  - [ ] Subtask 1.2: Define `QueueManager` struct with state (is_processing, current_job_id)
  - [ ] Subtask 1.3: Implement `start_processing()` method - spawn background Tokio task
  - [ ] Subtask 1.4: Implement `stop_processing()` method - graceful shutdown
  - [ ] Subtask 1.5: Add queue state to AppState in state.rs
  - [ ] Subtask 1.6: Add `mod queue_service;` to lib.rs

- [ ] Task 2: Implement job selection logic (AC: FIFO, pending jobs only)
  - [ ] Subtask 2.1: Query local DB: `SELECT * FROM jobs WHERE status='pending' ORDER BY queue_position ASC LIMIT 1`
  - [ ] Subtask 2.2: Return `Option<Job>` - None if queue empty
  - [ ] Subtask 2.3: Handle database errors gracefully
  - [ ] Subtask 2.4: Log job selection for debugging

- [ ] Task 3: Implement rsync project sync (AC: transfer files, exclude patterns)
  - [ ] Subtask 3.1: Build rsync command: `rsync -avz --delete --exclude '.git' --exclude '__pycache__' <local> <remote>`
  - [ ] Subtask 3.2: Execute rsync via SSH executor (reuse connection)
  - [ ] Subtask 3.3: Capture rsync output for error messages
  - [ ] Subtask 3.4: Handle permission denied, network errors
  - [ ] Subtask 3.5: Update job status to 'failed' on rsync failure

- [ ] Task 4: Implement tmux session creation with wrapper invocation (AC: unique session names, wrapper call)
  - [ ] Subtask 4.1: Generate session name: `solverpilot_{USER}_{job_id:0:8}`
  - [ ] Subtask 4.2: Check session collision: `tmux has-session -t <name> 2>/dev/null`
  - [ ] Subtask 4.3: Build wrapper command: `~/.solverpilot/bin/job_wrapper.sh <job_id> python3 <benchmark_path>`
  - [ ] Subtask 4.4: Create tmux session: `tmux new-session -d -s <name> "<wrapper_cmd>"`
  - [ ] Subtask 4.5: Store tmux_session_name in local DB
  - [ ] Subtask 4.6: Handle tmux creation errors

- [ ] Task 5: Implement polling loop for job status (AC: 2-second interval, query server DB)
  - [ ] Subtask 5.1: Create tokio::time::interval(Duration::from_secs(2))
  - [ ] Subtask 5.2: Query server DB: `SELECT status, exit_code, completed_at FROM jobs WHERE id=?`
  - [ ] Subtask 5.3: Parse SQL output (status|exit_code|completed_at)
  - [ ] Subtask 5.4: Update local DB when status changes
  - [ ] Subtask 5.5: Stop polling when job completes
  - [ ] Subtask 5.6: Handle SSH query failures gracefully

- [ ] Task 6: Implement continuous queue processing (AC: auto-start next job)
  - [ ] Subtask 6.1: After job completes, call `select_next_job()` immediately
  - [ ] Subtask 6.2: If next job exists, start execution
  - [ ] Subtask 6.3: If no jobs, stop processing loop
  - [ ] Subtask 6.4: Emit toast notification: "Queue completed - all jobs finished"

- [ ] Task 7: Add Tauri commands for queue control (AC: start_queue_processing command)
  - [ ] Subtask 7.1: Add `start_queue_processing()` command in commands.rs
  - [ ] Subtask 7.2: Add `get_queue_status()` command (returns processing state)
  - [ ] Subtask 7.3: Register commands in lib.rs invoke_handler
  - [ ] Subtask 7.4: Add `startQueueProcessing()` and `getQueueStatus()` to api.ts

- [ ] Task 8: Write unit tests for queue_service.rs (AC: FIFO, sequential execution)
  - [ ] Subtask 8.1: Test job selection returns lowest queue_position
  - [ ] Subtask 8.2: Test rsync command generation
  - [ ] Subtask 8.3: Test tmux session name uniqueness
  - [ ] Subtask 8.4: Test wrapper invocation command format
  - [ ] Subtask 8.5: Mock SSH and DB for isolated testing

- [ ] Task 9: Integration test with mock SSH (AC: full queue execution flow)
  - [ ] Subtask 9.1: Create 3 mock jobs in test DB
  - [ ] Subtask 9.2: Start queue processing
  - [ ] Subtask 9.3: Simulate job completion (update server DB mock)
  - [ ] Subtask 9.4: Verify all jobs processed in order
  - [ ] Subtask 9.5: Verify local DB updated correctly

## Dev Notes

### CRITICAL MISSION CONTEXT

**You are implementing the CORE QUEUE EXECUTION ENGINE that orchestrates the entire job processing pipeline!**

This story brings together ALL previous work in Epic 2:

- Story 2.1 ✅: Wrapper script ready to capture state
- Story 2.2 ✅: Server DB schema ready to store job state
- Story 2.3 ✅: Wrapper deployed to remote server
- **Story 2.4** (THIS): Queue execution loop that uses wrapper to run jobs sequentially

**Impact Chain:**

- Without this story, jobs sit in queue forever (no execution)
- This story enables: auto-sync → tmux launch → wrapper invocation → continuous processing
- Story 2.5 builds on this: Adds Start/Pause/Resume controls
- Story 2.6 depends on this: Reconciliation validates state written by this execution flow
- Epic 4 extends this: Real-time progress polling during execution

**Critical Success Criteria:**

- MUST execute jobs sequentially (max_concurrent = 1)
- MUST use wrapper script for state capture
- MUST poll server DB for completion detection
- MUST auto-start next job after current completes
- MUST handle failures gracefully (failed jobs don't block queue)

### Architecture Context

**Module Organization (from architecture.md):**

```
src-tauri/src/
├── queue_service.rs     # NEW FILE (Story 2.4) - Queue execution engine
├── wrapper.rs           # EXISTS (Story 2.3) - For wrapper invocation
├── server_db.rs         # EXISTS (Story 2.2) - Server DB schema
├── lib.rs               # EXTEND: add mod queue_service; and commands
├── commands.rs          # EXTEND: add start_queue_processing, get_queue_status
├── state.rs             # EXTEND: add Arc<Mutex<QueueManager>>
└── ssh/
    └── executor.rs      # USE: SshExecutor for rsync, tmux, SQL queries
```

**QueueManager Pattern (from architecture.md lines 2945-3120):**

```rust
// src-tauri/src/queue_service.rs

use std::sync::Arc;
use tokio::sync::Mutex;
use tokio::time::{interval, Duration};
use crate::ssh::SshManager;
use sqlx::SqlitePool;

pub struct QueueManager {
    is_processing: Arc<Mutex<bool>>,
    current_job_id: Arc<Mutex<Option<String>>>,
}

impl QueueManager {
    pub fn new() -> Self {
        Self {
            is_processing: Arc::new(Mutex::new(false)),
            current_job_id: Arc::new(Mutex::new(None)),
        }
    }

    /// Start queue processing loop in background task
    pub async fn start_processing(
        &self,
        db: SqlitePool,
        ssh: Arc<SshManager>,
    ) -> Result<(), String> {
        let mut processing = self.is_processing.lock().await;
        if *processing {
            return Err("Queue already processing".to_string());
        }
        *processing = true;
        drop(processing);

        // Spawn background task
        let is_processing = Arc::clone(&self.is_processing);
        let current_job_id = Arc::clone(&self.current_job_id);

        tokio::spawn(async move {
            loop {
                // Check if still processing
                if !*is_processing.lock().await {
                    break;
                }

                // Select next job
                match select_next_job(&db).await {
                    Ok(Some(job)) => {
                        *current_job_id.lock().await = Some(job.id.clone());

                        // Execute job
                        if let Err(e) = execute_job(&db, &ssh, &job).await {
                            log::error!("Job {} failed: {}", job.id, e);
                            mark_job_failed(&db, &job.id, &e).await.ok();
                        }

                        *current_job_id.lock().await = None;
                    }
                    Ok(None) => {
                        // Queue empty, stop processing
                        *is_processing.lock().await = false;
                        log::info!("Queue completed - all jobs finished");
                        break;
                    }
                    Err(e) => {
                        log::error!("Failed to select next job: {}", e);
                        tokio::time::sleep(Duration::from_secs(5)).await;
                    }
                }
            }
        });

        Ok(())
    }

    /// Stop queue processing gracefully
    pub async fn stop_processing(&self) -> Result<(), String> {
        let mut processing = self.is_processing.lock().await;
        *processing = false;
        Ok(())
    }

    /// Get current processing state
    pub async fn is_processing(&self) -> bool {
        *self.is_processing.lock().await
    }

    /// Get currently executing job ID
    pub async fn current_job(&self) -> Option<String> {
        self.current_job_id.lock().await.clone()
    }
}

/// Select next pending job by queue_position
async fn select_next_job(db: &SqlitePool) -> Result<Option<Job>, String> {
    sqlx::query_as!(
        Job,
        "SELECT * FROM jobs WHERE status = 'pending' ORDER BY queue_position ASC LIMIT 1"
    )
    .fetch_optional(db)
    .await
    .map_err(|e| format!("Failed to select next job: {}", e))
}

/// Execute a single job: rsync → tmux → poll
async fn execute_job(
    db: &SqlitePool,
    ssh: &SshManager,
    job: &Job,
) -> Result<(), String> {
    // 1. Update local DB to running
    sqlx::query!(
        "UPDATE jobs SET status = 'running', started_at = datetime('now') WHERE id = ?",
        job.id
    )
    .execute(db)
    .await
    .map_err(|e| format!("Failed to update job status: {}", e))?;

    // 2. Rsync project files
    rsync_project(ssh, &job.project_path).await?;

    // 3. Create tmux session with wrapper
    let session_name = format!("solverpilot_{}_{}", whoami::username(), &job.id[..8]);
    let wrapper_cmd = format!(
        "~/.solverpilot/bin/job_wrapper.sh {} python3 {}",
        job.id, job.benchmark_path
    );

    ssh.executor()
        .execute(&format!(
            "tmux new-session -d -s {} \"{}\"",
            session_name, wrapper_cmd
        ))
        .await
        .map_err(|e| format!("Failed to create tmux session: {}", e))?;

    // Store session name in DB
    sqlx::query!(
        "UPDATE jobs SET tmux_session_name = ? WHERE id = ?",
        session_name,
        job.id
    )
    .execute(db)
    .await
    .map_err(|e| format!("Failed to store session name: {}", e))?;

    // 4. Poll for completion
    poll_job_completion(db, ssh, &job.id).await?;

    Ok(())
}

/// Poll server DB every 2 seconds for job completion
async fn poll_job_completion(
    db: &SqlitePool,
    ssh: &SshManager,
    job_id: &str,
) -> Result<(), String> {
    let mut interval = interval(Duration::from_secs(2));

    loop {
        interval.tick().await;

        // Query server DB
        let query = format!(
            "SELECT status, exit_code, completed_at FROM jobs WHERE id = '{}'",
            job_id
        );
        let sql_cmd = format!(
            "sqlite3 ~/.solverpilot-server/server.db \"{}\"",
            query
        );

        match ssh.executor().execute(&sql_cmd).await {
            Ok(output) => {
                if let Some((status, exit_code, completed_at)) = parse_sql_output(&output) {
                    if status == "completed" || status == "failed" {
                        // Update local DB
                        sqlx::query!(
                            "UPDATE jobs SET status = ?, exit_code = ?, completed_at = ? WHERE id = ?",
                            status,
                            exit_code,
                            completed_at,
                            job_id
                        )
                        .execute(db)
                        .await
                        .map_err(|e| format!("Failed to update job: {}", e))?;

                        return Ok(());
                    }
                }
            }
            Err(e) => {
                log::warn!("Failed to query server DB: {}", e);
                // Continue polling - temporary SSH issues shouldn't fail job
            }
        }
    }
}

/// Rsync project to remote server
async fn rsync_project(ssh: &SshManager, project_path: &str) -> Result<(), String> {
    let remote_base = "~/solverpilot-projects";
    let rsync_cmd = format!(
        "rsync -avz --delete --exclude '.git' --exclude '__pycache__' {} {}:{}",
        project_path,
        ssh.config().host,
        remote_base
    );

    ssh.executor()
        .execute(&rsync_cmd)
        .await
        .map_err(|e| format!("Failed to sync project: {}", e))
}

/// Parse SQL output: status|exit_code|completed_at
fn parse_sql_output(output: &str) -> Option<(String, Option<i32>, Option<String>)> {
    let parts: Vec<&str> = output.trim().split('|').collect();
    if parts.len() == 3 {
        let status = parts[0].to_string();
        let exit_code = parts[1].parse::<i32>().ok();
        let completed_at = if parts[2].is_empty() {
            None
        } else {
            Some(parts[2].to_string())
        };
        Some((status, exit_code, completed_at))
    } else {
        None
    }
}

/// Mark job as failed in local DB
async fn mark_job_failed(
    db: &SqlitePool,
    job_id: &str,
    error: &str,
) -> Result<(), String> {
    sqlx::query!(
        "UPDATE jobs SET status = 'failed', error_message = ?, completed_at = datetime('now') WHERE id = ?",
        error,
        job_id
    )
    .execute(db)
    .await
    .map_err(|e| format!("Failed to mark job failed: {}", e))?;

    Ok(())
}
```

**Rationale:**

- ✅ **Sequential Execution** - Only one job at a time (max_concurrent = 1)
- ✅ **Background Task** - tokio::spawn for non-blocking execution
- ✅ **Polling Loop** - 2-second interval for status updates
- ✅ **Graceful Failure** - Failed jobs don't block queue
- ✅ **State Tracking** - is_processing and current_job_id for UI

### Previous Story Learnings

**From Story 2.1 (Wrapper Script):**

- Wrapper is deployed to `~/.solverpilot/bin/job_wrapper.sh`
- Invocation format: `~/.solverpilot/bin/job_wrapper.sh <job_id> <command> [args...]`
- Wrapper writes to both SQLite and JSON state file
- trap EXIT guarantees cleanup on SIGTERM/SIGINT (but not SIGKILL)

**From Story 2.2 (Server DB Schema):**

- Server DB at `~/.solverpilot-server/server.db`
- Jobs table has columns: id, status, exit_code, completed_at, started_at
- Status values: 'queued', 'running', 'completed', 'failed', 'killed'
- Query format: `sqlite3 ~/.solverpilot-server/server.db "SELECT ..."`

**From Story 2.3 (Wrapper Deployment):**

- Wrapper deployed via `deploy_wrapper()` command
- Idempotent deployment (safe to call multiple times)
- Version tracked in metadata table
- Deployment includes server DB initialization

**Integration Pattern:**

The queue execution flow integrates all three previous stories:

```
Queue Manager (Story 2.4)
    ↓
1. Check wrapper installed (Story 2.3)
2. Rsync project files
3. Create tmux session
    ↓
4. Invoke wrapper (Story 2.1)
    ↓
5. Wrapper updates server DB (Story 2.2)
    ↓
6. Poll server DB for completion (Story 2.4)
    ↓
7. Update local DB and start next job
```

### Git Intelligence (Recent Work Patterns)

**Last 5 commits show Epic 2 progression:**

```
1cb4828 fix(queue): code review fixes for Story 2.2
4b11b74 feat(queue): server-side SQLite schema initialization (Story 2.2)
3affa6b fix(queue): code review fixes for Story 2.1
11233a3 feat(queue): implement bash wrapper script - state capture foundation (Story 2.1)
9728f29 docs(retro): add Epic 1 retrospective
```

**Commit Pattern for Story 2.4:**

```
feat(queue): queue execution backend - sequential job processing (Story 2.4)

- Create queue_service.rs with QueueManager
- Implement start_processing() with background Tokio task
- Add job selection logic (FIFO by queue_position)
- Implement rsync project sync before execution
- Create tmux sessions with wrapper invocation
- Poll server DB every 2 seconds for completion
- Auto-start next job after current completes
- Add start_queue_processing and get_queue_status Tauri commands

Co-Authored-By: Claude Sonnet 4.5 <noreply@anthropic.com>
```

### Technical Requirements

**Tokio Async Patterns:**

```rust
// ✅ CORRECT: Background task with shared state
tokio::spawn(async move {
    let is_processing = Arc::clone(&self.is_processing);
    loop {
        if !*is_processing.lock().await {
            break;
        }
        // ... process jobs
    }
});

// ❌ WRONG: Blocking operations in async context
tokio::spawn(async move {
    std::thread::sleep(Duration::from_secs(2)); // BLOCKS!
});
```

**Polling Best Practices:**

- Use `tokio::time::interval(Duration::from_secs(2))` for consistent timing
- Always `.tick().await` before first iteration (skips immediate tick)
- Handle SSH failures gracefully (log warning, continue polling)
- Don't use `tokio::time::sleep` in tight loops (less accurate)

**SQLx Query Patterns:**

```rust
// ✅ CORRECT: Compile-time validated query
sqlx::query_as!(
    Job,
    "SELECT * FROM jobs WHERE status = ? ORDER BY queue_position ASC LIMIT 1",
    "pending"
)
.fetch_optional(&db)
.await?;

// ✅ CORRECT: Dynamic query when needed
sqlx::query(&format!("SELECT * FROM jobs WHERE id = '{}'", job_id))
    .fetch_one(&db)
    .await?;
```

**SSH Executor Reuse:**

The codebase uses bb8 connection pooling for 10x performance improvement:

```rust
// ✅ CORRECT: Reuse existing connection
let ssh = state.ssh_manager.lock().await
    .as_ref()
    .ok_or("SSH not connected")?
    .clone();

ssh.executor().execute("command").await?;

// ❌ WRONG: Creating new connections (slow, deprecated)
// Don't create new SSH connections per command
```

**Error Handling (Clippy Enforced):**

```rust
// ✅ CORRECT: Use Result<T, String> with contextual messages
let job = select_next_job(&db).await
    .map_err(|e| format!("Failed to select next job: {}", e))?;

// ❌ WRONG: Clippy denies unwrap/expect
let job = select_next_job(&db).await.unwrap(); // DENIED
```

### Architecture Compliance

**Module Isolation (CRITICAL):**

- ❌ DO NOT modify `db.rs` - it's the client-side database (Alpha)
- ❌ DO NOT modify `ssh/executor.rs` - it's an Alpha module
- ❌ DO NOT modify `job.rs` - it's for individual job operations (Alpha)
- ✅ CREATE new `queue_service.rs` module for queue orchestration
- ✅ EXTEND `lib.rs` with `mod queue_service;` declaration
- ✅ EXTEND `commands.rs` with new Tauri commands
- ✅ EXTEND `state.rs` with `Arc<Mutex<QueueManager>>`

**API Contract:**

```rust
// New Tauri commands for Story 2.4
#[tauri::command]
pub async fn start_queue_processing(state: State<'_, AppState>) -> Result<(), String>;

#[tauri::command]
pub async fn get_queue_status(state: State<'_, AppState>) -> Result<QueueStatus, String>;
```

```typescript
// New api.ts wrappers (frontend)
export async function startQueueProcessing(): Promise<void>;
export async function getQueueStatus(): Promise<QueueStatus>;

interface QueueStatus {
  isProcessing: boolean;
  currentJobId: string | null;
  pendingCount: number;
}
```

### Testing Requirements

**Unit Tests (in queue_service.rs):**

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_select_next_job_returns_lowest_position() -> Result<(), Box<dyn std::error::Error>> {
        let db = setup_test_db().await?;

        // Insert 3 jobs with different positions
        insert_job(&db, "job-1", 3).await?;
        insert_job(&db, "job-2", 1).await?;
        insert_job(&db, "job-3", 2).await?;

        let next = select_next_job(&db).await?;
        assert_eq!(next.unwrap().id, "job-2"); // Lowest position

        Ok(())
    }

    #[test]
    fn test_parse_sql_output() {
        let output = "completed|0|2026-01-11T14:23:45Z";
        let (status, exit_code, completed_at) = parse_sql_output(output).unwrap();

        assert_eq!(status, "completed");
        assert_eq!(exit_code, Some(0));
        assert_eq!(completed_at, Some("2026-01-11T14:23:45Z".to_string()));
    }

    #[test]
    fn test_generate_tmux_session_name() {
        let job_id = "550e8400-e29b-41d4-a716-446655440000";
        let session = format!("solverpilot_{}_{}", whoami::username(), &job_id[..8]);

        assert!(session.starts_with("solverpilot_"));
        assert!(session.ends_with("550e8400"));
    }

    #[test]
    fn test_rsync_command_format() {
        let cmd = "rsync -avz --delete --exclude '.git' --exclude '__pycache__' /local user@host:/remote";
        assert!(cmd.contains("--delete"));
        assert!(cmd.contains("--exclude '.git'"));
        assert!(cmd.contains("--exclude '__pycache__'"));
    }

    #[tokio::test]
    async fn test_wrapper_invocation_format() -> Result<(), Box<dyn std::error::Error>> {
        let job_id = "test-job-123";
        let benchmark = "bench.py";
        let cmd = format!(
            "~/.solverpilot/bin/job_wrapper.sh {} python3 {}",
            job_id, benchmark
        );

        assert!(cmd.starts_with("~/.solverpilot/bin/job_wrapper.sh"));
        assert!(cmd.contains("test-job-123"));
        assert!(cmd.contains("python3"));

        Ok(())
    }
}
```

**Integration Test (via manual testing):**

```bash
# After implementing, test manually:
# 1. Queue 3 jobs via UI
# 2. Start queue processing
# 3. Verify jobs execute in order (check tmux sessions)
ssh user@server "tmux ls"  # Should show: solverpilot_user_<job_id>

# 4. Verify server DB updated
ssh user@server "sqlite3 ~/.solverpilot-server/server.db 'SELECT id, status FROM jobs'"
# Should show: running → completed progression

# 5. Verify local DB synced
# Check local database: status should update to completed

# 6. Verify auto-start of next job
# Job #2 should start automatically after job #1 completes

# 7. Test failure scenario - kill job mid-execution
ssh user@server "tmux kill-session -t solverpilot_user_<job_id>"
# Queue should continue to next job
```

### Project Structure Notes

**Files to Create:**

```
src-tauri/src/queue_service.rs    # NEW (Story 2.4) - Queue execution engine
```

**Files to Modify:**

```
src-tauri/src/lib.rs               # ADD: mod queue_service;
src-tauri/src/commands.rs          # ADD: start_queue_processing, get_queue_status
src-tauri/src/state.rs             # ADD: queue_manager: Arc<Mutex<QueueManager>>
src/lib/api.ts                     # ADD: startQueueProcessing(), getQueueStatus()
src/lib/types.ts                   # ADD: QueueStatus interface
```

**Files NOT to Modify:**

```
src-tauri/src/db.rs                # EXISTS - Alpha (DO NOT MODIFY)
src-tauri/src/job.rs               # EXISTS - Alpha (DO NOT MODIFY)
src-tauri/src/ssh/executor.rs      # EXISTS - Alpha (DO NOT MODIFY)
src-tauri/scripts/job_wrapper.sh   # EXISTS - Story 2.1 (DO NOT MODIFY)
src-tauri/src/server_db.rs         # EXISTS - Story 2.2 (DO NOT MODIFY)
src-tauri/src/wrapper.rs           # EXISTS - Story 2.3 (DO NOT MODIFY)
```

### References

**Epic 2 Overview:**
[Source: _bmad-output/planning-artifacts/epics.md#Epic 2, lines 1510-2027]

- User Outcome: Sequential queue execution with 99.99% state reliability
- FRs Covered: FR155-FR163 + Architecture requirements

**Story 2.4 Requirements:**
[Source: _bmad-output/planning-artifacts/epics.md#Story 2.4, lines 1753-1832]

- Sequential execution (max_concurrent = 1)
- Rsync project files before execution
- Tmux session with wrapper invocation
- Poll server DB every 2 seconds for completion
- Auto-start next job after current completes

**Architecture - Queue Service:**
[Source: _bmad-output/planning-artifacts/architecture.md, lines 2945-3120]

- QueueManager struct with background task
- start_processing() spawns Tokio task
- Polling loop with 2-second interval
- Graceful failure handling

**Previous Stories:**
[Source: _bmad-output/implementation-artifacts/2-1-bash-wrapper-script-state-capture-foundation.md]
[Source: _bmad-output/implementation-artifacts/2-2-server-side-sqlite-database-schema-initialization.md]
[Source: _bmad-output/implementation-artifacts/2-3-wrapper-deployment-via-ssh.md]

- Story 2.1: Wrapper script with trap EXIT, double-write pattern
- Story 2.2: Server DB schema, init_server_db command
- Story 2.3: Wrapper deployment, idempotent installation

**Project Context:**
[Source: _bmad-output/project-context.md]

- Rust error handling: Result<T, String>, never unwrap/expect
- Tokio patterns: spawn for background tasks, interval for polling
- Module isolation: New Beta 1 modules in separate files
- SSH reuse: bb8 connection pooling for 10x performance

**Tokio Best Practices:**
[Source: Tokio documentation - tokio.rs/tokio/tutorial/spawning]

- Use tokio::spawn for CPU-bound or long-running tasks
- Use tokio::time::interval for periodic tasks
- Always handle task cancellation gracefully

**Rsync Best Practices:**
[Source: Rsync documentation - linux.die.net/man/1/rsync]

- Use --delete to mirror source to destination
- Use --exclude to skip unnecessary files (.git, **pycache**)
- Use -avz for archive mode, verbose, compression

### FRs Fulfilled

**From Epic 2 Requirements:**

This story fulfills the **queue execution engine** requirements:

- FR155: Sequential job execution (one at a time)
- FR156: Auto-start next job after current completes
- FR159: Job persistence (tmux sessions continue after disconnect)

**Story Dependency Chain:**

- Story 2.1 ✅: Wrapper script ready
- Story 2.2 ✅: Server DB schema ready
- Story 2.3 ✅: Wrapper deployed to remote
- **Story 2.4**: Queue execution (THIS STORY)
- Story 2.5: Start/Pause/Resume controls build on this
- Story 2.6: Reconciliation validates this execution flow

## Dev Agent Record

### Agent Model Used

{{agent_model_name_version}}

### Debug Log References

### Completion Notes List

### File List
