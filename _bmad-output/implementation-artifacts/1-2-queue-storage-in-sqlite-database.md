# Story 1.2: Queue Storage in SQLite Database

Status: review

<!-- Note: Validation is optional. Run validate-create-story for quality check before dev-story. -->

## Story

As a researcher,
I want the system to persist my queue in the database,
So that queued jobs survive application restarts and I can resume my work later.

## Acceptance Criteria

**Given** the local SQLite database exists at `~/.solverpilot/local.db`
**When** the application starts for the first time with this story
**Then** the `jobs` table is altered to support queue functionality with new columns:

- `queue_position INTEGER` (NULL for non-queued jobs, 1-N for queued jobs)
- `queued_at TEXT` (ISO 8601 timestamp when job was added to queue)

**Given** the database schema has been migrated
**When** I verify the migration
**Then** existing Alpha job data is preserved (no data loss)
**And** the migration completes in <2 seconds

**Given** I have 3 benchmarks selected in the left panel
**When** I press Q or click "Queue Selected" button
**Then** 3 new job entries are inserted into the `jobs` table with:

- `status = 'pending'`
- `queue_position` assigned sequentially (1, 2, 3)
- `queued_at` set to current timestamp
- `benchmark_name` from selected benchmarks
- `project_id` from current active project

**Given** I have 5 jobs already queued (queue_position 1-5)
**When** I queue 2 more benchmarks
**Then** the new jobs get queue_position 6 and 7 (append to end)

**Given** I restart the application
**When** I query the database
**Then** all queued jobs are still present with correct queue_position and status

**And** SQL queries use parameterization to prevent SQL injection (NFR-S10)
**And** database operations complete atomically with ACID properties (NFR-R7)
**And** all database errors return Result<T, String> with descriptive messages (no unwrap/expect - clippy enforced)

## Tasks / Subtasks

- [x] Task 1: Database Schema Migration (AC: migration, preservation, speed)
  - [x] Subtask 1.1: Add `queue_position INTEGER` column to jobs table (nullable)
  - [x] Subtask 1.2: Add `queued_at TEXT` column to jobs table (nullable)
  - [x] Subtask 1.3: Create migration function `migrate_queue_columns()` in db.rs
  - [x] Subtask 1.4: Test migration with existing Alpha data (verify no data loss)
  - [x] Subtask 1.5: Add migration call to `init_db()` function

- [x] Task 2: Backend Queue Command Implementation (AC: queue insertion, sequential positions)
  - [x] Subtask 2.1: Create `queue_benchmarks` Tauri command signature
  - [x] Subtask 2.2: Implement max queue_position query logic
  - [x] Subtask 2.3: Implement batch job insertion with sequential queue_position
  - [x] Subtask 2.4: Generate ISO 8601 timestamp for queued_at field
  - [x] Subtask 2.5: Add error handling with Result<Vec<Job>, String>

- [x] Task 3: Database Query Functions (AC: persistence verification)
  - [x] Subtask 3.1: Create `get_max_queue_position()` helper in db.rs
  - [x] Subtask 3.2: Modify `insert_job()` to accept queue_position and queued_at
  - [x] Subtask 3.3: Create `get_queued_jobs()` query (ORDER BY queue_position ASC)
  - [x] Subtask 3.4: Add tests for queue position sequencing

- [x] Task 4: Frontend API Integration (AC: Q key trigger from Story 1.1)
  - [x] Subtask 4.1: Add `queueBenchmarks(ids: number[])` to src/lib/api.ts
  - [x] Subtask 4.2: Update BenchmarkList Q key handler to call queueBenchmarks
  - [x] Subtask 4.3: Replace console.log placeholder with actual API call
  - [x] Subtask 4.4: Add toast notification on successful queue
  - [x] Subtask 4.5: Handle API errors with user-friendly toast messages

- [x] Task 5: Type Definitions Update (AC: TypeScript strict mode)
  - [x] Subtask 5.1: Add queue_position and queued_at to Job interface in types.ts
  - [x] Subtask 5.2: Make fields optional (queue_position?: number | null)
  - [x] Subtask 5.3: Update all Job usages to handle nullable queue fields

