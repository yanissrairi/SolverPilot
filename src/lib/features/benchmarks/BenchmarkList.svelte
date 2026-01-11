<script lang="ts">
  import { onMount, onDestroy } from 'svelte';
  import type { SvelteSet } from 'svelte/reactivity';
  import type { Benchmark, Project } from '../../types';
  import { registerShortcut, unregisterShortcut } from '../../stores/shortcuts.svelte';
  import { queueBenchmarks } from '../../api';
  import { toast } from '../../stores/toast.svelte';

  let {
    benchmarks,
    selectedBenchmarks,
    focusedBenchmark,
    activeProject,
    isRunning,
    benchmarkError = $bindable(),
    onadd,
    onrefresh,
    ontoggle,
    ontoggleall,
    onfocus,
    onremove,
    onrun,
  } = $props<{
    benchmarks: Benchmark[];
    selectedBenchmarks: SvelteSet<string>;
    focusedBenchmark: Benchmark | null;
    activeProject: Project | null;
    isRunning: boolean;
    benchmarkError?: string | null;
    onadd: () => void;
    onrefresh: () => void;
    ontoggle: (name: string) => void;
    ontoggleall: () => void;
    onfocus: (bench: Benchmark) => void;
    onremove: (bench: Benchmark) => void;
    onrun: () => void;
  }>();

  // Reference to the benchmark list container for click-outside detection (H1 fix)
  let listContainerRef: HTMLElement | null = $state(null);

  // Track last clicked index for range selection (Subtask 1.1)
  let lastClickedIndex = $state<number | null>(null);

  // Derived selection count for display (Subtask 1.2)
  let selectedCount = $derived(selectedBenchmarks.size);
  let selectionSummary = $derived(
    selectedCount > 0
      ? `${selectedCount.toString()} benchmark${selectedCount === 1 ? '' : 's'} selected`
      : '',
  );

  // Enhanced click handler for multi-select (Task 2)
  function handleBenchmarkClick(event: MouseEvent, bench: Benchmark, index: number): void {
    // Prevent event from bubbling to document click handler
    event.stopPropagation();

    if (event.shiftKey) {
      // Range selection: Shift+Click (Subtask 2.2) - M2 fix: handle null lastClickedIndex
      const start = Math.min(lastClickedIndex ?? 0, index);
      const end = Math.max(lastClickedIndex ?? 0, index);
      for (let i = start; i <= end; i++) {
        selectedBenchmarks.add(benchmarks[i].name);
      }
    } else if (event.ctrlKey || event.metaKey) {
      // Individual toggle: Ctrl/Cmd+Click (Subtask 2.3)
      if (selectedBenchmarks.has(bench.name)) {
        selectedBenchmarks.delete(bench.name);
      } else {
        selectedBenchmarks.add(bench.name);
      }
    } else {
      // Single selection: clear others and select this one (Subtask 2.1)
      ontoggle(bench.name);
    }
    lastClickedIndex = index;
  }

  // Keyboard handler for benchmark row (M3 fix: proper a11y instead of svelte-ignore)
  function handleBenchmarkKeydown(event: KeyboardEvent, bench: Benchmark, index: number): void {
    if (event.key === ' ' || event.key === 'Enter') {
      event.preventDefault();
      // Toggle selection on Space/Enter
      ontoggle(bench.name);
      lastClickedIndex = index;
    } else if (event.key === 'ArrowDown') {
      event.preventDefault();
      // Move focus to next benchmark (H2 fix)
      if (index < benchmarks.length - 1) {
        focusRowByIndex(index + 1);
      }
    } else if (event.key === 'ArrowUp') {
      event.preventDefault();
      // Move focus to previous benchmark (H2 fix)
      if (index > 0) {
        focusRowByIndex(index - 1);
      }
    }
  }

  // Focus a benchmark row by index (H2 fix)
  function focusRowByIndex(index: number): void {
    const row = listContainerRef?.querySelector(`[data-benchmark-index="${index.toString()}"]`);
    if (row instanceof HTMLElement) {
      row.focus();
    }
  }

  // Click-outside handler to clear selection (H1 fix: AC6 "click elsewhere")
  function handleDocumentClick(event: MouseEvent): void {
    if (listContainerRef && !listContainerRef.contains(event.target as Node)) {
      if (selectedBenchmarks.size > 0) {
        selectedBenchmarks.clear();
        lastClickedIndex = null;
      }
    }
  }

  // Register keyboard shortcuts (Task 3)
  onMount(() => {
    // H1 fix: Add document click listener for click-outside
    document.addEventListener('click', handleDocumentClick);

    // Space key: Toggle focused benchmark (Subtask 3.1)
    registerShortcut({
      key: ' ',
      action: () => {
        if (focusedBenchmark) {
          ontoggle(focusedBenchmark.name);
        }
      },
      description: 'Toggle selected benchmark',
    });

    // Q key: Queue selected benchmarks (Subtask 3.2 - Story 1.2)
    // Enhanced with duplicate detection (Story 1.5)
    registerShortcut({
      key: 'q',
      action: () => {
        if (selectedBenchmarks.size > 0) {
          void (async () => {
            try {
              // Get benchmark IDs from selected names
              const benchmarkIds = Array.from(selectedBenchmarks)
                .map(name => benchmarks.find((b: Benchmark) => b.name === name)?.id)
                .filter((id): id is number => id !== undefined);

              // Queue benchmarks with duplicate detection (Story 1.5)
              const queuedJobs = await queueBenchmarks(benchmarkIds, false);

              toast.success(
                `${queuedJobs.length.toString()} benchmark${queuedJobs.length === 1 ? '' : 's'} added to queue`,
              );

              // Clear selection after successful queue
              selectedBenchmarks.clear();
            } catch (error) {
              const message = String(error);

              // Check for duplicate warning format: "DUPLICATE_WARNING:benchmark_name:status1,status2"
              if (message.startsWith('DUPLICATE_WARNING:')) {
                const parts = message.split(':');
                const benchmarkName = parts[1] || 'Benchmark';
                const statuses = parts[2] || 'unknown';

                // Show interactive toast with "Add Anyway" and "Cancel" options
                toast.warning(
                  `${benchmarkName} is already in the queue (${statuses}). Add anyway?`,
                  undefined,
                  [
                    {
                      label: 'Add Anyway',
                      onClick: () => {
                        void (async () => {
                          try {
                            // Recalculate benchmark IDs for the retry
                            const idsToQueue = Array.from(selectedBenchmarks)
                              .map(name => benchmarks.find((b: Benchmark) => b.name === name)?.id)
                              .filter((id): id is number => id !== undefined);

                            const queuedJobs = await queueBenchmarks(idsToQueue, true);
                            toast.success(
                              `${queuedJobs.length.toString()} benchmark${queuedJobs.length === 1 ? '' : 's'} added to queue (duplicates allowed)`,
                            );
                            selectedBenchmarks.clear();
                          } catch (err) {
                            toast.error(`Failed to queue benchmarks: ${String(err)}`);
                          }
                        })();
                      },
                    },
                    {
                      label: 'Cancel',
                      onClick: () => {
                        // Just dismiss the toast
                      },
                    },
                  ],
                );
              } else {
                // Other errors (including "prevent" mode)
                toast.error(`Failed to queue benchmarks: ${message}`);
              }
            }
          })();
        }
      },
      description: 'Queue selected benchmarks',
    });

    // Escape key: Clear selection (Subtask 3.3)
    registerShortcut({
      key: 'Escape',
      action: () => {
        selectedBenchmarks.clear();
        lastClickedIndex = null;
      },
      description: 'Clear selection',
    });
  });

  onDestroy(() => {
    // H1 fix: Remove document click listener
    document.removeEventListener('click', handleDocumentClick);
    unregisterShortcut(' ');
    unregisterShortcut('q');
    unregisterShortcut('Escape');
  });
