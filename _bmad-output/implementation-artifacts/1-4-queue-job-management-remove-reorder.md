# Story 1.4: Queue Job Management - Remove & Reorder

Status: review

<!-- Note: Validation is optional. Run validate-create-story for quality check before dev-story. -->

## Story

As a researcher,
I want to remove specific jobs from the queue and reorder them,
So that I can adjust my queue dynamically as priorities change without starting over.

## Acceptance Criteria

**Given** I have 10 jobs in the queue with positions 1-10
**When** I click the "Remove" button (trash icon) on job #5
**Then** job #5 is deleted from the queue
**And** jobs #6-10 are renumbered to positions #5-9 (queue positions shift down)
**And** the UI updates immediately to reflect the change

**Given** I have pending jobs in the queue
**When** I select a job and press the Delete key
**Then** the job is removed from the queue (keyboard accessibility)

**Given** I have job #5 selected
**When** I click "Move to Front" button or action
**Then** job #5 becomes position #1
**And** previous jobs #1-4 shift to positions #2-5

**Given** I have job #3 selected
**When** I click "Move to End" button
**Then** job #3 moves to the last position in pending queue
**And** jobs #4+ shift up by one position

**Given** I have job #5 in the queue
**When** I drag job #5 and drop it at position #2
**Then** job #5 is reordered to position #2
**And** jobs #2-4 shift to positions #3-5 (make room)

**Given** I have jobs with status = 'running' or 'completed'
**When** I attempt to remove or reorder them
**Then** the action is blocked with a toast notification: "Cannot modify jobs that are running or completed"

**Given** I click "Cancel All Pending" button in Queue panel header
**When** I confirm the action
**Then** all jobs with status = 'pending' are deleted from the queue
**And** running/completed jobs remain untouched

**And** all queue modifications persist to the database immediately
**And** drag-and-drop provides visual feedback (ghost element, drop zones highlighted)
**And** undo is NOT required (user confirms destructive actions via modal)

## Tasks / Subtasks

- [x] Task 1: Backend - Remove Job from Queue (AC: remove job, renumber positions)
  - [x] Subtask 1.1: Create `remove_job_from_queue(job_id: i64)` function in db.rs
  - [x] Subtask 1.2: Implement position renumbering SQL after deletion
  - [x] Subtask 1.3: Add validation to prevent removal of running/completed jobs
  - [x] Subtask 1.4: Create Tauri command `remove_job_from_queue` in commands.rs
  - [x] Subtask 1.5: Register command in lib.rs

- [x] Task 2: Backend - Move Job to Front (AC: move to front, shift positions)
  - [x] Subtask 2.1: Create `move_job_to_front(job_id: i64)` function in db.rs
  - [x] Subtask 2.2: Implement SQL to set new position = 1 and shift others +1
  - [x] Subtask 2.3: Add validation for pending status only
  - [x] Subtask 2.4: Create Tauri command `move_job_to_front` in commands.rs

- [x] Task 3: Backend - Move Job to End (AC: move to end, shift positions)
  - [x] Subtask 3.1: Create `move_job_to_end(job_id: i64)` function in db.rs
  - [x] Subtask 3.2: Implement SQL to get max position and set job to max+1
  - [x] Subtask 3.3: Implement position recalculation for remaining jobs
  - [x] Subtask 3.4: Create Tauri command `move_job_to_end` in commands.rs

- [x] Task 4: Backend - Reorder Queue Job (AC: drag-drop reorder, arbitrary position)
  - [x] Subtask 4.1: Create `reorder_queue_job(job_id: i64, new_position: i32)` function in db.rs
  - [x] Subtask 4.2: Implement SQL to handle insertion at arbitrary position
  - [x] Subtask 4.3: Handle edge cases (move up vs move down logic)
  - [x] Subtask 4.4: Create Tauri command `reorder_queue_job` in commands.rs

- [x] Task 5: Backend - Cancel All Pending Jobs (AC: bulk delete pending)
  - [x] Subtask 5.1: Create `cancel_all_pending_jobs()` function in db.rs returning count
  - [x] Subtask 5.2: DELETE only status='pending' jobs, preserve running/completed
  - [x] Subtask 5.3: Create Tauri command `cancel_all_pending_jobs` in commands.rs

- [x] Task 6: Frontend - Add Job Action Buttons (AC: remove, move to front/end)
  - [x] Subtask 6.1: Add removeJob, moveJobToFront, moveJobToEnd, reorderJob, cancelAllPending to api.ts
  - [x] Subtask 6.2: Add trash icon button to each pending job item in QueuePanel.svelte
  - [x] Subtask 6.3: Add "Move to Front" and "Move to End" action buttons (context menu or icons)
  - [x] Subtask 6.4: Add keyboard handler for Delete key on focused job
  - [x] Subtask 6.5: Add "Cancel All Pending" button in Queue panel header

- [x] Task 7: Frontend - Drag and Drop Reordering (AC: drag-drop with visual feedback)
  - [x] Subtask 7.1: Implement HTML5 native drag-and-drop on pending job items
  - [x] Subtask 7.2: Add draggable="true" attribute and ondragstart handler
  - [x] Subtask 7.3: Implement ondragover and ondrop handlers for drop zones
  - [x] Subtask 7.4: Add visual feedback (ghost element opacity, drop zone highlighting)
  - [x] Subtask 7.5: Call reorderJob API on drop and refresh job list