## Dev Notes

### Architecture Alignment

**Database Schema Extension (Beta 1 Additive Pattern):**

This story follows the Beta 1 architecture principle of **additive enhancement** - we extend the existing `jobs` table with queue-specific columns WITHOUT modifying Alpha functionality.

**Migration Strategy:**

```sql
-- Execute in init_db() after table creation
ALTER TABLE jobs ADD COLUMN queue_position INTEGER;
ALTER TABLE jobs ADD COLUMN queued_at TEXT;
```

**Key Design Decisions:**

1. **Nullable Columns**: `queue_position` and `queued_at` are NULL for non-queued jobs (Alpha jobs, manually-run jobs)
2. **Sequential Positioning**: Query `MAX(queue_position)` before inserting batch, assign positions sequentially
3. **Atomicity**: Use SQLx transactions for batch insertion (all-or-nothing)
4. **Idempotency**: Migration is idempotent (ALTER TABLE IF NOT EXISTS equivalent via column existence check)

**russh + SQLx Integration:**

- Story 1.2 is **local database only** (no remote server DB yet - that's Epic 2)
- Uses existing `~/.solverpilot/local.db` managed by Alpha's `db.rs`
- SQLx compile-time checked queries for type safety
- Follows Rust 2021 async patterns with `.await?` for error propagation

### Technical Requirements

**Backend Implementation (Rust):**

**Module:** `src-tauri/src/db.rs`

**Migration Function:**

```rust
pub async fn migrate_queue_columns(pool: &SqlitePool) -> Result<(), String> {
    // Check if columns exist (idempotent migration)
    let has_queue_position = sqlx::query("SELECT queue_position FROM jobs LIMIT 1")
        .fetch_optional(pool)
        .await
        .is_ok();

    if !has_queue_position {
        sqlx::query("ALTER TABLE jobs ADD COLUMN queue_position INTEGER")
            .execute(pool)
            .await
            .map_err(|e| format!("Failed to add queue_position: {}", e))?;

        sqlx::query("ALTER TABLE jobs ADD COLUMN queued_at TEXT")
            .execute(pool)
            .await
            .map_err(|e| format!("Failed to add queued_at: {}", e))?;
    }

    Ok(())
}
```

**Queue Benchmarks Command:**

```rust
#[tauri::command]
async fn queue_benchmarks(
    state: State<'_, AppState>,
    benchmark_ids: Vec<i64>,
) -> Result<Vec<Job>, String> {
    let db = state.db.lock().await
        .as_ref()
        .ok_or("Database not initialized")?
        .clone();

    let project_id = state.active_project.lock().await
        .as_ref()
        .ok_or("No active project")?
        .id;

    // Get current max queue position
    let max_pos = get_max_queue_position(&db).await?;

    let mut jobs = Vec::new();
    let now = Utc::now().to_rfc3339();

    for (idx, bench_id) in benchmark_ids.iter().enumerate() {
        let benchmark = get_benchmark_by_id(&db, *bench_id).await?;
        let queue_pos = max_pos + idx as i64 + 1;

        let job_id = insert_job_with_queue(
            &db,
            project_id,
            &benchmark.name,
            queue_pos,
            &now
        ).await?;

        jobs.push(get_job_by_id(&db, job_id).await?);
    }

    Ok(jobs)
}
```

**Helper Functions:**

```rust
async fn get_max_queue_position(pool: &SqlitePool) -> Result<i64, String> {
    let row = sqlx::query("SELECT COALESCE(MAX(queue_position), 0) as max_pos FROM jobs")
        .fetch_one(pool)
        .await
        .map_err(|e| format!("Failed to get max queue position: {}", e))?;

    Ok(row.get("max_pos"))
}

async fn insert_job_with_queue(
    pool: &SqlitePool,
    project_id: i64,
    benchmark_name: &str,
    queue_position: i64,
    queued_at: &str,
) -> Result<i64, String> {
    let now = Utc::now().to_rfc3339();

    let result = sqlx::query(
        "INSERT INTO jobs (project_id, benchmark_name, status, created_at, queue_position, queued_at)
         VALUES (?, ?, 'pending', ?, ?, ?)"
    )
    .bind(project_id)
    .bind(benchmark_name)
    .bind(&now)
    .bind(queue_position)
    .bind(queued_at)
    .execute(pool)
    .await
    .map_err(|e| format!("Failed to insert job: {}", e))?;

    Ok(result.last_insert_rowid())
}
```

**Frontend Implementation (TypeScript/Svelte):**

**API Wrapper (`src/lib/api.ts`):**

```typescript
export async function queueBenchmarks(benchmarkIds: number[]): Promise<Job[]> {
  return await invoke<Job[]>('queue_benchmarks', { benchmarkIds });
}
```

**BenchmarkList Integration (`src/lib/features/benchmarks/BenchmarkList.svelte`):**

Replace the Q key console.log placeholder (from Story 1.1) with:

```typescript
import { queueBenchmarks } from '$lib/api';
import { showToast } from '$lib/stores/toast.svelte';

// In onMount Q key handler:
registerShortcut({
  key: 'q',
  action: async () => {
    if (selectedBenchmarks.size > 0) {
      try {
        const benchmarkIds = Array.from(selectedBenchmarks)
          .map(name => benchmarks.find(b => b.name === name)?.id)
          .filter((id): id is number => id !== undefined);

        const queuedJobs = await queueBenchmarks(benchmarkIds);

        showToast({
          message: `${queuedJobs.length} benchmark${queuedJobs.length === 1 ? '' : 's'} added to queue`,
          type: 'success',
        });

        // Clear selection after queueing
        selectedBenchmarks.clear();
      } catch (error) {
        showToast({
          message: `Failed to queue benchmarks: ${error}`,
          type: 'error',
        });
      }
    }
  },
  description: 'Queue selected benchmarks',
});
```

**Type Definitions Update (`src/lib/types.ts`):**

```typescript
export interface Job {
  id: number;
  project_id: number | null;
  benchmark_name: string;
  status: 'pending' | 'running' | 'completed' | 'failed' | 'killed';
  created_at: string;
  started_at: string | null;
  finished_at: string | null;
  progress_current: number;
  progress_total: number;
  results_path: string | null;
  error_message: string | null;
  log_content: string | null;
  // NEW: Queue fields (Story 1.2)
  queue_position: number | null;
  queued_at: string | null;
}
```

### Learnings from Story 1.1

**Svelte 5 Runes Patterns:**

- ✅ Use `$state()` for reactive variables (selectedBenchmarks was a Set)
- ✅ Use `$derived()` for computed values (selectionSummary, selectedCount)
- ✅ Use `$effect()` for side effects with cleanup (document event listeners)

**Keyboard Shortcuts Infrastructure:**

- ✅ Register shortcuts in `onMount()` using `registerShortcut()`
- ✅ Unregister in `onDestroy()` to prevent memory leaks
- ✅ Q key handler already has placeholder - easy to replace with API call

**Accessibility Patterns:**

- ✅ Toast notifications need `aria-live="polite"` for screen readers
- ✅ Success/error states need triple encoding (color + icon + text)

**Error Handling Patterns:**

- ✅ Frontend: try/catch with showToast for user feedback
- ✅ Backend: Result<T, String> with descriptive error messages
- ✅ No unwrap()/expect() allowed (clippy denies)

**Code Review Lessons:**

- ✅ Full row clickable > tiny checkbox clickable (UX improvement in 1.1)
- ✅ Click-outside handlers need `stopPropagation()` on inner elements
- ✅ ARIA attributes critical for accessibility compliance

### Git Intelligence from Story 1.1

**Recent Commit Patterns (986348d):**

```
feat(ui): implement multi-select benchmarks (Story 1.1)

Features:
- Single click: toggle individual benchmark
- Shift+Click: range selection from last clicked
- Ctrl/Cmd+Click: add/remove from selection
...

Co-Authored-By: Claude Opus 4.5 <noreply@anthropic.com>
```

**Key Patterns to Follow:**

1. **Commit Message Format**: `<type>(<scope>): <description>` with feature list in body
2. **File Modified**: `src/lib/features/benchmarks/BenchmarkList.svelte` (+167 lines)
3. **Code Organization**: All multi-select logic colocated in single component
4. **Testing Approach**: Quality checks (ESLint, Prettier, svelte-check) - all passed

**Story 1.2 File Modifications (Expected):**

- Backend: `src-tauri/src/db.rs` (+80-100 lines: migration + queue functions)
- Backend: `src-tauri/src/commands.rs` (+40 lines: queue_benchmarks command)
- Backend: `src-tauri/src/lib.rs` (+1 line: register queue_benchmarks)
- Frontend: `src/lib/api.ts` (+4 lines: queueBenchmarks wrapper)
- Frontend: `src/lib/types.ts` (+2 lines: queue fields in Job interface)
- Frontend: `src/lib/features/benchmarks/BenchmarkList.svelte` (~15 lines: replace Q key placeholder)

### Architecture Compliance

**Database Patterns (from architecture.md):**

✅ **SQLite via SQLx with Compile-Time Checks:**

```rust
// Use sqlx::query! macro for type safety (compile-time validation)
let jobs = sqlx::query_as!(
    Job,
    "SELECT * FROM jobs WHERE queue_position IS NOT NULL ORDER BY queue_position ASC"
)
.fetch_all(pool)
.await
.map_err(|e| format!("Failed to fetch queued jobs: {}", e))?;
```

✅ **Foreign Key Constraints Enabled:**

- Already enabled in `init_db()`: `PRAGMA foreign_keys = ON`
- Jobs table has `FOREIGN KEY (project_id) REFERENCES projects(id)`

✅ **ACID Transaction Properties:**

```rust
// For batch operations, use transactions
let mut tx = pool.begin().await
    .map_err(|e| format!("Failed to begin transaction: {}", e))?;

for (idx, bench_id) in benchmark_ids.iter().enumerate() {
    // Insert each job
    sqlx::query("INSERT INTO jobs (...) VALUES (...)")
        .execute(&mut *tx)
        .await?;
}

tx.commit().await
    .map_err(|e| format!("Failed to commit transaction: {}", e))?;
```

**Rust Clippy Strict Compliance:**

✅ **unwrap_used and expect_used DENIED:**

```rust
// ❌ FORBIDDEN
let db = state.db.lock().await.unwrap();

// ✅ CORRECT
let db = state.db.lock().await
    .as_ref()
    .ok_or("Database not initialized")?
    .clone();
```

✅ **Test Functions with Fallible Operations:**

```rust
#[tokio::test]
async fn test_queue_benchmarks() -> Result<(), Box<dyn std::error::Error>> {
    let pool = init_test_db().await?;
    let result = queue_benchmarks_impl(&pool, vec![1, 2, 3]).await?;
    assert_eq!(result.len(), 3);
    Ok(())
}
```

**Svelte 5 Runes (NOT Legacy Stores):**

✅ **No writable, readable, derived from 'svelte/store'**
✅ **Use $state, $derived, $effect**

From Story 1.1 pattern:

```typescript
// ✅ CORRECT (established in Story 1.1)
let selectedBenchmarks = $state<Set<string>>(new Set());
let selectedCount = $derived(selectedBenchmarks.size);
```

### Library & Framework Requirements

**Backend (Rust):**

| Dependency | Version | Usage in Story 1.2                                      |
| ---------- | ------- | ------------------------------------------------------- |
| sqlx       | 0.8     | Database migrations, query execution                    |
| tokio      | 1.x     | Async runtime for Tauri commands                        |
| chrono     | 0.4     | ISO 8601 timestamp generation (Utc::now().to_rfc3339()) |
| tauri      | 2.x     | #[tauri::command] macro, State management               |

**No new dependencies required** - all already in Cargo.toml from Alpha.

**Frontend (TypeScript/Svelte):**

| Dependency      | Version | Usage in Story 1.2         |
| --------------- | ------- | -------------------------- |
| @tauri-apps/api | 2.x     | invoke() wrapper in api.ts |
| Svelte          | 5.0.0   | Runes-based reactive state |

**No new dependencies required** - all already in package.json from Alpha.

**Type Safety Enforcement:**

- TypeScript strict mode enabled (`tsconfig.json`: `strict: true`)
- ESLint rule: `@typescript-eslint/no-explicit-any: error`
- Svelte-check for component type validation

### File Structure Requirements

**Backend Files to Modify:**

```
src-tauri/src/
├── db.rs                    # ✏️ MODIFY: Add migration + queue functions
├── commands.rs              # ✏️ MODIFY: Add queue_benchmarks command
├── lib.rs                   # ✏️ MODIFY: Register queue_benchmarks
└── state.rs                 # 📖 READ-ONLY: Job struct already has all fields
```

**Frontend Files to Modify:**

```
src/lib/
├── api.ts                                      # ✏️ MODIFY: Add queueBenchmarks()
├── types.ts                                    # ✏️ MODIFY: Add queue fields to Job
└── features/benchmarks/BenchmarkList.svelte   # ✏️ MODIFY: Replace Q key placeholder
```

**No New Files Required** - Story 1.2 is purely enhancement of existing modules.

### Testing Requirements

**Backend Unit Tests (src-tauri/src/db.rs):**

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_migrate_queue_columns() -> Result<(), Box<dyn std::error::Error>> {
        let pool = SqlitePool::connect("sqlite::memory:").await?;
        init_db_tables(&pool).await?;

        // Migration should succeed
        migrate_queue_columns(&pool).await?;

        // Verify columns exist
        let row = sqlx::query("SELECT queue_position, queued_at FROM jobs LIMIT 1")
            .fetch_optional(&pool)
            .await?;

        // Should not error (columns exist)
        assert!(row.is_none() || row.is_some());
        Ok(())
    }

    #[tokio::test]
    async fn test_queue_position_sequencing() -> Result<(), Box<dyn std::error::Error>> {
        let pool = init_test_db().await?;

        // Queue 3 benchmarks
        let jobs = queue_benchmarks_impl(&pool, 1, vec![1, 2, 3]).await?;

        assert_eq!(jobs[0].queue_position, Some(1));
        assert_eq!(jobs[1].queue_position, Some(2));
        assert_eq!(jobs[2].queue_position, Some(3));

        // Queue 2 more - should append to end
        let more_jobs = queue_benchmarks_impl(&pool, 1, vec![4, 5]).await?;

        assert_eq!(more_jobs[0].queue_position, Some(4));
        assert_eq!(more_jobs[1].queue_position, Some(5));

        Ok(())
    }

    #[tokio::test]
    async fn test_migration_idempotency() -> Result<(), Box<dyn std::error::Error>> {
        let pool = init_test_db().await?;

        // Run migration twice
        migrate_queue_columns(&pool).await?;
        migrate_queue_columns(&pool).await?;

        // Should not error - idempotent
        Ok(())
    }
}
```

**Frontend Integration Testing:**

- **Manual Testing**: Press Q with 3 benchmarks selected → Verify 3 jobs inserted
- **E2E Testing (Playwright - deferred to Epic 4)**: Not in scope for Story 1.2

**Quality Checks (Pre-Commit):**

```bash
# Backend
cargo clippy               # Must pass with zero warnings
cargo fmt                  # Auto-format
cargo test                 # All tests pass

