<script lang="ts">
  import { onMount, onDestroy } from 'svelte';
  import { SvelteSet } from 'svelte/reactivity';
  import {
    loadConfig,
    initSsh,
    closeSsh,
    checkSshKeyStatus,
    addSshKey,
    checkSyncStatus,
    syncCode,
    scanBenchmarks,
    queueJobs,
    startNextJob,
    stopJob,
    killJob,
    getJobStatus,
    loadHistory,
  } from './lib/api';
  import type { Benchmark, Job, JobStatusResponse, SyncStatus, SshKeyStatus } from './lib/types';

  // --- STATE ---
  let benchmarks = $state<Benchmark[]>([]);
  const selectedBenchmarks = new SvelteSet<string>();
  let history = $state<Job[]>([]);

  // Status
  let sshReady = $state(false);
  let syncStatus = $state<SyncStatus>({ type: 'Checking' });
  let isSyncing = $state(false);

  // SSH Key / Passphrase Modal
  let showPassphraseModal = $state(false);
  let sshKeyStatus = $state<SshKeyStatus | null>(null);
  let passphrase = $state('');
  let passphraseError = $state('');
  let isAddingKey = $state(false);

  // Active Job
  let currentJobStatus = $state<JobStatusResponse | null>(null);
  let isRunning = $state(false);

  // History view
  let selectedHistoryJob = $state<Job | null>(null);

  // Logs
  let logsContainer = $state<HTMLElement>();
  let autoScroll = $state(true);

  // Polling
  let pollInterval: number | null = null;

  // --- COMPUTED / DERIVED ---
  const progressPercent = $derived(
    currentJobStatus
      ? (currentJobStatus.progress / (currentJobStatus.job?.progress_total ?? 1)) * 100
      : 0,
  );

  const formattedTime = $derived((seconds: number) => {
    const m = Math.floor(seconds / 60);
    const s = Math.floor(seconds % 60);
    return `${m.toString().padStart(2, '0')}:${s.toString().padStart(2, '0')}`;
  });

  // --- ACTIONS ---

  async function init() {
    try {
      // 1. Charger la config
      await loadConfig();

      // 2. Vérifier le statut de la clé SSH
      sshKeyStatus = await checkSshKeyStatus();

      if (sshKeyStatus.type === 'NeedsPassphrase') {
        // Afficher le modal pour entrer la passphrase
        showPassphraseModal = true;
        return; // On attend que l'utilisateur entre la passphrase
      } else if (sshKeyStatus.type === 'NoKey') {
        passphraseError = `Clé SSH non trouvée: ${sshKeyStatus.data.expected_path}`;
        showPassphraseModal = true;
        return;
      } else if (sshKeyStatus.type === 'NoAgent') {
        passphraseError = "Agent SSH non démarré. Lancez 'eval $(ssh-agent)' dans un terminal.";
        showPassphraseModal = true;
        return;
      }

      // 3. Clé déjà dans l'agent, initialiser SSH
      await completeInit();
    } catch {
      sshReady = false;
    }
  }

  async function completeInit() {
    try {
      await initSsh();
      sshReady = true;
      showPassphraseModal = false;
      void refreshSync();
      void refreshBenchmarks();
      void refreshHistory();
      startPolling();
    } catch (e) {
      sshReady = false;
      passphraseError = e instanceof Error ? e.message : String(e);
    }
  }

  async function submitPassphrase() {
    if (!passphrase.trim()) {
      passphraseError = 'Veuillez entrer la passphrase';
      return;
    }

    isAddingKey = true;
    passphraseError = '';

    try {
      await addSshKey(passphrase);
      passphrase = ''; // Clear for security
      await completeInit();
    } catch (e) {
      passphraseError = String(e);
    } finally {
      isAddingKey = false;
    }
  }

  async function refreshSync() {
    if (isSyncing) return;
    try {
      syncStatus = await checkSyncStatus();
    } catch (e) {
      syncStatus = { type: 'Error', data: { message: String(e) } };
    }
  }

  async function performSync() {
    isSyncing = true;
    syncStatus = { type: 'Syncing' };
    try {
      await syncCode();
      await refreshSync();
    } catch (e) {
      syncStatus = { type: 'Error', data: { message: String(e) } };
    } finally {
      isSyncing = false;
    }
  }

  async function refreshBenchmarks() {
    benchmarks = await scanBenchmarks();
  }

  async function refreshHistory() {
    history = await loadHistory(5);
  }

  // Selection Logic
  function toggleBenchmark(name: string) {
    if (selectedBenchmarks.has(name)) {
      selectedBenchmarks.delete(name);
    } else {
      selectedBenchmarks.add(name);
    }
    // SvelteSet is already reactive, no need to reassign
  }

  function toggleAll() {
    if (selectedBenchmarks.size === benchmarks.length) {
      selectedBenchmarks.clear();
    } else {
      selectedBenchmarks.clear();
      for (const b of benchmarks) {
        selectedBenchmarks.add(b.name);
      }
    }
  }

  // Job Control
  async function runSelected() {
    if (selectedBenchmarks.size === 0) return;
    try {
      await queueJobs(Array.from(selectedBenchmarks));
      await startNextJob();
      startPolling(); // Ensure polling is active
    } catch {
      // Job start failed silently - status will be shown in UI
    }
  }

  async function handleStop() {
    await stopJob();
  }

  async function handleKill() {
    await killJob();
  }

  // Polling Logic
  function startPolling() {
    if (pollInterval !== null) return;
    pollInterval = setInterval(async () => {
      try {
        const status = await getJobStatus();
        currentJobStatus = status;

        // Check if a job is actively running or if we are processing the queue
        if (status.job && (status.job.status === 'running' || status.job.status === 'pending')) {
          isRunning = true;
        } else if (status.is_finished && isRunning) {
          // Job just finished
          isRunning = false;
          void refreshHistory();
          // Try to start next job automatically if one finished
          const next = await startNextJob();
          if (next) isRunning = true;
        } else {
          isRunning = false;
        }

        // Auto Scroll Logs
        if (logsContainer && autoScroll) {
          logsContainer.scrollTop = logsContainer.scrollHeight;
        }
      } catch {
        // Poll error - will retry on next interval
      }
    }, 1000); // 1s polling
  }

  // Lifecycle
  onMount(() => {
    void init();
  });

  onDestroy(() => {
    if (pollInterval !== null) clearInterval(pollInterval);
    void closeSsh();
  });