- [x] Task 8: Frontend - Confirmation Modal (AC: confirm destructive actions)
  - [x] Subtask 8.1: Create ConfirmModal.svelte component in src/lib/ui/
  - [x] Subtask 8.2: Add modal for "Cancel All Pending" action
  - [x] Subtask 8.3: Integrate with existing modal/toast patterns from Story 1.3

- [x] Task 9: Backend Unit Tests (AC: all operations tested)
  - [x] Subtask 9.1: Test remove_job_from_queue renumbers positions correctly
  - [x] Subtask 9.2: Test move_job_to_front shifts existing jobs
  - [x] Subtask 9.3: Test move_job_to_end calculates max+1 position
  - [x] Subtask 9.4: Test reorder_queue_job handles up/down movement
  - [x] Subtask 9.5: Test cancel_all_pending preserves running/completed jobs
  - [x] Subtask 9.6: Test status validation (cannot modify running/completed)

## Dev Notes

### Architecture Alignment

**Extends Story 1.3 Components:**

This story directly extends the QueuePanel component from Story 1.3 (`src/lib/features/queue/QueuePanel.svelte`) by adding:

- Job action buttons (remove, move to front, move to end)
- Drag-and-drop reordering capability
- Cancel All Pending button in panel header
- Delete key keyboard handler

**Database Operations Pattern:**

All queue operations modify `queue_position` column added in Story 1.2. Position renumbering must be atomic within transactions to maintain FIFO integrity:

```sql
-- Remove job and renumber (within transaction)
DELETE FROM jobs WHERE id = ? AND status = 'pending';
UPDATE jobs SET queue_position = queue_position - 1 WHERE queue_position > ?;
```

**Status Validation (CRITICAL):**

Only jobs with `status = 'pending'` can be removed or reordered. Jobs with `status IN ('running', 'completed', 'failed', 'killed')` must be protected with clear error messages.

### Technical Requirements

**Backend Implementation (Rust):**

**Module:** `src-tauri/src/db.rs`

**Remove Job from Queue:**

```rust
/// Removes a pending job from the queue and renumbers remaining positions
/// Returns error if job is not pending (cannot remove running/completed jobs)
pub async fn remove_job_from_queue(pool: &SqlitePool, job_id: i64) -> Result<(), String> {
    // Start transaction for atomic operation
    let mut tx = pool.begin().await
        .map_err(|e| format!("Failed to begin transaction: {e}"))?;

    // Get current job status and position (verify pending + exists)
    let job = sqlx::query(
        "SELECT status, queue_position FROM jobs WHERE id = ? AND queue_position IS NOT NULL"
    )
    .bind(job_id)
    .fetch_optional(&mut *tx)
    .await
    .map_err(|e| format!("Failed to query job: {e}"))?
    .ok_or("Job not found in queue")?;

    let status: String = job.get("status");
    if status != "pending" {
        return Err(format!("Cannot remove job with status '{}'. Only pending jobs can be removed.", status));
    }

    let position: i64 = job.get("queue_position");

    // Delete the job
    sqlx::query("DELETE FROM jobs WHERE id = ?")
        .bind(job_id)
        .execute(&mut *tx)
        .await
        .map_err(|e| format!("Failed to delete job: {e}"))?;

    // Renumber remaining jobs (shift positions down)
    sqlx::query("UPDATE jobs SET queue_position = queue_position - 1 WHERE queue_position > ? AND status = 'pending'")
        .bind(position)
        .execute(&mut *tx)
        .await
        .map_err(|e| format!("Failed to renumber queue: {e}"))?;

    tx.commit().await
        .map_err(|e| format!("Failed to commit transaction: {e}"))?;

    Ok(())
}
```

**Move Job to Front:**

```rust
/// Moves a pending job to position #1, shifting others down
pub async fn move_job_to_front(pool: &SqlitePool, job_id: i64) -> Result<(), String> {
    let mut tx = pool.begin().await
        .map_err(|e| format!("Failed to begin transaction: {e}"))?;

    // Verify job is pending
    let job = sqlx::query("SELECT status, queue_position FROM jobs WHERE id = ?")
        .bind(job_id)
        .fetch_optional(&mut *tx)
        .await
        .map_err(|e| format!("Failed to query job: {e}"))?
        .ok_or("Job not found")?;

    let status: String = job.get("status");
    if status != "pending" {
        return Err(format!("Cannot move job with status '{}'. Only pending jobs can be reordered.", status));
    }

    let current_position: i64 = job.get("queue_position");
    if current_position == 1 {
        return Ok(()); // Already at front
    }

    // Shift jobs between position 1 and current position (exclusive) down by 1
    sqlx::query(
        "UPDATE jobs SET queue_position = queue_position + 1 WHERE queue_position < ? AND status = 'pending'"
    )
    .bind(current_position)
    .execute(&mut *tx)
    .await
    .map_err(|e| format!("Failed to shift jobs: {e}"))?;

    // Move target job to position 1
    sqlx::query("UPDATE jobs SET queue_position = 1 WHERE id = ?")
        .bind(job_id)
        .execute(&mut *tx)
        .await
        .map_err(|e| format!("Failed to move job: {e}"))?;

    tx.commit().await
        .map_err(|e| format!("Failed to commit transaction: {e}"))?;

    Ok(())
}
```

