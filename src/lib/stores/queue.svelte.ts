// Queue state management with Svelte 5 runes (Story 2.5)
//
// Manages queue state (idle/running/paused) and provides actions
// to start, pause, and resume queue processing.
// Polling is initialized on first use via initPolling().

import * as api from '$lib/api';
import type { QueueStatus } from '$lib/types';

interface QueueStore {
  state: 'idle' | 'running' | 'paused';
  currentJobId: number | null;
  pendingCount: number;
  runningCount: number;
  completedCount: number;
  /** True for one polling cycle when queue naturally completes (all jobs done) */
  justCompleted: boolean;
}

// Track previous state to detect completion
let previousState: 'idle' | 'running' | 'paused' = 'idle';
let hadPendingJobs = false;
let pollingInterval: ReturnType<typeof setInterval> | null = null;

// Create reactive queue store using $state
const queueStore = $state<QueueStore>({
  state: 'idle',
  currentJobId: null,
  pendingCount: 0,
  runningCount: 0,
  completedCount: 0,
  justCompleted: false,
});

// Polling function - called from QueuePanel's onMount
async function pollQueueStatus() {
  try {
    const status: QueueStatus = await api.getQueueStatus();

    // Detect queue completion: was running/paused with jobs, now idle with none
    const wasActive = previousState === 'running' || previousState === 'paused';
    const nowIdle = status.state === 'idle';
    const queueEmpty = status.pendingCount === 0 && status.runningCount === 0;
    const completed = wasActive && hadPendingJobs && nowIdle && queueEmpty;

    // Update store
    queueStore.state = status.state;
    queueStore.currentJobId = status.currentJobId;
    queueStore.pendingCount = status.pendingCount;
    queueStore.runningCount = status.runningCount;
    queueStore.completedCount = status.completedCount;
    queueStore.justCompleted = completed;

    // Track for next poll
    previousState = status.state;
    hadPendingJobs = status.pendingCount > 0 || status.runningCount > 0;
  } catch {
    // Silently ignore polling errors to avoid spamming logs
  }
}

// Initialize polling - should be called from a component's onMount
function initPolling(): () => void {
  if (pollingInterval !== null) {
    return () => {
      /* noop - already polling */
    };
  }

  // Poll immediately on init
  void pollQueueStatus();

  // Then poll every 2 seconds
  pollingInterval = setInterval(() => {
    void pollQueueStatus();
  }, 2000);

  // Return cleanup function
  return () => {
    if (pollingInterval !== null) {
      clearInterval(pollingInterval);
      pollingInterval = null;
    }
  };
}

// Actions

async function startQueue(): Promise<void> {
  await api.startQueueProcessing();
  // Immediately poll to update state
  await pollQueueStatus();
}

async function pauseQueue(): Promise<void> {
  await api.pauseQueueProcessing();
  await pollQueueStatus();
}

async function resumeQueue(): Promise<void> {
  await api.resumeQueueProcessing();
  await pollQueueStatus();
}

// Export reactive queue store with getters and actions
export const queue = {
  get state() {
    return queueStore.state;
  },
  get currentJobId() {
    return queueStore.currentJobId;
  },
  get pendingCount() {
    return queueStore.pendingCount;
  },
  get runningCount() {
    return queueStore.runningCount;
  },
  get completedCount() {
    return queueStore.completedCount;
  },
  /** True for one polling cycle when queue naturally completes */
  get justCompleted() {
    return queueStore.justCompleted;
  },
  startQueue,
  pauseQueue,
  resumeQueue,
  initPolling,
};
