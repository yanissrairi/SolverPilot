<script lang="ts">
  import type { SvelteSet } from 'svelte/reactivity';
  import type { Benchmark, Project } from '../../types';

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
    onrun
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
</script>

<div class="flex-1 flex flex-col min-h-0">
  <div class="p-4 border-b border-white/5 flex items-center justify-between bg-slate-800/30">
    <h2 class="font-semibold text-white">Benchmarks</h2>
    <div class="flex gap-2">
      <button
        onclick={onadd}
        disabled={!activeProject}
        aria-label="Add benchmark file"
        title={activeProject ? 'Add .py file' : 'Select a project first'}
        class="p-1.5 rounded hover:bg-white/5 text-slate-400 hover:text-white transition-colors disabled:opacity-30 disabled:cursor-not-allowed"
      >
        <svg
          class="w-4 h-4"
          viewBox="0 0 24 24"
          fill="none"
          stroke="currentColor"
          stroke-width="2"><path d="M12 5v14M5 12h14" /></svg
        >
      </button>
      <button
        onclick={onrefresh}
        disabled={!activeProject}
        aria-label="Refresh Benchmarks"
        class="p-1.5 rounded hover:bg-white/5 text-slate-400 hover:text-white transition-colors disabled:opacity-30 disabled:cursor-not-allowed"
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
          <path
            d="M22 19a2 2 0 0 1-2 2H4a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h5l2 3h9a2 2 0 0 1 2 2z"
          />
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
        {selectedBenchmarks.size === benchmarks.length && benchmarks.length > 0
          ? 'Deselect All'
          : 'Select All'}
      </button>
    </div>

    <div class="overflow-y-auto flex-1 p-2 space-y-1 custom-scrollbar">
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
        {#each benchmarks as bench (bench.id)}
          <!-- svelte-ignore a11y_click_events_have_key_events -->
          <!-- svelte-ignore a11y_no_static_element_interactions -->
          <div
            class={`flex items-center gap-3 p-3 rounded-lg transition-all border ${focusedBenchmark?.path === bench.path ? 'ring-2 ring-purple-500/50' : ''} ${selectedBenchmarks.has(bench.name) ? 'bg-blue-500/10 border-blue-500/50' : 'hover:bg-white/5 border-transparent'}`}
          >
            <!-- svelte-ignore a11y_click_events_have_key_events -->
            <!-- svelte-ignore a11y_no_static_element_interactions -->
            <div
              class={`w-4 h-4 rounded border flex items-center justify-center transition-colors cursor-pointer ${selectedBenchmarks.has(bench.name) ? 'bg-blue-500 border-blue-500' : 'border-slate-600 hover:border-blue-400'}`}
              onclick={() => ontoggle(bench.name)}
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
            <!-- svelte-ignore a11y_click_events_have_key_events -->
            <!-- svelte-ignore a11y_no_static_element_interactions -->
            <span
              class={`text-sm flex-1 cursor-pointer truncate ${selectedBenchmarks.has(bench.name) ? 'text-white font-medium' : 'text-slate-300'}`}
              onclick={() => onfocus(bench)}
              title={bench.path}>{bench.name}</span
            >
            <button
              onclick={() => onfocus(bench)}
              class={`p-1 rounded transition-colors ${focusedBenchmark?.path === bench.path ? 'bg-purple-500/30 text-purple-300' : 'hover:bg-white/10 text-slate-500 hover:text-slate-300'}`}
              title="Analyze dependencies"
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
              onclick={() => onremove(bench)}
              class="p-1 rounded hover:bg-red-500/20 text-slate-500 hover:text-red-400 transition-colors"
              title="Remove from project"
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