**Move Job to End:**

```rust
/// Moves a pending job to the end of the queue
pub async fn move_job_to_end(pool: &SqlitePool, job_id: i64) -> Result<(), String> {
    let mut tx = pool.begin().await
        .map_err(|e| format!("Failed to begin transaction: {e}"))?;

    // Verify job is pending
    let job = sqlx::query("SELECT status, queue_position FROM jobs WHERE id = ?")
        .bind(job_id)
        .fetch_optional(&mut *tx)
        .await
        .map_err(|e| format!("Failed to query job: {e}"))?
        .ok_or("Job not found")?;

    let status: String = job.get("status");
    if status != "pending" {
        return Err(format!("Cannot move job with status '{}'. Only pending jobs can be reordered.", status));
    }

    let current_position: i64 = job.get("queue_position");

    // Get max position
    let max_row = sqlx::query("SELECT COALESCE(MAX(queue_position), 0) as max_pos FROM jobs WHERE status = 'pending'")
        .fetch_one(&mut *tx)
        .await
        .map_err(|e| format!("Failed to get max position: {e}"))?;

    let max_position: i64 = max_row.get("max_pos");
    if current_position == max_position {
        return Ok(()); // Already at end
    }

    // Shift jobs after current position up by 1 (to fill the gap)
    sqlx::query(
        "UPDATE jobs SET queue_position = queue_position - 1 WHERE queue_position > ? AND status = 'pending'"
    )
    .bind(current_position)
    .execute(&mut *tx)
    .await
    .map_err(|e| format!("Failed to shift jobs: {e}"))?;

    // Move target job to end (max_position stays same since we shifted)
    sqlx::query("UPDATE jobs SET queue_position = ? WHERE id = ?")
        .bind(max_position)
        .bind(job_id)
        .execute(&mut *tx)
        .await
        .map_err(|e| format!("Failed to move job: {e}"))?;

    tx.commit().await
        .map_err(|e| format!("Failed to commit transaction: {e}"))?;

    Ok(())
}
```

**Reorder Queue Job (Arbitrary Position):**

```rust
/// Reorders a job to a new position, shifting other jobs accordingly
pub async fn reorder_queue_job(pool: &SqlitePool, job_id: i64, new_position: i32) -> Result<(), String> {
    let mut tx = pool.begin().await
        .map_err(|e| format!("Failed to begin transaction: {e}"))?;

    // Verify job is pending
    let job = sqlx::query("SELECT status, queue_position FROM jobs WHERE id = ?")
        .bind(job_id)
        .fetch_optional(&mut *tx)
        .await
        .map_err(|e| format!("Failed to query job: {e}"))?
        .ok_or("Job not found")?;

    let status: String = job.get("status");
    if status != "pending" {
        return Err(format!("Cannot reorder job with status '{}'. Only pending jobs can be reordered.", status));
    }

    let current_position: i64 = job.get("queue_position");
    let new_pos = i64::from(new_position);

    if current_position == new_pos {
        return Ok(()); // No change needed
    }

    if new_pos < current_position {
        // Moving up: shift jobs in range [new_pos, current_pos) down by 1
        sqlx::query(
            "UPDATE jobs SET queue_position = queue_position + 1 WHERE queue_position >= ? AND queue_position < ? AND status = 'pending'"
        )
        .bind(new_pos)
        .bind(current_position)
        .execute(&mut *tx)
        .await
        .map_err(|e| format!("Failed to shift jobs: {e}"))?;
    } else {
        // Moving down: shift jobs in range (current_pos, new_pos] up by 1
        sqlx::query(
            "UPDATE jobs SET queue_position = queue_position - 1 WHERE queue_position > ? AND queue_position <= ? AND status = 'pending'"
        )
        .bind(current_position)
        .bind(new_pos)
        .execute(&mut *tx)
        .await
        .map_err(|e| format!("Failed to shift jobs: {e}"))?;
    }

    // Set target job to new position
    sqlx::query("UPDATE jobs SET queue_position = ? WHERE id = ?")
        .bind(new_pos)
        .bind(job_id)
        .execute(&mut *tx)
        .await
        .map_err(|e| format!("Failed to move job: {e}"))?;

    tx.commit().await
        .map_err(|e| format!("Failed to commit transaction: {e}"))?;

    Ok(())
}
```

**Cancel All Pending Jobs:**

```rust
/// Deletes all pending jobs from queue, returns count deleted
pub async fn cancel_all_pending_jobs(pool: &SqlitePool) -> Result<u32, String> {
    let result = sqlx::query("DELETE FROM jobs WHERE status = 'pending' AND queue_position IS NOT NULL")
        .execute(pool)
        .await
        .map_err(|e| format!("Failed to cancel pending jobs: {e}"))?;

    // Safe: rows_affected is always non-negative
    #[allow(clippy::cast_possible_truncation)]
    Ok(result.rows_affected() as u32)
}
```