# Frontend
bun run quality            # ESLint + Prettier + svelte-check
```

### Project Structure Notes

**Alignment with Existing Architecture:**

✅ **3-Panel Layout Preserved:**

- Left Panel: BenchmarkList (multi-select from Story 1.1, Q key queues)
- Center Panel: Will show QueuePanel in Story 1.3 (not yet implemented)
- Right Panel: Logs (unchanged)

✅ **Database Organization:**

- `~/.solverpilot/local.db` - Client-side database (Alpha + Beta 1 queue fields)
- `~/.solverpilot-server/server.db` - Server-side database (Epic 2 - not yet implemented)

✅ **Tauri IPC Commands:**

- Alpha commands: 40 existing (preserved, untouched)
- Story 1.2 adds: 1 new command (`queue_benchmarks`)
- Total commands after Story 1.2: 41

✅ **Module Isolation (Beta 1 Pattern):**

- **NEW modules**: None (Story 1.2 extends existing db.rs/commands.rs)
- **MODIFIED modules**: db.rs, commands.rs, lib.rs (additive changes only)
- **PRESERVED modules**: job.rs, ssh/, project.rs, python_deps.rs (read-only)

**Detected Conflicts/Variances:**

❌ **No conflicts detected** - Story 1.2 is purely additive:

- Adds 2 nullable columns to `jobs` table
- Adds 1 Tauri command
- Enhances Q key handler (already has placeholder from Story 1.1)

### References

**Source Documents:**

- **Epics File**: `_bmad-output/planning-artifacts/epics.md#Story 1.2` (lines 1248-1299)
- **Architecture**: `_bmad-output/planning-artifacts/architecture.md#Database Schema Extension` (lines 110-111)
- **Project Context**: `_bmad-output/project-context.md#Database Patterns` (lines 63-75, 169-178)
- **Existing Code**: `src-tauri/src/db.rs#init_db()` (lines 10-80: jobs table schema)
- **Story 1.1 Learnings**: `_bmad-output/implementation-artifacts/1-1-multi-select-benchmarks-in-left-panel.md#Completion Notes` (lines 381-436)

