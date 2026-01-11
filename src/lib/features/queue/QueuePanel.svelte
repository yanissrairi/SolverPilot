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
  let cancelAllLoading = $state(false);
  let draggedJobId = $state<number | null>(null);
  let dropTargetPosition = $state<number | null>(null);
  let operationInProgress = $state(false);

  // Group jobs by status for visual hierarchy (Task 5)
  let jobsByStatus = $derived.by((): { running: Job[]; pending: Job[]; completed: Job[] } => {
    const running: Job[] = jobs.filter(j => j.status === 'running');
    const pending: Job[] = jobs.filter(j => j.status === 'pending');
    const completed: Job[] = jobs.filter(
      j => j.status === 'completed' || j.status === 'failed' || j.status === 'killed',
    );
    return { running, pending, completed };
  });

  async function loadJobs() {
    try {
      jobs = await getAllQueueJobs();
    } catch (error) {
      const message = error instanceof Error ? error.message : String(error);
      toast.error(`Failed to load queue: ${message}`);
    }
  }

  onMount(() => {
    void loadJobs();
    // TODO Epic 4: Add polling every 2 seconds
    // const interval = setInterval(() => void loadJobs(), 2000);
    // return () => clearInterval(interval);
  });

  // Timestamp formatting logic (Task 6)
  function formatTimestamp(job: Job): string {
    if (job.status === 'pending' && job.queued_at !== null) {
      const queued = new Date(job.queued_at);
      const ago = Math.floor((Date.now() - queued.getTime()) / 60000);
      if (ago < 1) return 'Queued just now';
      if (ago < 60) return `Queued ${String(ago)}m ago`;
      const hours = Math.floor(ago / 60);
      if (hours < 24) return `Queued ${String(hours)}h ago`;
      const days = Math.floor(hours / 24);
      return `Queued ${String(days)}d ago`;
    }
    if (job.status === 'running' && job.started_at !== null) {
      // TODO Epic 4: Replace with live elapsed time counter
      const started = new Date(job.started_at);
      const elapsed = Math.floor((Date.now() - started.getTime()) / 60000);
      if (elapsed < 1) return 'Running for <1m';
      if (elapsed < 60) return `Running for ${String(elapsed)}m`;
      const hours = Math.floor(elapsed / 60);
      if (hours < 24) return `Running for ${String(hours)}h`;
      const days = Math.floor(hours / 24);
      return `Running for ${String(days)}d ${String(hours % 24)}h`;
    }
    if (
      (job.status === 'completed' || job.status === 'failed' || job.status === 'killed') &&
      job.finished_at !== null
    ) {
      const finished = new Date(job.finished_at);
      const ago = Math.floor((Date.now() - finished.getTime()) / 60000);
      if (ago < 1) return 'Finished just now';
      if (ago < 60) return `Finished ${String(ago)}m ago`;
      const hours = Math.floor(ago / 60);
      if (hours < 24) return `Finished ${String(hours)}h ago`;
      const days = Math.floor(hours / 24);
      return `Finished ${String(days)}d ago`;
    }
    return '';
  }

  // Story 1.4 - Job removal handler
  async function handleRemoveJob(jobId: number) {
    if (operationInProgress) return;
    try {
      operationInProgress = true;
      await removeJobFromQueue(jobId);
      toast.success('Job removed from queue');
      await loadJobs();
    } catch (error) {
      const message = error instanceof Error ? error.message : String(error);
      toast.error(message);
    } finally {
      operationInProgress = false;
    }
  }

  // Story 1.4 - Move to front handler
  async function handleMoveToFront(jobId: number) {
    if (operationInProgress) return;
    try {
      operationInProgress = true;
      await moveJobToFront(jobId);
      toast.success('Job moved to front of queue');
      await loadJobs();
    } catch (error) {
      const message = error instanceof Error ? error.message : String(error);
      toast.error(message);
    } finally {
      operationInProgress = false;
    }
  }

  // Story 1.4 - Move to end handler
  async function handleMoveToEnd(jobId: number) {
    if (operationInProgress) return;
    try {
      operationInProgress = true;
      await moveJobToEnd(jobId);
      toast.success('Job moved to end of queue');
      await loadJobs();
    } catch (error) {
      const message = error instanceof Error ? error.message : String(error);
      toast.error(message);
    } finally {
      operationInProgress = false;
    }
  }

  // Story 1.4 - Cancel all pending handler
  async function handleCancelAllPending() {
    try {
      cancelAllLoading = true;
      const count = await cancelAllPendingJobs();
      showCancelAllModal = false;
      toast.success(`Cancelled ${String(count)} pending jobs`);
      await loadJobs();
    } catch (error) {
      const message = error instanceof Error ? error.message : String(error);
      toast.error(message);
    } finally {
      cancelAllLoading = false;
    }
  }

  // Story 1.4 - Drag and drop handlers
  function handleDragStart(event: DragEvent, jobId: number) {
    draggedJobId = jobId;
    if (event.dataTransfer) {
      event.dataTransfer.effectAllowed = 'move';
      event.dataTransfer.setData('text/plain', String(jobId));
    }
    // Add visual feedback to dragged element
    const target = event.target as HTMLElement;
    target.classList.add('opacity-50');
  }

  function handleDragEnd(event: DragEvent) {
    draggedJobId = null;
    dropTargetPosition = null;
    // Remove visual feedback
    const target = event.target as HTMLElement;
    target.classList.remove('opacity-50');
  }

  function handleDragOver(event: DragEvent, position: number) {
    event.preventDefault();
    if (event.dataTransfer) {
      event.dataTransfer.dropEffect = 'move';
    }
    dropTargetPosition = position;
  }

  function handleDragLeave() {
    dropTargetPosition = null;
  }

  async function handleDrop(event: DragEvent, targetPosition: number) {
    event.preventDefault();
    if (draggedJobId === null || operationInProgress) return;

    try {
      operationInProgress = true;
      await reorderQueueJob(draggedJobId, targetPosition);
      await loadJobs();
    } catch (error) {
      const message = error instanceof Error ? error.message : String(error);
      toast.error(message);
    } finally {
      draggedJobId = null;
      dropTargetPosition = null;
      operationInProgress = false;
    }
  }

  // Story 1.4 - Keyboard handler for Delete key
  function handleKeyDown(event: KeyboardEvent, jobId: number, status: string) {
    if (event.key === 'Delete' && status === 'pending') {
      void handleRemoveJob(jobId);
    }
  }