**Module:** `src-tauri/src/commands.rs`

**Tauri Commands:**

```rust
#[tauri::command]
async fn remove_job_from_queue(state: State<'_, AppState>, job_id: i64) -> Result<(), String> {
    let db = state.db.lock().await
        .as_ref()
        .ok_or("Database not initialized")?
        .clone();

    db::remove_job_from_queue(&db, job_id).await
}

#[tauri::command]
async fn move_job_to_front(state: State<'_, AppState>, job_id: i64) -> Result<(), String> {
    let db = state.db.lock().await
        .as_ref()
        .ok_or("Database not initialized")?
        .clone();

    db::move_job_to_front(&db, job_id).await
}

#[tauri::command]
async fn move_job_to_end(state: State<'_, AppState>, job_id: i64) -> Result<(), String> {
    let db = state.db.lock().await
        .as_ref()
        .ok_or("Database not initialized")?
        .clone();

    db::move_job_to_end(&db, job_id).await
}

#[tauri::command]
async fn reorder_queue_job(state: State<'_, AppState>, job_id: i64, new_position: i32) -> Result<(), String> {
    let db = state.db.lock().await
        .as_ref()
        .ok_or("Database not initialized")?
        .clone();

    db::reorder_queue_job(&db, job_id, new_position).await
}

#[tauri::command]
async fn cancel_all_pending_jobs(state: State<'_, AppState>) -> Result<u32, String> {
    let db = state.db.lock().await
        .as_ref()
        .ok_or("Database not initialized")?
        .clone();

    db::cancel_all_pending_jobs(&db).await
}
```

**Frontend Implementation (TypeScript/Svelte):**

**API Wrappers (`src/lib/api.ts`):**

```typescript
export async function removeJobFromQueue(jobId: number): Promise<void> {
  return await invoke<void>('remove_job_from_queue', { jobId });
}

export async function moveJobToFront(jobId: number): Promise<void> {
  return await invoke<void>('move_job_to_front', { jobId });
}

export async function moveJobToEnd(jobId: number): Promise<void> {
  return await invoke<void>('move_job_to_end', { jobId });
}

export async function reorderQueueJob(jobId: number, newPosition: number): Promise<void> {
  return await invoke<void>('reorder_queue_job', { jobId, newPosition });
}

export async function cancelAllPendingJobs(): Promise<number> {
  return await invoke<number>('cancel_all_pending_jobs');
}
```

**QueuePanel.svelte Additions:**

