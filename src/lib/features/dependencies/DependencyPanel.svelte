<script lang="ts">
  import type { DependencyAnalysis, LocalDependency, Project } from '../../types';
  import * as api from '../../api';

  interface Props {
    analysis: DependencyAnalysis | null;
    isLoading: boolean;
    activeProject: Project | null;
    onDependencyAdded?: () => void;
  }

  const { analysis, isLoading, activeProject, onDependencyAdded }: Props = $props();

  // Project dependencies state
  let projectDeps = $state<string[]>([]);
  let isLoadingDeps = $state(false);
  let newDepName = $state('');
  let depsError = $state<string | null>(null);

  // Track which package is being added/removed
  let addingPackage = $state<string | null>(null);
  let removingPackage = $state<string | null>(null);
  let isUpdatingAll = $state(false);
  let isSyncing = $state(false);

  // Load project dependencies when project changes
  $effect(() => {
    if (activeProject) {
      void loadProjectDependencies();
    } else {
      projectDeps = [];
    }
  });

  async function loadProjectDependencies() {
    if (!activeProject) return;
    isLoadingDeps = true;
    depsError = null;
    try {
      projectDeps = await api.listProjectDependencies();
    } catch (e) {
      depsError = e instanceof Error ? e.message : String(e);
    } finally {
      isLoadingDeps = false;
    }
  }

  // Extract filename from full path
  function getFileName(path: string): string {
    return path.split('/').pop() ?? path;
  }

  // Add package to project (from benchmark analysis or manual input)
  async function handleAddPackage(packageName: string) {
    if (!packageName.trim()) return;
    addingPackage = packageName;
    depsError = null;

    try {
      await api.addProjectDependency(packageName);
      await loadProjectDependencies();
      newDepName = '';
      onDependencyAdded?.();
    } catch (e) {
      depsError = e instanceof Error ? e.message : String(e);
    } finally {
      addingPackage = null;
    }
  }

  // Remove package from project
  async function handleRemovePackage(packageName: string) {
    removingPackage = packageName;
    depsError = null;

    try {
      await api.removeProjectDependency(packageName);
      await loadProjectDependencies();
      onDependencyAdded?.();
    } catch (e) {
      depsError = e instanceof Error ? e.message : String(e);
    } finally {
      removingPackage = null;
    }
  }

  // Update all dependencies
  async function handleUpdateAll() {
    isUpdatingAll = true;
    depsError = null;

    try {
      await api.updateProjectDependencies();
      await loadProjectDependencies();
    } catch (e) {
      depsError = e instanceof Error ? e.message : String(e);
    } finally {
      isUpdatingAll = false;
    }
  }

  // Sync environment
  async function handleSyncEnv() {
    isSyncing = true;
    depsError = null;

    try {
      await api.syncProjectEnvironment();
    } catch (e) {
      depsError = e instanceof Error ? e.message : String(e);
    } finally {
      isSyncing = false;
    }
  }

  // Handle form submission
  function handleAddFormSubmit(e: Event) {
    e.preventDefault();
    const trimmed = newDepName.trim();
    if (trimmed) {
      void handleAddPackage(trimmed);
    }
  }

  // Unified packages list: merge projectDeps + benchmark analysis
  const unifiedPackages = $derived.by(() => {
    const packages: Record<string, boolean> = {};

    // Add project deps (all installed)
    for (const dep of projectDeps) {
      packages[dep] = true;
    }

    // Add benchmark imports (check if in project)
    if (analysis) {
      for (const pkg of analysis.external_packages) {
        if (!(pkg.name in packages)) {
          packages[pkg.name] = pkg.in_pyproject;
        }
      }
    }

    return Object.entries(packages)
      .map(([name, installed]) => ({ name, installed }))
      .sort((a, b) => a.name.localeCompare(b.name));
  });
</script>