</script>

<div class="h-screen flex flex-col p-4 md:p-6 gap-6 max-w-[1600px] mx-auto">
  <!-- HEADER -->
  <header
    class="glass-header rounded-2xl p-4 flex flex-col md:flex-row items-center justify-between gap-4 shrink-0"
  >
    <div class="flex items-center gap-3">
      <div class="p-2 bg-blue-500/20 rounded-lg">
        <svg
          xmlns="http://www.w3.org/2000/svg"
          class="w-6 h-6 text-blue-400"
          viewBox="0 0 24 24"
          fill="none"
          stroke="currentColor"
          stroke-width="2"
          stroke-linecap="round"
          stroke-linejoin="round"
          ><polyline points="4 17 10 11 4 5"></polyline><line x1="12" y1="19" x2="20" y2="19"
          ></line></svg
        >
      </div>
      <h1 class="text-xl font-bold tracking-tight text-white">Benchmark Runner</h1>
    </div>

    <div class="flex items-center gap-4 text-sm font-medium">
      <!-- SSH Status -->
      <div
        class="flex items-center gap-2 px-3 py-1.5 rounded-full bg-slate-900/50 border border-white/5"
      >
        <span class="text-slate-400">SSH</span>
        {#if sshReady}
          <span class="flex items-center gap-1.5 text-emerald-400">
            <span class="relative flex h-2.5 w-2.5">
              <span
                class="animate-ping absolute inline-flex h-full w-full rounded-full bg-emerald-400 opacity-75"
              ></span>
              <span class="relative inline-flex rounded-full h-2.5 w-2.5 bg-emerald-500"></span>
            </span>
            Connected
          </span>
        {:else}
          <span class="flex items-center gap-1.5 text-red-400">
            <span class="h-2.5 w-2.5 rounded-full bg-red-500"></span>
            Disconnected
          </span>
        {/if}
      </div>

      <!-- Sync Status -->
      <div
        class="flex items-center gap-3 px-3 py-1.5 rounded-full bg-slate-900/50 border border-white/5"
      >
        <span class="text-slate-400">Sync</span>

        {#if syncStatus.type === 'Checking'}
          <span class="text-yellow-400 animate-pulse">Checking...</span>
        {:else if syncStatus.type === 'UpToDate'}
          <span class="text-emerald-400 flex items-center gap-1"
            ><svg
              class="w-4 h-4"
              viewBox="0 0 24 24"
              fill="none"
              stroke="currentColor"
              stroke-width="2"><polyline points="20 6 9 17 4 12"></polyline></svg
            > Up to date</span
          >
        {:else if syncStatus.type === 'Modified'}
          <span class="text-amber-400">{syncStatus.data.count} changes</span>
          <button
            onclick={performSync}
            disabled={isSyncing}
            class="text-xs bg-amber-500/20 hover:bg-amber-500/30 text-amber-300 px-2 py-0.5 rounded transition-colors uppercase tracking-wider"
          >
            Sync
          </button>
        {:else if syncStatus.type === 'Syncing'}
          <span class="text-blue-400 animate-pulse">Syncing...</span>
        {:else if syncStatus.type === 'Error'}
          <span class="text-red-400" title={syncStatus.data.message}>Error</span>
        {/if}

        {#if syncStatus.type !== 'Modified' && syncStatus.type !== 'Syncing' && syncStatus.type !== 'Checking'}
          <button
            onclick={refreshSync}
            class="opacity-50 hover:opacity-100 transition-opacity"
            title="Check Sync"
          >
            <svg
              class="w-4 h-4"
              viewBox="0 0 24 24"
              fill="none"
              stroke="currentColor"
              stroke-width="2"
              ><path
                d="M21.5 2v6h-6M2.5 22v-6h6M2 11.5a10 10 0 0 1 18.8-4.3M22 12.5a10 10 0 0 1-18.8 4.3"
              /></svg
            >
          </button>
        {/if}
      </div>
    </div>
  </header>

  <!-- MAIN CONTENT GRID -->
  <div class="flex-1 grid grid-cols-1 lg:grid-cols-12 gap-6 min-h-0">
    <!-- LEFT: Benchmarks List -->
    <div class="lg:col-span-4 flex flex-col gap-4 min-h-0">
      <div class="glass-panel flex-1 flex flex-col min-h-0">
        <div class="p-4 border-b border-white/5 flex items-center justify-between bg-slate-800/30">
          <h2 class="font-semibold text-white">Benchmarks</h2>
          <div class="flex gap-2">
            <button
              onclick={refreshBenchmarks}
              aria-label="Refresh Benchmarks"
              class="p-1.5 rounded hover:bg-white/5 text-slate-400 hover:text-white transition-colors"
            >
              <svg
                class="w-4 h-4"
                viewBox="0 0 24 24"
                fill="none"
                stroke="currentColor"
                stroke-width="2"
                ><path d="M21 12a9 9 0 1 1-9-9c2.52 0 4.93 1 6.74 2.74L21 8" /><path
                  d="M21 3v5h-5"
                /></svg
              >
            </button>
          </div>
        </div>

        <div class="p-3 border-b border-white/5 bg-slate-800/20">
          <button
            onclick={toggleAll}
            class="text-xs font-medium text-slate-400 hover:text-white uppercase tracking-wider flex items-center gap-2"
          >
            <div
              class={`w-4 h-4 rounded border ${selectedBenchmarks.size === benchmarks.length && benchmarks.length > 0 ? 'bg-blue-500 border-blue-500' : 'border-slate-600'} flex items-center justify-center transition-colors`}
            >
              {#if selectedBenchmarks.size === benchmarks.length && benchmarks.length > 0}
                <svg
                  class="w-3 h-3 text-white"
                  viewBox="0 0 24 24"
                  fill="none"
                  stroke="currentColor"
                  stroke-width="3"><polyline points="20 6 9 17 4 12"></polyline></svg
                >
              {/if}
            </div>
            {selectedBenchmarks.size === benchmarks.length ? 'Deselect All' : 'Select All'}
          </button>
        </div>

        <div class="overflow-y-auto flex-1 p-2 space-y-1 custom-scrollbar">
          {#each benchmarks as bench (bench.name)}
            <!-- svelte-ignore a11y_click_events_have_key_events -->
            <!-- svelte-ignore a11y_no_static_element_interactions -->
            <div
              class={`flex items-center gap-3 p-3 rounded-lg cursor-pointer transition-all border ${selectedBenchmarks.has(bench.name) ? 'bg-blue-500/10 border-blue-500/50' : 'hover:bg-white/5 border-transparent'}`}
              onclick={() => {
                toggleBenchmark(bench.name);
              }}
            >
              <div
                class={`w-4 h-4 rounded border flex items-center justify-center transition-colors ${selectedBenchmarks.has(bench.name) ? 'bg-blue-500 border-blue-500' : 'border-slate-600'}`}
              >
                {#if selectedBenchmarks.has(bench.name)}
                  <svg
                    class="w-3 h-3 text-white"
                    viewBox="0 0 24 24"
                    fill="none"
                    stroke="currentColor"
                    stroke-width="3"><polyline points="20 6 9 17 4 12"></polyline></svg
                  >
                {/if}
              </div>
              <span
                class={`text-sm ${selectedBenchmarks.has(bench.name) ? 'text-white font-medium' : 'text-slate-300'}`}
                >{bench.name}</span
              >
            </div>
          {/each}
        </div>

        <div class="p-4 border-t border-white/5 bg-slate-800/30">
          <button
            onclick={runSelected}
            disabled={selectedBenchmarks.size === 0 || isRunning}
            class="w-full btn-glass btn-primary flex items-center justify-center gap-2 disabled:grayscale disabled:opacity-50"
          >
            {#if isRunning}
              <svg
                class="animate-spin w-4 h-4"
                viewBox="0 0 24 24"
                fill="none"
                stroke="currentColor"
                stroke-width="2"><path d="M21 12a9 9 0 1 1-6.219-8.56" /></svg
              >
              Running...
            {:else}
              <svg
                class="w-4 h-4"
                viewBox="0 0 24 24"
                fill="none"
                stroke="currentColor"
                stroke-width="2"><polygon points="5 3 19 12 5 21 5 3"></polygon></svg
              >
              Run {selectedBenchmarks.size} Benchmarks
            {/if}
          </button>
        </div>
      </div>
    </div>

    <!-- RIGHT: Active Job & Status -->
    <div class="lg:col-span-8 flex flex-col gap-6 min-h-0">
      <!-- Monitor Panel -->
      <div class="glass-panel flex-1 flex flex-col relative min-h-0">
        {#if selectedHistoryJob}
          <!-- Viewing History Job Logs -->
          <div
            class="p-4 border-b border-white/5 bg-slate-800/30 flex items-center justify-between"
          >
            <div class="flex items-center gap-4">
              <button
                onclick={() => (selectedHistoryJob = null)}
                class="p-1.5 rounded hover:bg-white/10 text-slate-400 hover:text-white transition-colors"
                title="Back to live view"
              >
                <svg
                  class="w-5 h-5"
                  viewBox="0 0 24 24"
                  fill="none"
                  stroke="currentColor"
                  stroke-width="2"><path d="M19 12H5M12 19l-7-7 7-7" /></svg
                >
              </button>
              <div
                class={`w-2 h-2 rounded-full ${
                  selectedHistoryJob.status === 'completed'
                    ? 'bg-emerald-500'
                    : selectedHistoryJob.status === 'failed'
                      ? 'bg-red-500'
                      : selectedHistoryJob.status === 'killed'
                        ? 'bg-slate-500'
                        : 'bg-blue-500'
                }`}
              ></div>
              <div>
                <h3 class="font-semibold text-white">{selectedHistoryJob.benchmark_name}</h3>
                <p class="text-xs text-slate-400">
                  ID: #{selectedHistoryJob.id} •
                  <span
                    class={selectedHistoryJob.status === 'completed'
                      ? 'text-emerald-400'
                      : selectedHistoryJob.status === 'failed'
                        ? 'text-red-400'
                        : selectedHistoryJob.status === 'killed'
                          ? 'text-slate-400'
                          : 'text-blue-400'}>{selectedHistoryJob.status}</span
                  >
                </p>
              </div>
            </div>
            <div class="flex items-center gap-3">
              {#if selectedHistoryJob.started_at !== null && selectedHistoryJob.finished_at !== null}
                <span class="font-mono text-sm text-slate-400 bg-slate-800/50 px-2 py-1 rounded">
                  {formattedTime(
                    (new Date(selectedHistoryJob.finished_at).getTime() -
                      new Date(selectedHistoryJob.started_at).getTime()) /
                      1000,
                  )}
                </span>
              {/if}
              <span class="text-xs text-slate-500 bg-slate-800/30 px-2 py-1 rounded">
                History
              </span>
            </div>
          </div>

          <!-- Terminal/Logs for history -->
          <div
            class="flex-1 bg-black/40 p-4 font-mono text-xs md:text-sm text-slate-300 overflow-y-auto custom-scrollbar"
          >
            <pre class="whitespace-pre-wrap">{selectedHistoryJob.log_content ||
                'No logs available for this job.'}</pre>
          </div>
        {:else if !currentJobStatus?.job}
          <div
            class="absolute inset-0 flex flex-col items-center justify-center text-slate-500 gap-4"
          >
            <div class="p-6 rounded-full bg-slate-800/50 border border-white/5">
              <svg
                class="w-12 h-12 opacity-50"
                viewBox="0 0 24 24"
                fill="none"
                stroke="currentColor"
                stroke-width="1"
                ><rect x="2" y="3" width="20" height="14" rx="2" ry="2"></rect><line
                  x1="8"
                  y1="21"
                  x2="16"
                  y2="21"
                ></line><line x1="12" y1="17" x2="12" y2="21"></line></svg
              >
            </div>
            <p>Ready to launch</p>
            <p class="text-xs text-slate-600">Click on a history entry to view its logs</p>
          </div>
        {:else}
          <div
            class="p-4 border-b border-white/5 bg-slate-800/30 flex items-center justify-between"
          >
            <div class="flex items-center gap-4">
              <div
                class={`w-2 h-2 rounded-full ${isRunning ? 'bg-emerald-500 animate-pulse' : 'bg-slate-500'}`}
              ></div>
              <div>
                <h3 class="font-semibold text-white">{currentJobStatus.job.benchmark_name}</h3>
                <p class="text-xs text-slate-400">
                  ID: #{currentJobStatus.job.id} • {currentJobStatus.progress_text}
                </p>
              </div>
            </div>
            <div class="flex items-center gap-3">
              <span class="font-mono text-sm text-blue-300 bg-blue-900/30 px-2 py-1 rounded">
                {formattedTime(currentJobStatus.elapsed_seconds)}
              </span>
              {#if isRunning}
                <button
                  onclick={handleStop}
                  class="btn-glass hover:bg-yellow-500/20 text-yellow-300 border-yellow-500/30 text-xs py-1.5"
                  >Stop</button
                >
                <button onclick={handleKill} class="btn-glass btn-danger text-xs py-1.5"
                  >Kill</button
                >
              {/if}
            </div>
          </div>

          <!-- Progress Bar -->
          <div class="h-1 bg-slate-800 w-full">
            <div
              class="h-full bg-blue-500 transition-all duration-500 ease-out"
              style={`width: ${String(progressPercent)}%`}
            ></div>
          </div>

          <!-- Terminal/Logs -->
          <div
            class="flex-1 bg-black/40 p-4 font-mono text-xs md:text-sm text-slate-300 overflow-y-auto custom-scrollbar relative group"
            bind:this={logsContainer}
          >
            <pre class="whitespace-pre-wrap">{currentJobStatus.logs ||
                'Waiting for output...'}</pre>

            <!-- Auto-scroll toggle overlay -->
            <button
              onclick={() => (autoScroll = !autoScroll)}
              class={`absolute bottom-4 right-4 p-2 rounded-lg backdrop-blur border transition-all ${autoScroll ? 'bg-blue-500/20 text-blue-400 border-blue-500/30' : 'bg-slate-800/50 text-slate-400 border-white/10'}`}
              title="Toggle Auto-scroll"
            >
              <svg
                class="w-4 h-4"
                viewBox="0 0 24 24"
                fill="none"
                stroke="currentColor"
                stroke-width="2"><path d="M12 5v14M19 12l-7 7-7-7" /></svg
              >
            </button>
          </div>
        {/if}
      </div>

      <!-- Footer: History -->
      <div class="glass-panel h-48 flex flex-col shrink-0">
        <div class="p-3 border-b border-white/5 bg-slate-800/30 flex justify-between items-center">
          <h3 class="text-sm font-medium text-slate-300">Recent History</h3>
          <button
            onclick={refreshHistory}
            class="text-xs text-slate-500 hover:text-white transition-colors">Refresh</button
          >
        </div>
        <div class="overflow-y-auto p-0 custom-scrollbar">
          <table class="w-full text-left text-sm text-slate-400">
            <thead
              class="bg-slate-900/50 text-xs uppercase font-semibold text-slate-500 sticky top-0 backdrop-blur-sm"
            >
              <tr>
                <th class="px-4 py-2">ID</th>
                <th class="px-4 py-2">Benchmark</th>
                <th class="px-4 py-2">Status</th>
                <th class="px-4 py-2">Duration</th>
                <th class="px-4 py-2">Finished</th>
              </tr>
            </thead>
            <tbody class="divide-y divide-white/5">
              {#each history as job (job.id)}
                <tr
                  class={`hover:bg-white/5 transition-colors cursor-pointer ${selectedHistoryJob?.id === job.id ? 'bg-blue-500/10' : ''}`}
                  onclick={() => (selectedHistoryJob = job)}
                >
                  <td class="px-4 py-2 font-mono text-xs">#{job.id}</td>
                  <td class="px-4 py-2 text-slate-200">{job.benchmark_name}</td>
                  <td class="px-4 py-2">
                    <span
                      class={`px-2 py-0.5 rounded text-xs border ${
                        job.status === 'completed'
                          ? 'bg-emerald-500/10 text-emerald-400 border-emerald-500/20'
                          : job.status === 'failed'
                            ? 'bg-red-500/10 text-red-400 border-red-500/20'
                            : job.status === 'killed'
                              ? 'bg-slate-500/10 text-slate-400 border-slate-500/20'
                              : 'bg-blue-500/10 text-blue-400 border-blue-500/20'
                      }`}
                    >
                      {job.status}
                    </span>
                  </td>
                  <td class="px-4 py-2 font-mono text-xs">
                    {#if job.started_at !== null && job.finished_at !== null}
                      {formattedTime(
                        (new Date(job.finished_at).getTime() - new Date(job.started_at).getTime()) /
                          1000,
                      )}
                    {:else}
                      -
                    {/if}
                  </td>
                  <td class="px-4 py-2 text-xs">
                    {job.finished_at !== null
                      ? new Date(job.finished_at).toLocaleTimeString()
                      : '-'}
                  </td>
                </tr>
              {/each}
              {#if history.length === 0}
                <tr>
                  <td colspan="5" class="px-4 py-8 text-center text-slate-600">No recent jobs</td>
                </tr>
              {/if}
            </tbody>
          </table>
        </div>
      </div>
    </div>
  </div>
</div>

<!-- Passphrase Modal -->
{#if showPassphraseModal}
  <div class="fixed inset-0 bg-black/60 backdrop-blur-sm flex items-center justify-center z-50">
    <div class="glass-panel w-full max-w-md mx-4 p-6">
      <div class="flex items-center gap-3 mb-6">
        <div class="p-3 bg-blue-500/20 rounded-xl">
          <svg
            class="w-6 h-6 text-blue-400"
            viewBox="0 0 24 24"
            fill="none"
            stroke="currentColor"
            stroke-width="2"
          >
            <rect x="3" y="11" width="18" height="11" rx="2" ry="2"></rect>
            <path d="M7 11V7a5 5 0 0 1 10 0v4"></path>
          </svg>
        </div>
        <div>
          <h2 class="text-xl font-bold text-white">SSH Key Passphrase</h2>
          <p class="text-sm text-slate-400">
            {#if sshKeyStatus?.type === 'NeedsPassphrase'}
              Enter passphrase for {sshKeyStatus.data.key_path}
            {:else}
              Authentication required
            {/if}
          </p>
        </div>
      </div>

      {#if passphraseError}
        <div
          class="mb-4 p-3 bg-red-500/10 border border-red-500/20 rounded-lg text-red-400 text-sm"
        >
          {passphraseError}
        </div>
      {/if}

      {#if sshKeyStatus?.type === 'NeedsPassphrase'}
        <form
          onsubmit={e => {
            e.preventDefault();
            void submitPassphrase();
          }}
        >
          <div class="mb-6">
            <label for="passphrase" class="block text-sm font-medium text-slate-300 mb-2">
              Passphrase
            </label>
            <!-- svelte-ignore a11y_autofocus -->
            <input
              type="password"
              id="passphrase"
              bind:value={passphrase}
              disabled={isAddingKey}
              placeholder="Enter your SSH key passphrase"
              class="w-full px-4 py-3 bg-slate-800/50 border border-white/10 rounded-lg text-white placeholder-slate-500 focus:outline-none focus:border-blue-500/50 focus:ring-2 focus:ring-blue-500/20 transition-all disabled:opacity-50"
              autofocus
            />
          </div>

          <div class="flex gap-3">
            <button
              type="submit"
              disabled={isAddingKey || !passphrase.trim()}
              class="flex-1 btn-glass btn-primary flex items-center justify-center gap-2 py-3"
            >
              {#if isAddingKey}
                <svg
                  class="animate-spin w-4 h-4"
                  viewBox="0 0 24 24"
                  fill="none"
                  stroke="currentColor"
                  stroke-width="2"
                >
                  <path d="M21 12a9 9 0 1 1-6.219-8.56" />
                </svg>
                Unlocking...
              {:else}
                <svg
                  class="w-4 h-4"
                  viewBox="0 0 24 24"
                  fill="none"
                  stroke="currentColor"
                  stroke-width="2"
                >
                  <path d="M15 3h4a2 2 0 0 1 2 2v14a2 2 0 0 1-2 2h-4M10 17l5-5-5-5M13.8 12H3" />
                </svg>
                Unlock & Connect
              {/if}
            </button>
          </div>
        </form>
      {:else}
        <!-- Error state (NoKey or NoAgent) -->
        <p class="text-slate-300 mb-6">
          {#if sshKeyStatus?.type === 'NoAgent'}
            The SSH agent is not running. Start it with:
            <code class="block mt-2 p-2 bg-black/40 rounded text-blue-300 text-sm font-mono">
              eval $(ssh-agent) && ssh-add
            </code>
          {:else if sshKeyStatus?.type === 'NoKey'}
            No SSH key found at the expected location. Create one with:
            <code class="block mt-2 p-2 bg-black/40 rounded text-blue-300 text-sm font-mono">
              ssh-keygen -t ed25519
            </code>
          {/if}
        </p>
        <button
          onclick={() => init()}
          class="w-full btn-glass flex items-center justify-center gap-2 py-3"
        >
          <svg
            class="w-4 h-4"
            viewBox="0 0 24 24"
            fill="none"
            stroke="currentColor"
            stroke-width="2"
          >
            <path d="M21 12a9 9 0 1 1-9-9c2.52 0 4.93 1 6.74 2.74L21 8" />
            <path d="M21 3v5h-5" />
          </svg>
          Retry
        </button>
      {/if}
    </div>
  </div>
{/if}