```svelte
<script lang="ts">
  import { onMount } from 'svelte';
  import {
    getAllQueueJobs,
    removeJobFromQueue,
    moveJobToFront,
    moveJobToEnd,
    reorderQueueJob,
    cancelAllPendingJobs,
  } from '$lib/api';
  import type { Job } from '$lib/types';
  import StatusBadge from '$lib/ui/StatusBadge.svelte';
  import ConfirmModal from '$lib/ui/ConfirmModal.svelte';
  import { toast } from '$lib/stores/toast.svelte';

  let jobs = $state<Job[]>([]);
  let showCancelAllModal = $state(false);
  let draggedJobId = $state<number | null>(null);

  // ... existing code from Story 1.3 ...

  // Job removal handler
  async function handleRemoveJob(jobId: number) {
    try {
      await removeJobFromQueue(jobId);
      toast.success('Job removed from queue');
      await loadJobs();
    } catch (error) {
      const message = error instanceof Error ? error.message : String(error);
      toast.error(message);
    }
  }

  // Move to front handler
  async function handleMoveToFront(jobId: number) {
    try {
      await moveJobToFront(jobId);
      await loadJobs();
    } catch (error) {
      const message = error instanceof Error ? error.message : String(error);
      toast.error(message);
    }
  }

  // Move to end handler
  async function handleMoveToEnd(jobId: number) {
    try {
      await moveJobToEnd(jobId);
      await loadJobs();
    } catch (error) {
      const message = error instanceof Error ? error.message : String(error);
      toast.error(message);
    }
  }

  // Cancel all pending handler
  async function handleCancelAllPending() {
    try {
      const count = await cancelAllPendingJobs();
      showCancelAllModal = false;
      toast.success(`Cancelled ${String(count)} pending jobs`);
      await loadJobs();
    } catch (error) {
      const message = error instanceof Error ? error.message : String(error);
      toast.error(message);
    }
  }

  // Drag and drop handlers
  function handleDragStart(event: DragEvent, jobId: number) {
    draggedJobId = jobId;
    if (event.dataTransfer) {
      event.dataTransfer.effectAllowed = 'move';
      event.dataTransfer.setData('text/plain', String(jobId));
    }
  }

  function handleDragOver(event: DragEvent) {
    event.preventDefault();
    if (event.dataTransfer) {
      event.dataTransfer.dropEffect = 'move';
    }
  }

  async function handleDrop(event: DragEvent, targetPosition: number) {
    event.preventDefault();
    if (draggedJobId === null) return;

    try {
      await reorderQueueJob(draggedJobId, targetPosition);
      await loadJobs();
    } catch (error) {
      const message = error instanceof Error ? error.message : String(error);
      toast.error(message);
    } finally {
      draggedJobId = null;
    }
  }

  // Keyboard handler for Delete key
  function handleKeyDown(event: KeyboardEvent, jobId: number, status: string) {
    if (event.key === 'Delete' && status === 'pending') {
      void handleRemoveJob(jobId);
    }
  }
</script>

<!-- Add Cancel All button to header -->
<div class="p-4 border-b border-slate-700/50 flex justify-between items-center">
  <div>
    <h2 class="text-lg font-semibold text-slate-200">Queue</h2>
    <p class="text-xs text-slate-400">{jobs.length} jobs</p>
  </div>
  {#if jobsByStatus.pending.length > 0}
    <button
      class="text-xs text-red-400 hover:text-red-300 px-3 py-1 rounded border border-red-500/30 hover:bg-red-500/10 transition-colors"
      onclick={() => {
        showCancelAllModal = true;
      }}
    >
      Cancel All Pending
    </button>
  {/if}
</div>

<!-- In pending jobs section, add action buttons and drag handlers -->
{#each jobsByStatus.pending as job, idx (job.id)}
  <div
    class="px-3 py-2 hover:bg-slate-700/50 rounded-lg transition-colors cursor-grab active:cursor-grabbing {idx %
      2 ===
    0
      ? 'bg-slate-800/30'
      : ''}"
    draggable="true"
    ondragstart={e => handleDragStart(e, job.id)}
    ondragover={handleDragOver}
    ondrop={e => {
      if (job.queue_position !== null) void handleDrop(e, job.queue_position);
    }}
    onkeydown={e => handleKeyDown(e, job.id, job.status)}
    tabindex="0"
  >
    <div class="flex items-center justify-between">
      <div class="flex items-center gap-2">
        {#if job.queue_position !== null}
          <span class="text-sm text-slate-400">#{String(job.queue_position)}</span>
        {/if}
        <span class="text-slate-200">{job.benchmark_name}</span>
      </div>
      <div class="flex items-center gap-2">
        <StatusBadge status={job.status} />
        <!-- Action buttons -->
        <button
          class="text-slate-400 hover:text-blue-400 p-1"
          onclick={() => {
            void handleMoveToFront(job.id);
          }}
          title="Move to Front"
          aria-label="Move to front of queue"
        >
          <svg class="w-4 h-4" fill="none" viewBox="0 0 24 24" stroke="currentColor">
            <path
              stroke-linecap="round"
              stroke-linejoin="round"
              stroke-width="2"
              d="M5 15l7-7 7 7"
            />
          </svg>
        </button>
        <button
          class="text-slate-400 hover:text-blue-400 p-1"
          onclick={() => {
            void handleMoveToEnd(job.id);
          }}
          title="Move to End"
          aria-label="Move to end of queue"
        >
          <svg class="w-4 h-4" fill="none" viewBox="0 0 24 24" stroke="currentColor">
            <path
              stroke-linecap="round"
              stroke-linejoin="round"
              stroke-width="2"
              d="M19 9l-7 7-7-7"
            />
          </svg>
        </button>
        <button
          class="text-slate-400 hover:text-red-400 p-1"
          onclick={() => {
            void handleRemoveJob(job.id);
          }}
          title="Remove from queue"
          aria-label="Remove job from queue"
        >
          <svg class="w-4 h-4" fill="none" viewBox="0 0 24 24" stroke="currentColor">
            <path
              stroke-linecap="round"
              stroke-linejoin="round"
              stroke-width="2"
              d="M19 7l-.867 12.142A2 2 0 0116.138 21H7.862a2 2 0 01-1.995-1.858L5 7m5 4v6m4-6v6m1-10V4a1 1 0 00-1-1h-4a1 1 0 00-1 1v3M4 7h16"
            />
          </svg>
        </button>
      </div>
    </div>
    <p class="text-sm text-slate-500 mt-1">{formatTimestamp(job)}</p>
  </div>
{/each}

<!-- Confirmation Modal -->
{#if showCancelAllModal}
  <ConfirmModal
    title="Cancel All Pending Jobs?"
    message="This will remove all {jobsByStatus.pending
      .length} pending jobs from the queue. Running and completed jobs will not be affected."
    confirmText="Cancel All"
    confirmClass="bg-red-600 hover:bg-red-700"
    onConfirm={handleCancelAllPending}
    onCancel={() => {
      showCancelAllModal = false;
    }}
  />
{/if}
```

**ConfirmModal Component (`src/lib/ui/ConfirmModal.svelte`):**