</script>

<div class="flex-1 flex flex-col min-h-0">
  <div class="p-4 border-b border-white/5 flex items-center justify-between bg-slate-800/30">
    <div>
      <h2 class="font-semibold text-white">Benchmarks</h2>
      {#if selectionSummary}
        <p class="text-xs text-slate-400 mt-0.5" aria-live="polite">{selectionSummary}</p>
      {/if}
    </div>
    <div class="flex gap-2">
      <button
        onclick={onadd}
        disabled={!activeProject}
        aria-label="Add benchmark file"
        title={activeProject ? 'Add .py file' : 'Select a project first'}
        class="p-1.5 rounded-sm hover:bg-white/5 text-slate-400 hover:text-white transition-colors disabled:opacity-30 disabled:cursor-not-allowed"
      >
        <svg class="w-4 h-4" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"
          ><path d="M12 5v14M5 12h14" /></svg
        >
      </button>
      <button
        onclick={onrefresh}
        disabled={!activeProject}
        aria-label="Refresh Benchmarks"
        class="p-1.5 rounded-sm hover:bg-white/5 text-slate-400 hover:text-white transition-colors disabled:opacity-30 disabled:cursor-not-allowed"
      >
        <svg class="w-4 h-4" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"
          ><path d="M21 12a9 9 0 1 1-9-9c2.52 0 4.93 1 6.74 2.74L21 8" /><path
            d="M21 3v5h-5"
          /></svg
        >
      </button>
    </div>
  </div>

  {#if !activeProject}
    <div class="flex-1 flex items-center justify-center text-slate-500 p-4 text-center">
      <div>
        <svg
          class="w-12 h-12 mx-auto mb-3 opacity-30"
          viewBox="0 0 24 24"
          fill="none"
          stroke="currentColor"
          stroke-width="1"
        >
          <path d="M22 19a2 2 0 0 1-2 2H4a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h5l2 3h9a2 2 0 0 1 2 2z" />
        </svg>
        <p class="text-sm">Sélectionnez ou créez un projet</p>
      </div>
    </div>
  {:else}
    <div class="p-3 border-b border-white/5 bg-slate-800/20">
      <button
        onclick={ontoggleall}
        disabled={benchmarks.length === 0}
        class="text-xs font-medium text-slate-400 hover:text-white uppercase tracking-wider flex items-center gap-2 disabled:opacity-30"
      >
        <div
          class={`w-4 h-4 rounded-sm border ${selectedBenchmarks.size === benchmarks.length && benchmarks.length > 0 ? 'bg-blue-500 border-blue-500' : 'border-slate-600'} flex items-center justify-center transition-colors`}
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
        {selectedBenchmarks.size === benchmarks.length && benchmarks.length > 0
          ? 'Deselect All'
          : 'Select All'}
      </button>
    </div>

    <div
      class="overflow-y-auto flex-1 p-2 space-y-1 custom-scrollbar"
      bind:this={listContainerRef}
      role="listbox"
      aria-label="Benchmark list"
      aria-multiselectable="true"
    >
      {#if benchmarkError}
        <div
          class="mb-2 p-2 bg-red-500/10 border border-red-500/20 rounded-lg text-red-400 text-xs flex items-center justify-between"
        >
          <span>{benchmarkError}</span>
          <button onclick={() => (benchmarkError = null)} class="hover:text-white">✕</button>
        </div>
      {/if}
      {#if benchmarks.length === 0}
        <div class="text-center text-slate-500 py-8 text-sm">
          <p>Aucun benchmark</p>
          <p class="text-xs mt-1">Cliquez sur + pour ajouter un fichier .py</p>
        </div>
      {:else}
        {#each benchmarks as bench, index (bench.id)}
          <div
            class={`flex items-center gap-3 p-3 rounded-lg transition-all border cursor-pointer outline-none ${focusedBenchmark?.path === bench.path ? 'ring-2 ring-blue-500 ring-offset-2 ring-offset-slate-900' : ''} ${selectedBenchmarks.has(bench.name) ? 'bg-blue-500/10 border-blue-500/50' : 'hover:bg-white/5 border-transparent'} focus-visible:ring-2 focus-visible:ring-blue-500 focus-visible:ring-offset-2 focus-visible:ring-offset-slate-900`}
            role="option"
            tabindex="0"
            aria-selected={selectedBenchmarks.has(bench.name)}
            data-benchmark-index={index}
            onclick={(e: MouseEvent) => handleBenchmarkClick(e, bench, index)}
            onkeydown={(e: KeyboardEvent) => handleBenchmarkKeydown(e, bench, index)}
          >
            <div
              class={`w-4 h-4 rounded-sm border flex items-center justify-center transition-colors ${selectedBenchmarks.has(bench.name) ? 'bg-blue-500 border-blue-500' : 'border-slate-600 group-hover:border-blue-400'}`}
              role="checkbox"
              aria-checked={selectedBenchmarks.has(bench.name)}
              aria-label={`Select ${bench.name}`}
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
              class={`text-sm flex-1 truncate ${selectedBenchmarks.has(bench.name) ? 'text-white font-medium' : 'text-slate-300'}`}
              title={bench.path}>{bench.name}</span
            >
            <button
              onclick={(e: MouseEvent) => {
                e.stopPropagation();
                onfocus(bench);
              }}
              class={`p-1 rounded-sm transition-colors ${focusedBenchmark?.path === bench.path ? 'bg-purple-500/30 text-purple-300' : 'hover:bg-white/10 text-slate-500 hover:text-slate-300'}`}
              title="Analyze dependencies"
              aria-label={`Analyze ${bench.name} dependencies`}
            >
              <svg
                class="w-3.5 h-3.5"
                viewBox="0 0 24 24"
                fill="none"
                stroke="currentColor"
                stroke-width="2"
              >
                <circle cx="11" cy="11" r="8" />
                <path d="m21 21-4.35-4.35" />
              </svg>
            </button>
            <button
              onclick={(e: MouseEvent) => {
                e.stopPropagation();
                onremove(bench);
              }}
              class="p-1 rounded-sm hover:bg-red-500/20 text-slate-500 hover:text-red-400 transition-colors"
              title="Remove from project"
              aria-label={`Remove ${bench.name} from project`}
            >
              <svg
                class="w-3.5 h-3.5"
                viewBox="0 0 24 24"
                fill="none"
                stroke="currentColor"
                stroke-width="2"
              >
                <path d="M18 6L6 18M6 6l12 12" />
              </svg>
            </button>
          </div>
        {/each}
      {/if}
    </div>
  {/if}

  <div class="p-4 border-t border-white/5 bg-slate-800/30">
    <button
      onclick={onrun}
      disabled={selectedBenchmarks.size === 0 || isRunning || !activeProject}
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
        <svg class="w-4 h-4" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"
          ><polygon points="5 3 19 12 5 21 5 3"></polygon></svg
        >
        Run {selectedBenchmarks.size} Benchmarks
      {/if}
    </button>
  </div>
</div>