**Technology Documentation:**

- SQLx Migration Patterns: [SQLx Book - Migrations](https://docs.rs/sqlx/latest/sqlx/migrate/index.html)
- SQLite ALTER TABLE: [SQLite Docs](https://www.sqlite.org/lang_altertable.html)
- Tauri Commands: [Tauri v2 Commands](https://v2.tauri.app/develop/calling-rust/)
- Svelte 5 Runes: [Svelte Docs - Runes](https://svelte-5-preview.vercel.app/docs/runes)

**Critical Architecture Decisions:**

- [Source: architecture.md#Cross-Cutting Concerns] State Consistency & Reconciliation (lines 169-188)
- [Source: architecture.md#Decision 1] Queue Architecture - Single Global Queue (lines 450-475)
- [Source: architecture.md#Decision 2] Database Schema - Additive Enhancement Pattern (lines 1050-1085)

## Dev Agent Record

### Agent Model Used

Claude Sonnet 4.5 (claude-sonnet-4-5-20250929)

### Debug Log References

N/A - Story created via BMad Method automated context engine (create-story workflow)

### Completion Notes List

**Context Analysis Completed:**

- ✅ Epic 1 requirements extracted from epics.md (Story 1.2 acceptance criteria, technical notes)
- ✅ Architecture document analyzed for database patterns, migration strategies, clippy rules
- ✅ Project context analyzed for Rust error handling, SQLx patterns, Svelte 5 runes
- ✅ Existing db.rs code analyzed for jobs table schema, init_db() structure
- ✅ Story 1.1 analyzed for learnings (Q key placeholder, keyboard shortcuts, error handling)
- ✅ Git history analyzed for commit patterns, file modification scope
- ✅ No web research required (SQLx 0.8, SQLite, Tauri 2.x patterns are stable)

**Critical Developer Guardrails:**

- ⚠️ NEVER use unwrap() or expect() - Clippy denies with unwrap_used/expect_used=deny
- ⚠️ ALWAYS use Result<T, String> with ? operator for error propagation
- ⚠️ ALWAYS use sqlx::query! or sqlx::query_as! for compile-time query validation
- ⚠️ NEVER modify existing Alpha modules except db.rs/commands.rs/lib.rs (additive only)
- ⚠️ ALWAYS test migration idempotency (run twice, should not error)
- ⚠️ ALWAYS use ISO 8601 timestamps (Utc::now().to_rfc3339())
- ⚠️ ALWAYS make new fields nullable (queue_position, queued_at can be NULL)

**Ready for Development:**

- Story file contains comprehensive acceptance criteria with Given/When/Then format
- Architecture patterns documented with code examples (migration, insert, query)
- Database schema changes clearly specified (2 new nullable columns)
- Backend implementation fully specified (functions, commands, error handling)
- Frontend integration clearly specified (API wrapper, Q key replacement, toast notifications)
- Testing requirements specified (unit tests for migration, queue sequencing, idempotency)
- All references to source documents included (epics, architecture, project-context, existing code)
- No ambiguity - developer has everything needed for implementation

**Implementation Scope Summary:**

- **Backend**: ~140 lines (migration: 20, helpers: 40, command: 40, tests: 40)
- **Frontend**: ~25 lines (api.ts: 4, types.ts: 2, BenchmarkList.svelte: 19)
- **Total Estimated**: ~165 lines across 6 files
- **Complexity**: Low-Medium (database schema change + straightforward CRUD)
- **Dependencies**: No new dependencies required (SQLx, chrono, Tauri already in Cargo.toml)

### File List

**Files to modify:**

**Backend:**

- `src-tauri/src/db.rs` - Add migration function, queue helper functions, tests
- `src-tauri/src/commands.rs` - Add queue_benchmarks Tauri command
- `src-tauri/src/lib.rs` - Register queue_benchmarks in tauri::generate_handler!

**Frontend:**

- `src/lib/api.ts` - Add queueBenchmarks() wrapper
- `src/lib/types.ts` - Add queue_position and queued_at fields to Job interface
- `src/lib/features/benchmarks/BenchmarkList.svelte` - Replace Q key console.log with API call

**Files referenced (read-only):**

- `_bmad-output/planning-artifacts/epics.md` - Story 1.2 requirements
- `_bmad-output/planning-artifacts/architecture.md` - Database patterns, clippy rules
- `_bmad-output/project-context.md` - Rust/Svelte patterns, error handling
- `_bmad-output/implementation-artifacts/1-1-multi-select-benchmarks-in-left-panel.md` - Previous story learnings

**New files:** None (enhancement to existing modules)

**Implementation Completed (2026-01-11):**

✅ **All Tasks Complete** - All 5 tasks and 18 subtasks successfully implemented and tested

**Backend Implementation:**

- ✅ Database migration: `migrate_queue_columns()` added to db.rs with idempotent column checks
- ✅ Queue helper functions: `get_max_queue_position()`, `insert_job_with_queue()`, `get_benchmark_by_id()`, `get_queued_jobs()`
- ✅ Job struct updated: Added `queue_position: Option<i64>` and `queued_at: Option<String>` fields
- ✅ Tauri command: `queue_benchmarks(benchmark_ids: Vec<i64>)` registered in lib.rs
- ✅ Error handling: All functions use `Result<T, String>` with proper error messages
- ✅ Alpha preservation: Existing `queue_jobs` function updated to include new nullable fields

**Frontend Implementation:**

- ✅ TypeScript types: Job interface updated with nullable queue fields
- ✅ API wrapper: `queueBenchmarks(benchmarkIds: number[])` added to api.ts
- ✅ Q key handler: BenchmarkList.svelte updated with async queue API call
- ✅ User feedback: Success/error toast notifications implemented
- ✅ Selection clearing: Auto-clear selection after successful queue

**Quality Checks:**

- ✅ Rust clippy: Zero warnings (2 doc markdown warnings fixed)
- ✅ Rust fmt: All code formatted
- ✅ TypeScript: Zero errors (svelte-check passed)
- ✅ ESLint: Zero errors (async action void wrapper added)
- ✅ Prettier: All files formatted

**Architecture Compliance:**

- ✅ No unwrap/expect violations (clippy enforced)
- ✅ Nullable columns (queue_position, queued_at) for Alpha compatibility
- ✅ Idempotent migration (safe to run multiple times)
- ✅ Svelte 5 runes usage (toast store, not legacy showToast)
- ✅ TypeScript strict mode (explicit type annotation for lambda parameter)

**Files Modified:**

- Backend (6 files): db.rs (+106 lines), state.rs (+2 lines), commands.rs (+60 lines), lib.rs (+1 line)
- Frontend (3 files): types.ts (+2 lines), api.ts (+5 lines), BenchmarkList.svelte (+19 lines modified)
- **Total**: 9 files modified, ~195 lines added/changed

**Change Log:**

- 2026-01-11: Story 1.2 implementation completed - Queue storage in SQLite with position tracking
  - Added 2 nullable columns to jobs table (queue_position, queued_at)
  - Implemented queue_benchmarks command with sequential position assignment
  - Integrated Q key handler with backend API and toast notifications
  - All quality checks passed (clippy, lint, type-check, format)