```svelte
<script lang="ts">
  interface Props {
    title: string;
    message: string;
    confirmText?: string;
    cancelText?: string;
    confirmClass?: string;
    onConfirm: () => void;
    onCancel: () => void;
  }

  const {
    title,
    message,
    confirmText = 'Confirm',
    cancelText = 'Cancel',
    confirmClass = 'bg-blue-600 hover:bg-blue-700',
    onConfirm,
    onCancel,
  }: Props = $props();
</script>

<div
  class="fixed inset-0 bg-black/50 backdrop-blur-sm flex items-center justify-center z-50"
  onclick={onCancel}
  onkeydown={e => e.key === 'Escape' && onCancel()}
  role="dialog"
  aria-modal="true"
  aria-labelledby="modal-title"
>
  <div
    class="bg-slate-800 rounded-xl border border-slate-700 p-6 max-w-md w-full mx-4 shadow-2xl"
    onclick={e => e.stopPropagation()}
    onkeydown={() => {}}
    role="document"
  >
    <h3 id="modal-title" class="text-lg font-semibold text-slate-100 mb-2">{title}</h3>
    <p class="text-slate-400 mb-6">{message}</p>
    <div class="flex justify-end gap-3">
      <button
        class="px-4 py-2 text-slate-300 hover:text-slate-100 transition-colors"
        onclick={onCancel}
      >
        {cancelText}
      </button>
      <button
        class="px-4 py-2 text-white rounded-lg transition-colors {confirmClass}"
        onclick={onConfirm}
      >
        {confirmText}
      </button>
    </div>
  </div>
</div>
```

### Learnings from Story 1.3

**Patterns to Follow:**

1. **Transaction Wrapping:** All queue operations that modify multiple rows MUST use transactions (Story 1.2 code review lesson)
2. **Status Validation:** Always verify job status before modification (pending only for remove/reorder)
3. **Error Handling:** Use `Result<T, String>` with descriptive error messages
4. **Toast Notifications:** Success/error feedback via existing toast store
5. **Svelte 5 Runes:** Continue using `$state`, `$derived`, `$props` patterns

**Code Review Fixes from Story 1.3 to Apply:**

- Accessibility: Use `text-sm` (14px) for all user-facing text (not `text-xs`)
- Include `aria-label` attributes on action buttons
- Add `role` and `tabindex` for keyboard navigation

### Architecture Compliance

**Svelte 5 Runes (NOT Legacy Stores):**

- `$state` for `draggedJobId`, `showCancelAllModal`
- Event handlers use Svelte 5 syntax: `onclick`, `ondragstart`, `ondragover`, `ondrop`
- No legacy `on:` directive syntax

**Rust Clippy Strict Compliance:**

- `unwrap_used` and `expect_used` DENIED
- All error handling uses `Result<T, String>` with `?` operator
- Transaction handling with proper error propagation

**Database Patterns (Story 1.2):**

- Use transactions for multi-row operations
- `queue_position` is i64 in Rust, number in TypeScript
- Status validation before modification

### Library & Framework Requirements

**Backend (Rust):**

| Dependency | Version | Usage in Story 1.4                   |
| ---------- | ------- | ------------------------------------ |
| sqlx       | 0.8     | Transaction support, query execution |
| tauri      | 2.x     | 5 new #[tauri::command] functions    |

**No new dependencies required.**

**Frontend (TypeScript/Svelte):**

| Dependency      | Version | Usage in Story 1.4    |
| --------------- | ------- | --------------------- |
| Svelte          | 5.0.0   | Runes, event handlers |
| @tauri-apps/api | 2.x     | invoke() API calls    |

**No new dependencies required** - native HTML5 drag-and-drop used instead of external library.

### File Structure Requirements

**New Files to Create:**

```
src/lib/ui/
└── ConfirmModal.svelte         # Reusable confirmation modal component
```

**Files to Modify:**

```
src-tauri/src/
├── db.rs                       # Add 5 new queue operation functions
├── commands.rs                 # Add 5 new Tauri commands
└── lib.rs                      # Register 5 new commands in handler

src/lib/
├── api.ts                      # Add 5 new API wrapper functions
└── features/queue/
    └── QueuePanel.svelte       # Add action buttons, drag-drop, modal integration
```

**Tauri Commands to Register in lib.rs:**

```rust
tauri::generate_handler![
    // ... existing commands ...
    remove_job_from_queue,
    move_job_to_front,
    move_job_to_end,
    reorder_queue_job,
    cancel_all_pending_jobs,
]
```

### Testing Requirements

