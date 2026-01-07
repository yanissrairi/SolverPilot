<script lang="ts">
  import { onMount, onDestroy } from 'svelte';
  import { SvelteSet } from 'svelte/reactivity';
  import {
    checkConfigExists,
    loadConfig,
    initSsh,
    closeSsh,
    checkSshKeyStatus,
    addSshKey,
    checkSyncStatus,
    syncCode,
    pickBenchmarkFile,
    addBenchmarkToProject,
    removeBenchmarkFromProject,
    listProjectBenchmarks,
    getBenchmarkDependencies,
    queueJobs,
    startNextJob,
    stopJob,
    killJob,
    getJobStatus,
    loadHistory,
    getActiveProject,
  } from './lib/api';
  import type {
    Benchmark,
    Project,
    Job,
    JobStatusResponse,
    SyncStatus,
    SshKeyStatus,
    DependencyAnalysis,
  } from './lib/types';
  import DependencyPanel from './lib/DependencyPanel.svelte';
  import MainLayout from './lib/layout/MainLayout.svelte';
  import BenchmarkList from './lib/features/benchmarks/BenchmarkList.svelte';
  import JobMonitor from './lib/features/jobs/JobMonitor.svelte';
  import HistoryPanel from './lib/features/history/HistoryPanel.svelte';
  import SetupWizard from './lib/features/setup/SetupWizard.svelte';
  import ToastContainer from './lib/ui/ToastContainer.svelte';
  import { setupGlobalShortcuts, registerShortcut } from './lib/stores/shortcuts.svelte';

  // --- STATE ---
  let activeProject = $state<Project | null>(null);
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

  // Config error / Setup wizard
  let configError = $state<string | null>(null);
  let needsSetup = $state(false);
  let checkingConfig = $state(true);

  // Active Job
  let currentJobStatus = $state<JobStatusResponse | null>(null);
  let isRunning = $state(false);

  // History view
  let selectedHistoryJob = $state<Job | null>(null);

  // Dependency analysis
  let focusedBenchmark = $state<Benchmark | null>(null);
  let dependencyAnalysis = $state<DependencyAnalysis | null>(null);
  let isLoadingDeps = $state(false);

  // Logs
  let autoScroll = $state(true);

  // Polling
  let pollInterval: number | null = null;

  // Error state
  let benchmarkError = $state<string | null>(null);

  // --- ACTIONS ---

  async function init() {
    configError = null;
    checkingConfig = true;

    try {
      // Check if config exists first
      const exists = await checkConfigExists();
      if (!exists) {
        needsSetup = true;
        checkingConfig = false;
        return;
      }

      await loadConfig();
    } catch (e) {
      const errMsg = e instanceof Error ? e.message : String(e);
      if (errMsg.includes('missing field')) {
        const regex = /missing field `(\w+)`/;
        const match = regex.exec(errMsg);
        const field = match?.[1] ?? 'unknown';
        configError = `Configuration invalide: le champ "${field}" est manquant dans config.toml.\nVoir config.example.toml pour référence.`;
      } else {
        configError = `Erreur de configuration: ${errMsg}`;
      }
      checkingConfig = false;
      return;
    }

    checkingConfig = false;

    try {
      sshKeyStatus = await checkSshKeyStatus();

      if (sshKeyStatus.type === 'NeedsPassphrase') {
        showPassphraseModal = true;
        return;
      } else if (sshKeyStatus.type === 'NoKey') {
        passphraseError = `Clé SSH non trouvée: ${sshKeyStatus.data.expected_path}`;
        showPassphraseModal = true;
        return;
      } else if (sshKeyStatus.type === 'NoAgent') {
        passphraseError = "Agent SSH non démarré. Lancez 'eval $(ssh-agent)' dans un terminal.";
        showPassphraseModal = true;
        return;
      }

      await completeInit();
    } catch {
      sshReady = false;
    }
  }

  async function completeInit(skipInit = false) {
    try {
      // Skip initSsh() if we just called addSshKey() with passphrase
      if (!skipInit) {
        await initSsh();
      }
      sshReady = true;
      showPassphraseModal = false;
      passphraseError = '';

      activeProject = await getActiveProject();
      if (activeProject) {
        void refreshBenchmarks();
        void refreshSync();
      }
      void refreshHistory();
      startPolling();
    } catch (e) {
      sshReady = false;
      passphraseError = e instanceof Error ? e.message : String(e);
      showPassphraseModal = true;
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
      passphrase = '';
      // Skip initSsh() since addSshKey() already created the manager with passphrase
      await completeInit(true);
    } catch (e) {
      passphraseError = String(e);
    } finally {
      isAddingKey = false;
    }
  }

  function handleProjectChange(project: Project | null) {
    activeProject = project;
    benchmarks = [];
    selectedBenchmarks.clear();
    focusedBenchmark = null;
    dependencyAnalysis = null;

    if (project) {
      void refreshBenchmarks();
      void refreshSync();
    }
  }

  async function refreshSync() {
    if (isSyncing || !activeProject) return;
    try {
      syncStatus = await checkSyncStatus();
    } catch (e) {
      syncStatus = { type: 'Error', data: { message: String(e) } };
    }
  }

  async function performSync() {
    if (!activeProject) return;
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
    if (!activeProject) {
      benchmarks = [];
      return;
    }
    try {
      benchmarkError = null;
      benchmarks = await listProjectBenchmarks();
    } catch (e) {
      benchmarkError = `Échec chargement: ${String(e)}`;
      benchmarks = [];
    }
  }

  async function addBenchmarkFile() {
    if (!activeProject) return;

    try {
      const filePath = await pickBenchmarkFile();
      if (filePath === null) return;

      const benchmark = await addBenchmarkToProject(filePath);

      if (!benchmarks.some(b => b.id === benchmark.id)) {
        benchmarks = [...benchmarks, benchmark];
      }
    } catch (e) {
      benchmarkError = `Échec ajout: ${String(e)}`;
    }
  }

  async function removeBenchmark(benchmark: Benchmark) {
    if (!confirm(`Retirer "${benchmark.name}" du projet ?`)) return;

    try {
      await removeBenchmarkFromProject(benchmark.id);
      benchmarks = benchmarks.filter(b => b.id !== benchmark.id);
      selectedBenchmarks.delete(benchmark.name);

      if (focusedBenchmark?.id === benchmark.id) {
        focusedBenchmark = null;
        dependencyAnalysis = null;
      }
    } catch (e) {
      benchmarkError = `Échec suppression: ${String(e)}`;
    }
  }

  async function reanalyzeDependencies() {
    if (!focusedBenchmark) return;

    isLoadingDeps = true;
    try {
      dependencyAnalysis = await getBenchmarkDependencies(focusedBenchmark.path);
    } catch (e) {
      benchmarkError = `Échec analyse: ${String(e)}`;
    } finally {
      isLoadingDeps = false;
    }
  }

  async function focusBenchmark(bench: Benchmark) {
    if (focusedBenchmark?.path === bench.path) {
      focusedBenchmark = null;
      dependencyAnalysis = null;
      return;
    }

    focusedBenchmark = bench;
    isLoadingDeps = true;
    dependencyAnalysis = null;

    try {
      dependencyAnalysis = await getBenchmarkDependencies(bench.path);
    } catch (e) {
      benchmarkError = `Échec analyse: ${String(e)}`;
    } finally {
      isLoadingDeps = false;
    }
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
    if (selectedBenchmarks.size === 0 || !activeProject) return;
    try {
      await queueJobs(Array.from(selectedBenchmarks));
      await startNextJob();
      startPolling();
    } catch {
      // Job start failed silently
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

        if (status.job && (status.job.status === 'running' || status.job.status === 'pending')) {
          isRunning = true;
        } else if (status.is_finished && isRunning) {
          isRunning = false;
          void refreshHistory();
          const next = await startNextJob();
          if (next) isRunning = true;
        } else {
          isRunning = false;
        }
      } catch {
        // Poll error
      }
    }, 1000);
  }

  // Lifecycle
  onMount(() => {
    setupGlobalShortcuts();

    registerShortcut({
      key: 'Enter',
      ctrl: true,
      action: () => void runSelected(),
      description: 'Run selected benchmarks',
    });

    registerShortcut({
      key: 's',
      ctrl: true,
      action: () => void performSync(),
      description: 'Sync code',
    });

    registerShortcut({
      key: 'a',
      ctrl: true,
      action: toggleAll,
      description: 'Select/Deselect all',
    });

    registerShortcut({
      key: ' ',
      action: () => {
        if (focusedBenchmark) toggleBenchmark(focusedBenchmark.name);
      },
      description: 'Toggle selection',
    });

    registerShortcut({
      key: 'l',
      ctrl: true,
      action: () => {
        autoScroll = !autoScroll;
      },
      description: 'Toggle auto-scroll',
    });

    registerShortcut({
      key: 'Escape',
      action: () => {
        // Priority: Passphrase Modal -> History -> Focus -> Selection
        if (showPassphraseModal) {
          showPassphraseModal = false;
          return;
        }

        if (selectedHistoryJob) {
          selectedHistoryJob = null;
          return;
        }

        if (focusedBenchmark) {
          focusedBenchmark = null;
          return;
        }

        if (selectedBenchmarks.size > 0) {
          selectedBenchmarks.clear();
          return;
        }
      },
      description: 'Close/Deselect',
    });

    void init();
  });

  async function handleSetupComplete() {
    needsSetup = false;
    await init();
  }

  onDestroy(() => {
    if (pollInterval !== null) clearInterval(pollInterval);
    void closeSsh();
    // We don't verify destroy shortcuts here because App is root
  });
