// Queue state management with Svelte 5 runes (Story 2.5)
//
// Manages queue state (idle/running/paused) and provides actions
// to start, pause, and resume queue processing.
// Automatically polls queue status every 2 seconds.

import * as api from '$lib/api';
import type { QueueStatus } from '$lib/types';

interface QueueStore {
  state: 'idle' | 'running' | 'paused';
  currentJobId: number | null;
  pendingCount: number;
  runningCount: number;
  completedCount: number;
}

// Create reactive queue store using $state
const queueStore = $state<QueueStore>({
  state: 'idle',
  currentJobId: null,
  pendingCount: 0,
  runningCount: 0,
  completedCount: 0,
});

// Poll queue status every 2 seconds
$effect(() => {
  const interval = setInterval(async () => {
    try {
      const status: QueueStatus = await api.getQueueStatus();
      queueStore.state = status.state;
      queueStore.currentJobId = status.currentJobId;
      queueStore.pendingCount = status.pendingCount;
      queueStore.runningCount = status.runningCount;
      queueStore.completedCount = status.completedCount;
    } catch {
      // Silently ignore polling errors to avoid spamming logs
    }
  }, 2000);

  // Cleanup interval on destroy
  return () => clearInterval(interval);
});

// Actions

async function startQueue(): Promise<void> {
  await api.startQueueProcessing();
  // Toast notification handled by QueuePanel or backend event
}

async function pauseQueue(): Promise<void> {
  await api.pauseQueueProcessing();
  // Toast notification: "Queue paused - X jobs remaining"
}

async function resumeQueue(): Promise<void> {
  await api.resumeQueueProcessing();
  // Toast notification: "Queue resumed - processing X jobs"
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
  startQueue,
  pauseQueue,
  resumeQueue,
};