**Backend Unit Tests (`src-tauri/src/db.rs`):**

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_remove_job_renumbers_positions() -> Result<(), Box<dyn std::error::Error>> {
        let pool = init_test_db().await?;

        // Insert 5 jobs with positions 1-5
        insert_job_with_queue(&pool, 1, "job1.py", 1, "2026-01-11T10:00:00Z").await?;
        insert_job_with_queue(&pool, 1, "job2.py", 2, "2026-01-11T10:01:00Z").await?;
        let job3_id = insert_job_with_queue(&pool, 1, "job3.py", 3, "2026-01-11T10:02:00Z").await?;
        insert_job_with_queue(&pool, 1, "job4.py", 4, "2026-01-11T10:03:00Z").await?;
        insert_job_with_queue(&pool, 1, "job5.py", 5, "2026-01-11T10:04:00Z").await?;

        // Remove job at position 3
        remove_job_from_queue(&pool, job3_id).await?;

        // Verify remaining jobs renumbered correctly
        let jobs = get_queued_jobs(&pool).await?;
        assert_eq!(jobs.len(), 4);
        assert_eq!(jobs[0].benchmark_name, "job1.py");
        assert_eq!(jobs[0].queue_position, Some(1));
        assert_eq!(jobs[1].benchmark_name, "job2.py");
        assert_eq!(jobs[1].queue_position, Some(2));
        assert_eq!(jobs[2].benchmark_name, "job4.py");
        assert_eq!(jobs[2].queue_position, Some(3)); // Was 4, now 3
        assert_eq!(jobs[3].benchmark_name, "job5.py");
        assert_eq!(jobs[3].queue_position, Some(4)); // Was 5, now 4

        Ok(())
    }

    #[tokio::test]
    async fn test_remove_running_job_fails() -> Result<(), Box<dyn std::error::Error>> {
        let pool = init_test_db().await?;

        let job_id = insert_job_with_queue(&pool, 1, "job1.py", 1, "2026-01-11T10:00:00Z").await?;
        update_job_status(&pool, job_id, "running").await?;

        let result = remove_job_from_queue(&pool, job_id).await;
        assert!(result.is_err());
        assert!(result.err().unwrap_or_default().contains("running"));

        Ok(())
    }

    #[tokio::test]
    async fn test_move_to_front_shifts_jobs() -> Result<(), Box<dyn std::error::Error>> {
        let pool = init_test_db().await?;

        insert_job_with_queue(&pool, 1, "job1.py", 1, "2026-01-11T10:00:00Z").await?;
        insert_job_with_queue(&pool, 1, "job2.py", 2, "2026-01-11T10:01:00Z").await?;
        let job3_id = insert_job_with_queue(&pool, 1, "job3.py", 3, "2026-01-11T10:02:00Z").await?;

        move_job_to_front(&pool, job3_id).await?;

        let jobs = get_queued_jobs(&pool).await?;
        assert_eq!(jobs[0].benchmark_name, "job3.py");
        assert_eq!(jobs[0].queue_position, Some(1));
        assert_eq!(jobs[1].benchmark_name, "job1.py");
        assert_eq!(jobs[1].queue_position, Some(2));
        assert_eq!(jobs[2].benchmark_name, "job2.py");
        assert_eq!(jobs[2].queue_position, Some(3));

        Ok(())
    }

    #[tokio::test]
    async fn test_move_to_end_shifts_jobs() -> Result<(), Box<dyn std::error::Error>> {
        let pool = init_test_db().await?;

        let job1_id = insert_job_with_queue(&pool, 1, "job1.py", 1, "2026-01-11T10:00:00Z").await?;
        insert_job_with_queue(&pool, 1, "job2.py", 2, "2026-01-11T10:01:00Z").await?;
        insert_job_with_queue(&pool, 1, "job3.py", 3, "2026-01-11T10:02:00Z").await?;

        move_job_to_end(&pool, job1_id).await?;

        let jobs = get_queued_jobs(&pool).await?;
        assert_eq!(jobs[0].benchmark_name, "job2.py");
        assert_eq!(jobs[0].queue_position, Some(1));
        assert_eq!(jobs[1].benchmark_name, "job3.py");
        assert_eq!(jobs[1].queue_position, Some(2));
        assert_eq!(jobs[2].benchmark_name, "job1.py");
        assert_eq!(jobs[2].queue_position, Some(3));

        Ok(())
    }

    #[tokio::test]
    async fn test_reorder_job_move_up() -> Result<(), Box<dyn std::error::Error>> {
        let pool = init_test_db().await?;

        insert_job_with_queue(&pool, 1, "job1.py", 1, "2026-01-11T10:00:00Z").await?;
        insert_job_with_queue(&pool, 1, "job2.py", 2, "2026-01-11T10:01:00Z").await?;
        insert_job_with_queue(&pool, 1, "job3.py", 3, "2026-01-11T10:02:00Z").await?;
        let job4_id = insert_job_with_queue(&pool, 1, "job4.py", 4, "2026-01-11T10:03:00Z").await?;

        // Move job4 from position 4 to position 2
        reorder_queue_job(&pool, job4_id, 2).await?;

        let jobs = get_queued_jobs(&pool).await?;
        assert_eq!(jobs[0].benchmark_name, "job1.py");
        assert_eq!(jobs[0].queue_position, Some(1));
        assert_eq!(jobs[1].benchmark_name, "job4.py"); // Moved here
        assert_eq!(jobs[1].queue_position, Some(2));
        assert_eq!(jobs[2].benchmark_name, "job2.py"); // Shifted
        assert_eq!(jobs[2].queue_position, Some(3));
        assert_eq!(jobs[3].benchmark_name, "job3.py"); // Shifted
        assert_eq!(jobs[3].queue_position, Some(4));

        Ok(())
    }

    #[tokio::test]
    async fn test_cancel_all_pending_preserves_running() -> Result<(), Box<dyn std::error::Error>> {
        let pool = init_test_db().await?;

        insert_job_with_queue(&pool, 1, "pending1.py", 1, "2026-01-11T10:00:00Z").await?;
        let running_id = insert_job_with_queue(&pool, 1, "running.py", 2, "2026-01-11T10:01:00Z").await?;
        insert_job_with_queue(&pool, 1, "pending2.py", 3, "2026-01-11T10:02:00Z").await?;

        update_job_status(&pool, running_id, "running").await?;

        let count = cancel_all_pending_jobs(&pool).await?;
        assert_eq!(count, 2); // Only pending jobs deleted

        let jobs = get_queued_jobs(&pool).await?;
        assert_eq!(jobs.len(), 1);
        assert_eq!(jobs[0].status, "running");

        Ok(())
    }
}
```

**Quality Checks (Pre-Commit):**

```bash
# Backend
cargo clippy               # Must pass with zero warnings
cargo fmt                  # Auto-format
cargo test                 # All tests pass (including 6 new queue management tests)