</script>

<!-- Toast notifications -->
<ToastContainer />

{#if checkingConfig}
  <!-- Loading state while checking config -->
  <div class="min-h-screen flex items-center justify-center">
    <div class="text-center space-y-4">
      <div
        class="w-8 h-8 border-2 border-blue-500 border-t-transparent rounded-full animate-spin mx-auto"
      ></div>
      <p class="text-slate-400">Chargement...</p>
    </div>
  </div>
{:else if needsSetup}
  <!-- Setup wizard for first-time configuration -->
  <SetupWizard onComplete={handleSetupComplete} />
{:else}
  <!-- Main application -->
  <MainLayout {activeProject} onProjectChange={handleProjectChange}>
    {#snippet headerChildren()}
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
      {#if activeProject}
        <div
          class="flex items-center gap-3 px-3 py-1.5 rounded-full bg-slate-900/50 border border-white/5"
        >
          <span class="text-slate-400">Sync</span>

          {#if syncStatus.type === 'Checking'}
            <span class="text-yellow-400 animate-pulse">Checking...</span>
          {:else if syncStatus.type === 'UpToDate'}
            <span class="text-emerald-400 flex items-center gap-1">
              <svg
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
              class="text-xs bg-amber-500/20 hover:bg-amber-500/30 text-amber-300 px-2 py-0.5 rounded-sm transition-colors uppercase tracking-wider"
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
      {/if}
    {/snippet}

    {#snippet leftPanel()}
      <BenchmarkList
        {benchmarks}
        {selectedBenchmarks}
        {focusedBenchmark}
        {activeProject}
        {isRunning}
        bind:benchmarkError
        onadd={addBenchmarkFile}
        onrefresh={refreshBenchmarks}
        ontoggle={toggleBenchmark}
        ontoggleall={toggleAll}
        onfocus={focusBenchmark}
        onremove={removeBenchmark}
        onrun={runSelected}
      />
    {/snippet}

    {#snippet middlePanel()}
      <div class="h-full flex flex-col min-h-0 glass-panel">
        <JobMonitor
          {currentJobStatus}
          {isRunning}
          {selectedHistoryJob}
          bind:autoScroll
          onstop={handleStop}
          onkill={handleKill}
          onbacktolist={() => (selectedHistoryJob = null)}
        />
        <div class="h-px bg-white/5 my-0"></div>
        <HistoryPanel
          {history}
          {selectedHistoryJob}
          onselect={(job: Job) => (selectedHistoryJob = job)}
          onrefresh={refreshHistory}
        />
      </div>
    {/snippet}

    {#snippet rightPanel()}
      <DependencyPanel
        analysis={dependencyAnalysis}
        isLoading={isLoadingDeps}
        {activeProject}
        onDependencyAdded={reanalyzeDependencies}
      />
    {/snippet}
  </MainLayout>
{/if}

<!-- Passphrase Modal -->
{#if showPassphraseModal}
  <div class="fixed inset-0 bg-black/60 backdrop-blur-xs flex items-center justify-center z-50">
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
              class="w-full px-4 py-3 bg-slate-800/50 border border-white/10 rounded-lg text-white placeholder-slate-500 focus:outline-hidden focus:border-blue-500/50 focus:ring-2 focus:ring-blue-500/20 transition-all disabled:opacity-50"
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
      {:else if sshKeyStatus?.type === 'InAgent'}
        <p class="text-slate-300 mb-6">
          SSH key is loaded in agent, but connection failed. Check your network or server
          availability.
        </p>
        <button
          onclick={() => init()}
          class="w-full btn-glass btn-primary flex items-center justify-center gap-2 py-3"
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
          Retry Connection
        </button>
      {:else}
        <p class="text-slate-300 mb-6">
          {#if sshKeyStatus?.type === 'NoAgent'}
            The SSH agent is not running. Start it with:
            <code class="block mt-2 p-2 bg-black/40 rounded-sm text-blue-300 text-sm font-mono">
              eval $(ssh-agent) && ssh-add
            </code>
          {:else if sshKeyStatus?.type === 'NoKey'}
            No SSH key found at the expected location. Create one with:
            <code class="block mt-2 p-2 bg-black/40 rounded-sm text-blue-300 text-sm font-mono">
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

<!-- Config Error Modal -->
{#if configError}
  <div class="fixed inset-0 bg-black/60 backdrop-blur-xs flex items-center justify-center z-50">
    <div class="glass-panel w-full max-w-lg mx-4 p-6">
      <div class="flex items-center gap-3 mb-6">
        <div class="p-3 bg-red-500/20 rounded-xl">
          <svg
            class="w-6 h-6 text-red-400"
            viewBox="0 0 24 24"
            fill="none"
            stroke="currentColor"
            stroke-width="2"
          >
            <circle cx="12" cy="12" r="10"></circle>
            <line x1="12" y1="8" x2="12" y2="12"></line>
            <line x1="12" y1="16" x2="12.01" y2="16"></line>
          </svg>
        </div>
        <div>
          <h2 class="text-xl font-bold text-white">Erreur de Configuration</h2>
          <p class="text-sm text-slate-400">Le fichier config.toml est invalide</p>
        </div>
      </div>

      <div class="mb-6 p-4 bg-red-500/10 border border-red-500/20 rounded-lg">
        <pre class="text-red-300 text-sm whitespace-pre-wrap font-mono">{configError}</pre>
      </div>

      <div class="p-4 bg-slate-800/50 rounded-lg mb-6">
        <p class="text-sm text-slate-300 mb-2">Fichier attendu :</p>
        <code class="text-xs text-blue-300 bg-black/40 px-2 py-1 rounded-sm">./config.toml</code>
        <p class="text-xs text-slate-500 mt-2">Copiez config.example.toml et adaptez-le.</p>
      </div>

      <button
        onclick={() => init()}
        class="w-full btn-glass flex items-center justify-center gap-2 py-3"
      >
        <svg class="w-4 h-4" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
          <path d="M21 12a9 9 0 1 1-9-9c2.52 0 4.93 1 6.74 2.74L21 8" />
          <path d="M21 3v5h-5" />
        </svg>
        Réessayer
      </button>
    </div>
  </div>
{/if}