</script>

<!-- Glassmorphism panel with backdrop-blur-sm (Task 1.5) -->
<div
  class="h-full flex flex-col bg-slate-900/75 backdrop-blur-sm rounded-xl border border-slate-700/50 shadow-2xl"
>
  <!-- Header with Cancel All button (Story 1.4) -->
  <div class="p-4 border-b border-slate-700/50 flex justify-between items-center">
    <div>
      <h2 class="text-lg font-semibold text-slate-200">Queue</h2>
      <p class="text-sm text-slate-400">{jobs.length} jobs</p>
    </div>
    {#if jobsByStatus.pending.length > 0}
      <button
        class="text-sm text-red-400 hover:text-red-300 px-3 py-1 rounded border border-red-500/30 hover:bg-red-500/10 transition-colors"
        onclick={() => {
          showCancelAllModal = true;
        }}
        aria-label="Cancel all pending jobs"
      >
        Cancel All
      </button>
    {/if}
  </div>

  <!-- Scrolling container with overflow-y-auto (Task 8.1) -->
  <div class="flex-1 overflow-y-auto">
    {#if jobs.length === 0}
      <!-- Empty state (Task 1.4) -->
      <div class="flex items-center justify-center h-full text-center p-8">
        <p class="text-slate-400">
          No jobs in queue. Select benchmarks and press Q to get started.
        </p>
      </div>
    {:else}
      <!-- Running jobs section (Task 5.2) -->
      {#if jobsByStatus.running.length > 0}
        <div class="p-2">
          <h3 class="text-sm font-semibold text-slate-400 uppercase tracking-wide px-3 py-2">
            Running ({jobsByStatus.running.length})
          </h3>
          {#each jobsByStatus.running as job (job.id)}
            <!-- Job item with py-2 spacing and hover effect (Task 8.2, 8.5) -->
            <div class="px-3 py-2 hover:bg-slate-700/50 rounded-lg transition-colors">
              <div class="flex items-center justify-between">
                <span class="font-semibold text-slate-100">{job.benchmark_name}</span>
                <StatusBadge status={job.status} />
              </div>
              <p class="text-sm text-slate-500 mt-1">{formatTimestamp(job)}</p>
            </div>
          {/each}
        </div>
      {/if}

      <!-- Pending jobs section (Task 5.3) with drag-drop (Story 1.4) -->
      {#if jobsByStatus.pending.length > 0}
        <div class="p-2">
          <h3 class="text-sm font-semibold text-slate-400 uppercase tracking-wide px-3 py-2">
            Pending ({jobsByStatus.pending.length})
          </h3>
          {#each jobsByStatus.pending as job, idx (job.id)}
            <!-- svelte-ignore a11y_no_noninteractive_tabindex a11y_no_noninteractive_element_interactions -->
            <!-- Alternating backgrounds with drag-drop support (Story 1.4) -->
            <div
              class="px-3 py-2 hover:bg-slate-700/50 rounded-lg transition-colors cursor-grab active:cursor-grabbing {idx %
                2 ===
              0
                ? 'bg-slate-800/30'
                : ''} {dropTargetPosition === job.queue_position
                ? 'ring-2 ring-blue-500/50 bg-blue-500/10'
                : ''}"
              draggable="true"
              ondragstart={e => handleDragStart(e, job.id)}
              ondragend={handleDragEnd}
              ondragover={e => {
                if (job.queue_position !== null) handleDragOver(e, job.queue_position);
              }}
              ondragleave={handleDragLeave}
              ondrop={e => {
                if (job.queue_position !== null) void handleDrop(e, job.queue_position);
              }}
              onkeydown={e => handleKeyDown(e, job.id, job.status)}
              tabindex="0"
              role="listitem"
            >
              <div class="flex items-center justify-between">
                <div class="flex items-center gap-2">
                  <!-- Queue position number (Task 5.3) -->
                  {#if job.queue_position !== null}
                    <span class="text-sm text-slate-400">#{String(job.queue_position)}</span>
                  {/if}
                  <span class="text-slate-200">{job.benchmark_name}</span>
                </div>
                <div class="flex items-center gap-1">
                  <StatusBadge status={job.status} />
                  <!-- Action buttons (Story 1.4) -->
                  <button
                    class="text-slate-400 hover:text-blue-400 p-1 rounded transition-colors"
                    onclick={() => {
                      void handleMoveToFront(job.id);
                    }}
                    title="Move to Front"
                    aria-label="Move job to front of queue"
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
                    class="text-slate-400 hover:text-blue-400 p-1 rounded transition-colors"
                    onclick={() => {
                      void handleMoveToEnd(job.id);
                    }}
                    title="Move to End"
                    aria-label="Move job to end of queue"
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
                    class="text-slate-400 hover:text-red-400 p-1 rounded transition-colors"
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
        </div>
      {/if}

      <!-- Completed/Failed jobs section (Task 5.4) -->
      {#if jobsByStatus.completed.length > 0}
        <div class="p-2">
          <h3 class="text-sm font-semibold text-slate-400 uppercase tracking-wide px-3 py-2">
            Completed ({jobsByStatus.completed.length})
          </h3>
          {#each jobsByStatus.completed as job (job.id)}
            <div class="px-3 py-2 hover:bg-slate-700/50 rounded-lg transition-colors">
              <div class="flex items-center justify-between">
                <!-- Muted text for completed jobs (Task 5.4) -->
                <span class="text-slate-400">{job.benchmark_name}</span>
                <StatusBadge status={job.status} />
              </div>
              <p class="text-sm text-slate-500 mt-1">{formatTimestamp(job)}</p>
            </div>
          {/each}
        </div>
      {/if}
    {/if}
  </div>
</div>

<!-- Cancel All Confirmation Modal (Story 1.4) -->
<ConfirmModal
  open={showCancelAllModal}
  title="Cancel All Pending Jobs?"
  message="This will remove all {jobsByStatus.pending
    .length} pending jobs from the queue. Running and completed jobs will not be affected."
  confirmText="Cancel All"
  cancelText="Keep Jobs"
  variant="danger"
  loading={cancelAllLoading}
  onConfirm={handleCancelAllPending}
  onCancel={() => {
    showCancelAllModal = false;
  }}
/>