# Frontend
bun run quality            # ESLint + Prettier + svelte-check
```

### Project Structure Notes

**Alignment with Existing Architecture:**

- **QueuePanel Extension**: Story 1.4 adds interaction capabilities to the read-only display from Story 1.3
- **ConfirmModal**: New reusable component following existing `src/lib/ui/` pattern (Button, StatusBadge, etc.)
- **API Pattern**: 5 new functions follow existing pattern in api.ts

**Tauri IPC Commands (Story 1.4):**

- Alpha commands: 40 existing
- Story 1.2 added: `queue_benchmarks` (total 41)
- Story 1.3 added: `get_all_queue_jobs` (total 42)
- **Story 1.4 adds**: `remove_job_from_queue`, `move_job_to_front`, `move_job_to_end`, `reorder_queue_job`, `cancel_all_pending_jobs` (5 commands, total 47)

**Database Operations:**

All operations use existing `queue_position` column from Story 1.2 migration. No schema changes required.

### References

**Source Documents:**

- **Epics File**: `_bmad-output/planning-artifacts/epics.md#Story 1.4` (lines 1365-1425)
- **Project Context**: `_bmad-output/project-context.md` (Rust clippy rules, Svelte 5 patterns)
- **Story 1.3 Learnings**: `_bmad-output/implementation-artifacts/1-3-queue-panel-ui-view-queued-jobs.md` (transaction wrapping, accessibility fixes)
- **Architecture**: `_bmad-output/planning-artifacts/architecture.md` (Trust-Building Patterns, Performance Under Load)

**Critical Architecture Decisions:**

- [Source: epics.md#Story 1.4] Remove, reorder, move to front/end operations (lines 1365-1425)
- [Source: project-context.md] Rust error handling patterns (lines 63-75)
- [Source: project-context.md] Svelte 5 runes patterns (lines 85-102)

**FRs Fulfilled:**

- FR157: User can cancel all pending jobs in queue
- FR158: User can remove specific job from queue (before execution)
- FR164: User can reorder jobs in queue (drag-and-drop or priority numbers)
- FR165: User can move job to front of queue
- FR166: User can move job to end of queue

## Dev Agent Record

### Agent Model Used

Claude Opus 4.5 (claude-opus-4-5-20251101)

### Debug Log References

N/A - Implementation proceeded without major blockers

### Completion Notes List

**Implementation Summary (2026-01-11):**

- ✅ Backend: 5 queue management functions in db.rs with transaction support
- ✅ Backend: 5 Tauri commands in commands.rs registered in lib.rs
- ✅ Frontend: 5 API wrapper functions in api.ts
- ✅ Frontend: QueuePanel.svelte updated with action buttons, drag-drop, keyboard support
- ✅ Frontend: New ConfirmModal.svelte component for destructive actions
- ✅ Tests: 8 new unit tests for queue operations (all passing)
- ✅ Quality: cargo clippy passes with zero warnings
- ✅ Quality: bun run quality passes (lint, format, type-check)

**Key Implementation Decisions:**

- Used HTML5 native drag-and-drop (no external library) for reordering
- Transaction wrapping for all multi-row position updates (atomic operations)
- Status validation enforced at database layer (only pending jobs modifiable)
- Drop zone highlighting via ring-2 class for visual feedback
- Delete key keyboard handler for quick job removal

**Tests Added:**

1. test_remove_job_renumbers_positions
2. test_remove_running_job_fails
3. test_move_to_front_shifts_jobs
4. test_move_to_end_shifts_jobs
5. test_reorder_job_move_up
6. test_reorder_job_move_down
7. test_cancel_all_pending_preserves_running
8. test_status_validation_on_modify

**Deferred to Future Stories:**

- Duplicate detection warnings (Story 1.5)
- Queue filtering by status (Story 1.5)
- Polling for live updates (Epic 4)

### File List

**Files created:**

- `src/lib/ui/ConfirmModal.svelte` - Reusable confirmation modal component (80 lines)

**Files modified:**

**Backend:**

- `src-tauri/src/db.rs` - Added 5 queue operation functions + 8 unit tests (~250 lines)
- `src-tauri/src/commands.rs` - Added 5 Tauri commands (~70 lines)
- `src-tauri/src/lib.rs` - Registered 5 new commands (5 lines)

**Frontend:**

- `src/lib/api.ts` - Added 5 API wrapper functions (~45 lines)
- `src/lib/features/queue/QueuePanel.svelte` - Added action buttons, drag-drop, modal integration (~220 lines)

## Change Log

| Date       | Change                                                | Author          |
| ---------- | ----------------------------------------------------- | --------------- |
| 2026-01-11 | Implemented all 9 tasks for Story 1.4                 | Claude Opus 4.5 |
| 2026-01-11 | Added 5 backend queue management functions with tests | Claude Opus 4.5 |
| 2026-01-11 | Added frontend action buttons, drag-drop, modal       | Claude Opus 4.5 |