<div class="glass-panel h-full flex flex-col">
  <div class="p-3 border-b border-white/5 bg-slate-800/30">
    <h3 class="text-sm font-medium text-slate-300 flex items-center gap-2">
      <svg
        class="w-4 h-4 text-blue-400"
        viewBox="0 0 24 24"
        fill="none"
        stroke="currentColor"
        stroke-width="2"
      >
        <path d="M3 3v18h18" />
        <path d="M7 16l4-8 4 4 4-6" />
      </svg>
      Dependencies
    </h3>
  </div>

  <div class="flex-1 overflow-y-auto custom-scrollbar p-3 space-y-4">
    {#if !activeProject}
      <!-- No project selected -->
      <div class="flex flex-col items-center justify-center py-8 text-slate-500 gap-2">
        <svg
          class="w-8 h-8 opacity-50"
          viewBox="0 0 24 24"
          fill="none"
          stroke="currentColor"
          stroke-width="1.5"
        >
          <path d="M3 7v10a2 2 0 002 2h14a2 2 0 002-2V9a2 2 0 00-2-2h-6l-2-2H5a2 2 0 00-2 2z" />
        </svg>
        <p class="text-xs">Select a project first</p>
      </div>
    {:else}
      {#if depsError}
        <div
          class="mb-3 p-2 bg-red-500/10 border border-red-500/20 rounded-sm text-xs text-red-300"
        >
          {depsError}
        </div>
      {/if}

      <!-- Add dependency form -->
      <form onsubmit={handleAddFormSubmit} class="mb-3 flex gap-2">
        <input
          type="text"
          bind:value={newDepName}
          placeholder="package-name"
          class="flex-1 bg-slate-800/50 border border-slate-700 rounded-sm px-2 py-1.5 text-xs focus:outline-hidden focus:border-blue-500 font-mono"
          disabled={addingPackage !== null || removingPackage !== null}
        />
        <button
          type="submit"
          disabled={!newDepName.trim() || addingPackage !== null}
          class="px-3 py-1.5 bg-emerald-600 hover:bg-emerald-500 rounded-sm text-xs font-medium transition-colors disabled:opacity-50 disabled:cursor-not-allowed"
        >
          {#if addingPackage === newDepName.trim()}
            <svg
              class="w-3.5 h-3.5 animate-spin"
              viewBox="0 0 24 24"
              fill="none"
              stroke="currentColor"
              stroke-width="2"
            >
              <path d="M21 12a9 9 0 1 1-6.219-8.56" />
            </svg>
          {:else}
            Add
          {/if}
        </button>
      </form>

      <!-- Action buttons -->
      <div class="flex gap-2 mb-4">
        <button
          onclick={handleUpdateAll}
          disabled={isUpdatingAll || projectDeps.length === 0}
          class="flex-1 px-2 py-1.5 bg-slate-700 hover:bg-slate-600 rounded-sm text-xs font-medium transition-colors disabled:opacity-50 flex items-center justify-center gap-1.5"
        >
          {#if isUpdatingAll}
            <svg
              class="w-3.5 h-3.5 animate-spin"
              viewBox="0 0 24 24"
              fill="none"
              stroke="currentColor"
              stroke-width="2"
            >
              <path d="M21 12a9 9 0 1 1-6.219-8.56" />
            </svg>
          {:else}
            <svg
              class="w-3.5 h-3.5"
              viewBox="0 0 24 24"
              fill="none"
              stroke="currentColor"
              stroke-width="2"
            >
              <path
                d="M21 2v6h-6M3 12a9 9 0 0 1 15-6.7L21 8M3 22v-6h6M21 12a9 9 0 0 1-15 6.7L3 16"
              />
            </svg>
          {/if}
          Update
        </button>
        <button
          onclick={handleSyncEnv}
          disabled={isSyncing}
          class="flex-1 px-2 py-1.5 bg-blue-600/80 hover:bg-blue-600 rounded-sm text-xs font-medium transition-colors disabled:opacity-50 flex items-center justify-center gap-1.5"
        >
          {#if isSyncing}
            <svg
              class="w-3.5 h-3.5 animate-spin"
              viewBox="0 0 24 24"
              fill="none"
              stroke="currentColor"
              stroke-width="2"
            >
              <path d="M21 12a9 9 0 1 1-6.219-8.56" />
            </svg>
          {:else}
            <svg
              class="w-3.5 h-3.5"
              viewBox="0 0 24 24"
              fill="none"
              stroke="currentColor"
              stroke-width="2"
            >
              <polyline points="20 6 9 17 4 12" />
            </svg>
          {/if}
          Sync
        </button>
      </div>

      <!-- Unified Packages List -->
      {#if isLoadingDeps || isLoading}
        <div class="flex items-center gap-2 text-slate-500 text-xs py-4 justify-center">
          <svg
            class="w-4 h-4 animate-spin"
            viewBox="0 0 24 24"
            fill="none"
            stroke="currentColor"
            stroke-width="2"
          >
            <path d="M21 12a9 9 0 1 1-6.219-8.56" />
          </svg>
          {isLoading ? 'Analyzing...' : 'Loading...'}
        </div>
      {:else if unifiedPackages.length === 0}
        <p class="text-xs text-slate-500 italic text-center py-4">No dependencies yet</p>
      {:else}
        <div class="space-y-1">
          {#each unifiedPackages as pkg (pkg.name)}
            <div
              class="flex items-center gap-2 px-2 py-1.5 rounded text-sm transition-colors group {pkg.installed
                ? 'text-slate-300 hover:bg-white/5'
                : 'text-amber-300 bg-amber-500/10'}"
              title={pkg.installed ? 'Installed' : 'Missing - click + to add'}
            >
              {#if pkg.installed}
                <svg
                  class="w-3.5 h-3.5 text-emerald-400 shrink-0"
                  viewBox="0 0 24 24"
                  fill="none"
                  stroke="currentColor"
                  stroke-width="2"
                >
                  <polyline points="20 6 9 17 4 12" />
                </svg>
              {:else}
                <svg
                  class="w-3.5 h-3.5 text-amber-400 shrink-0"
                  viewBox="0 0 24 24"
                  fill="none"
                  stroke="currentColor"
                  stroke-width="2"
                >
                  <path
                    d="M10.29 3.86L1.82 18a2 2 0 0 0 1.71 3h16.94a2 2 0 0 0 1.71-3L13.71 3.86a2 2 0 0 0-3.42 0z"
                  />
                  <line x1="12" y1="9" x2="12" y2="13" />
                  <line x1="12" y1="17" x2="12.01" y2="17" />
                </svg>
              {/if}
              <span class="font-mono text-xs flex-1">{pkg.name}</span>

              {#if pkg.installed}
                <!-- Remove button -->
                <button
                  onclick={() => {
                    void handleRemovePackage(pkg.name);
                  }}
                  disabled={removingPackage !== null}
                  class="p-1 rounded-sm hover:bg-red-500/20 text-red-400 opacity-0 group-hover:opacity-100 transition-all disabled:opacity-50"
                  title="Remove"
                >
                  {#if removingPackage === pkg.name}
                    <svg
                      class="w-3.5 h-3.5 animate-spin"
                      viewBox="0 0 24 24"
                      fill="none"
                      stroke="currentColor"
                      stroke-width="2"
                    >
                      <path d="M21 12a9 9 0 1 1-6.219-8.56" />
                    </svg>
                  {:else}
                    <svg
                      class="w-3.5 h-3.5"
                      viewBox="0 0 24 24"
                      fill="none"
                      stroke="currentColor"
                      stroke-width="2"
                    >
                      <path d="M18 6L6 18M6 6l12 12" />
                    </svg>
                  {/if}
                </button>
              {:else}
                <!-- Add button -->
                <button
                  onclick={() => {
                    void handleAddPackage(pkg.name);
                  }}
                  disabled={addingPackage !== null}
                  class="p-1 rounded-sm hover:bg-emerald-500/20 text-emerald-400 transition-colors disabled:opacity-50"
                  title="Add to project"
                >
                  {#if addingPackage === pkg.name}
                    <svg
                      class="w-3.5 h-3.5 animate-spin"
                      viewBox="0 0 24 24"
                      fill="none"
                      stroke="currentColor"
                      stroke-width="2"
                    >
                      <path d="M21 12a9 9 0 1 1-6.219-8.56" />
                    </svg>
                  {:else}
                    <svg
                      class="w-3.5 h-3.5"
                      viewBox="0 0 24 24"
                      fill="none"
                      stroke="currentColor"
                      stroke-width="2"
                    >
                      <path d="M12 5v14M5 12h14" />
                    </svg>
                  {/if}
                </button>
              {/if}
            </div>
          {/each}
        </div>
      {/if}

      <!-- Local Files Section (only if has files) -->
      {#if analysis?.local_files && analysis.local_files.length > 0}
        <div class="border-t border-white/5 mt-4 pt-4">
          <h4
            class="text-xs font-semibold text-slate-400 uppercase tracking-wider mb-2 flex items-center gap-2"
          >
            <svg
              class="w-3.5 h-3.5"
              viewBox="0 0 24 24"
              fill="none"
              stroke="currentColor"
              stroke-width="2"
            >
              <path d="M13 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V9z" />
              <polyline points="13 2 13 9 20 9" />
            </svg>
            Local Files ({analysis.local_files.length})
          </h4>
          <div class="space-y-0.5">
            {#each analysis.local_files as dep (dep.file_path)}
              {@render localDep(dep)}
            {/each}
          </div>
        </div>
      {/if}
    {/if}
  </div>
</div>

{#snippet localDep(dep: LocalDependency)}
  <div
    class="flex items-center gap-2 px-2 py-1 rounded-sm text-sm hover:bg-white/5 transition-colors"
    title={dep.file_path}
  >
    <svg
      class="w-3 h-3 text-slate-500 shrink-0"
      viewBox="0 0 24 24"
      fill="none"
      stroke="currentColor"
      stroke-width="2"
    >
      <path d="M13 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V9z" />
      <polyline points="13 2 13 9 20 9" />
    </svg>

    <span class={dep.exists ? 'text-slate-300' : 'text-red-400'}>
      {getFileName(dep.file_path)}
    </span>

    {#if !dep.exists}
      <span
        class="text-xs px-1.5 py-0.5 rounded-sm bg-red-500/20 text-red-300 border border-red-500/30"
      >
        missing
      </span>
    {/if}
  </div>

  {#each dep.children as child (child.file_path)}
    {@render localDep(child)}
  {/each}
{/snippet}
