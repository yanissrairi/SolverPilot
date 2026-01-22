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
  import type { Job, QueueFilter } from '$lib/types';
  import StatusBadge from '$lib/ui/StatusBadge.svelte';
  import ConfirmModal from '$lib/ui/ConfirmModal.svelte';
  import { toast } from '$lib/stores/toast.svelte';
  import { queue } from '$lib/stores/queue.svelte';

  let jobs = $state<Job[]>([]);
  let showCancelAllModal = $state(false);
  let cancelAllLoading = $state(false);
  let draggedJobId = $state<number | null>(null);
  let dropTargetPosition = $state<number | null>(null);
  let operationInProgress = $state(false);

  // Story 1.5 - Queue filtering
  let activeFilter = $state<QueueFilter>('all');
  let showFilterDropdown = $state(false);

  // Story 1.5 - Filtered jobs based on activeFilter
  // Note: 'completed' filter includes only 'completed' status, not 'failed' or 'killed'
  let filteredJobs = $derived(() => {
    if (activeFilter === 'all') {
      return jobs;
    }
    return jobs.filter(job => job.status === activeFilter);
  });

  // Story 1.5 - Filter label for header
  let filterLabel = $derived(() => {
    if (activeFilter === 'all') {
      return `${jobs.length.toString()} jobs`;
    }
    const count = filteredJobs().length;
    return `${count.toString()} ${activeFilter}`;
  });

  // Group jobs by status for visual hierarchy (Task 5)
  // Uses filteredJobs instead of jobs for filtering support
  let jobsByStatus = $derived.by((): { running: Job[]; pending: Job[]; completed: Job[] } => {
    const filtered = filteredJobs();
    const running: Job[] = filtered.filter(j => j.status === 'running');
    const pending: Job[] = filtered.filter(j => j.status === 'pending');
    const completed: Job[] = filtered.filter(
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

  // Story 1.5 - Set filter and persist to localStorage
  function setFilter(filter: QueueFilter) {
    activeFilter = filter;
    showFilterDropdown = false;
    localStorage.setItem('solverpilot_queue_filter', filter);
  }

  // Story 1.5 - Filter options (includes 'killed' for completeness)
  const filterOptions: { value: QueueFilter; label: string }[] = [
    { value: 'all', label: 'All' },
    { value: 'pending', label: 'Pending' },
    { value: 'running', label: 'Running' },
    { value: 'completed', label: 'Completed' },
    { value: 'failed', label: 'Failed' },
    { value: 'killed', label: 'Killed' },
  ];

  // Story 2.5 - Queue control button state
  let queueControlLoading = $state(false);

  // Story 2.5 - Derived button label based on queue state
  let queueButtonLabel = $derived(() => {
    if (queue.state === 'idle') return 'Start Queue';
    if (queue.state === 'running') return 'Pause Queue';
    return 'Resume Queue'; // paused
  });

  // Story 2.5 - Button variant based on queue state
  let queueButtonVariant = $derived(() => {
    if (queue.state === 'running') return 'warning';
    return 'primary';
  });

  // Story 2.5 - Button disabled when no pending jobs
  let queueButtonDisabled = $derived(() => {
    return queue.pendingCount === 0 && queue.state === 'idle';
  });

  // Story 2.5 - Button tooltip
  let queueButtonTooltip = $derived(() => {
    if (queue.pendingCount === 0 && queue.state === 'idle') {
      return 'No pending jobs to execute';
    }
    if (queue.state === 'idle') {
      return 'Jobs run on remote server - safe to close app';
    }
    if (queue.state === 'running') {
      return 'Pause queue - running jobs will complete';
    }
    return 'Resume queue processing'; // paused
  });

  // Story 2.5 - Queue status summary for header
  let queueStatusSummary = $derived(() => {
    return `${String(queue.runningCount)} running • ${String(queue.pendingCount)} pending • ${String(queue.completedCount)} completed`;
  });

  // Story 2.5 - Handle queue control button click
  async function handleQueueControl() {
    if (queueControlLoading) return;

    try {
      queueControlLoading = true;

      if (queue.state === 'idle') {
        await queue.startQueue();
        toast.success(`Queue started - ${String(queue.pendingCount)} jobs executing`);
      } else if (queue.state === 'running') {
        await queue.pauseQueue();
        toast.info(`Queue paused - ${String(queue.pendingCount)} jobs remaining`);
      } else {
        // paused
        await queue.resumeQueue();
        toast.success(`Queue resumed - processing ${String(queue.pendingCount)} jobs`);
      }
    } catch (error) {
      const message = error instanceof Error ? error.message : String(error);
      toast.error(`Failed to control queue: ${message}`);
    } finally {
      queueControlLoading = false;
    }
  }

  // Story 2.5 - Keyboard shortcut handler for 'S' key
  $effect(() => {
    function handleKeyPress(event: KeyboardEvent) {
      // Only trigger if not in an input field
      if (event.target instanceof HTMLInputElement || event.target instanceof HTMLTextAreaElement) {
        return;
      }

      if (
        (event.key === 's' || event.key === 'S') &&
        !queueButtonDisabled() &&
        !queueControlLoading
      ) {
        event.preventDefault();
        void handleQueueControl();
      }
    }

    window.addEventListener('keydown', handleKeyPress);

    return () => {
      window.removeEventListener('keydown', handleKeyPress);
    };
  });

  // Story 2.5 - Queue completion toast notification
  $effect(() => {
    if (queue.justCompleted) {
      toast.success('Queue completed - all jobs finished');
    }
  });

  onMount(() => {
    // Story 1.5 - Load filter preference from localStorage
    const savedFilter = localStorage.getItem('solverpilot_queue_filter');
    if (savedFilter !== null) {
      activeFilter = savedFilter as QueueFilter;
    }

    void loadJobs();

    // Initialize queue status polling (Story 2.5)
    const stopPolling = queue.initPolling();

    // Poll jobs list every 2 seconds as well
    const jobsInterval = setInterval(() => void loadJobs(), 2000);

    return () => {
      stopPolling();
      clearInterval(jobsInterval);
    };
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
  <!-- Header with Filter, Queue Controls, and Cancel All (Story 1.4 + 1.5 + 2.5) -->
  <div class="p-4 border-b border-slate-700/50 flex justify-between items-center">
    <div class="flex items-center gap-3">
      <h2 class="text-lg font-semibold text-slate-200">Queue</h2>

      <!-- Story 2.5 - Queue status summary -->
      <span class="text-xs text-slate-400">{queueStatusSummary()}</span>

      <!-- Filter Dropdown (Story 1.5) -->
      <div class="relative">
        <button
          class="text-sm px-3 py-1 rounded border border-slate-600 bg-slate-800/50 text-slate-300 hover:bg-slate-700/50 transition-colors flex items-center gap-2"
          onclick={() => {
            showFilterDropdown = !showFilterDropdown;
          }}
          aria-label="Filter queue by status"
        >
          <svg class="w-4 h-4" fill="none" viewBox="0 0 24 24" stroke="currentColor">
            <path
              stroke-linecap="round"
              stroke-linejoin="round"
              stroke-width="2"
              d="M3 4a1 1 0 011-1h16a1 1 0 011 1v2.586a1 1 0 01-.293.707l-6.414 6.414a1 1 0 00-.293.707V17l-4 4v-6.586a1 1 0 00-.293-.707L3.293 7.293A1 1 0 013 6.586V4z"
            />
          </svg>
          <span>{filterLabel()}</span>
        </button>

        {#if showFilterDropdown}
          <div
            class="absolute top-full left-0 mt-1 bg-slate-800 border border-slate-700 rounded-lg shadow-xl z-10 py-1 min-w-[140px]"
            onclick={e => e.stopPropagation()}
            role="menu"
          >
            {#each filterOptions as option (option.value)}
              <button
                class="w-full px-4 py-2 text-sm text-left hover:bg-slate-700/50 transition-colors {activeFilter ===
                option.value
                  ? 'bg-slate-700/30 text-blue-400'
                  : 'text-slate-300'}"
                onclick={() => setFilter(option.value)}
                role="menuitem"
              >
                {option.label}
                {#if activeFilter === option.value}
                  <span class="float-right">✓</span>
                {/if}
              </button>
            {/each}
          </div>
        {/if}
      </div>
    </div>

    <!-- Story 2.5 - Queue control buttons -->
    <div class="flex items-center gap-2">
      <!-- Start/Pause/Resume button -->
      <button
        class="px-4 py-2 rounded font-medium transition-colors disabled:opacity-50 disabled:cursor-not-allowed {queueButtonVariant() ===
        'warning'
          ? 'bg-yellow-600 hover:bg-yellow-700 text-white'
          : 'bg-blue-600 hover:bg-blue-700 text-white'}"
        onclick={handleQueueControl}
        disabled={queueButtonDisabled() || queueControlLoading}
        title={queueButtonTooltip()}
        aria-label={queueButtonLabel()}
      >
        {#if queueControlLoading}
          <span
            class="inline-block w-4 h-4 border-2 border-white border-t-transparent rounded-full animate-spin"
          ></span>
        {:else}
          {queueButtonLabel()}
          <span class="ml-1 text-xs opacity-75">(S)</span>
        {/if}
      </button>

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
              <!-- Story 1.5 AC8: Show error message snippet for failed jobs -->
              {#if (job.status === 'failed' || job.status === 'killed') && job.error_message}
                <p class="text-sm text-red-400/80 mt-1 truncate" title={job.error_message}>
                  {job.error_message.length > 80
                    ? job.error_message.substring(0, 80) + '...'
                    : job.error_message}
                </p>
              {/if}
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

<!-- Close dropdown when clicking outside (Story 1.5) -->
{#if showFilterDropdown}
  <div
    class="fixed inset-0 z-0"
    onclick={() => {
      showFilterDropdown = false;
    }}
    onkeydown={e => {
      if (e.key === 'Escape') showFilterDropdown = false;
    }}
    role="presentation"
  ></div>
{/if}
