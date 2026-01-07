<script lang="ts">
  import type { JobStatusResponse, Job } from '../../types';

  let {
    currentJobStatus,
    isRunning,
    selectedHistoryJob,
    autoScroll = $bindable(),
    onstop,
    onkill,
    onbacktolist,
  } = $props<{
    currentJobStatus: JobStatusResponse | null;
    isRunning: boolean;
    selectedHistoryJob: Job | null;
    autoScroll?: boolean;
    onstop: () => void;
    onkill: () => void;
    onbacktolist: () => void;
  }>();

  let logsContainer = $state<HTMLElement>();

  $effect(() => {
    // If logs container exists, auto-scroll is enabled, and there's new content
    // We can just try to scroll to bottom when component updates or deps change
    // Using currentJobStatus.logs or selectedHistoryJob.log_content as dependencies to trigger scroll
    if (logsContainer && autoScroll === true) {
      // Just accessing the logs to make it reactive
      void (currentJobStatus?.logs ?? selectedHistoryJob?.log_content);
      logsContainer.scrollTop = logsContainer.scrollHeight;
    }
  });

  const formattedTime = (seconds: number) => {
    const m = Math.floor(seconds / 60);
    const s = Math.floor(seconds % 60);
    return `${m.toString().padStart(2, '0')}:${s.toString().padStart(2, '0')}`;
  };

  const progressPercent = $derived(
    currentJobStatus
      ? (currentJobStatus.progress / (currentJobStatus.job?.progress_total ?? 1)) * 100
      : 0,
  );
</script>

<div class="flex-1 flex flex-col relative min-h-0">
  {#if selectedHistoryJob}
    <!-- Viewing History Job Logs -->
    <div class="p-4 border-b border-white/5 bg-slate-800/30 flex items-center justify-between">
      <div class="flex items-center gap-4">
        <button
          onclick={onbacktolist}
          class="p-1.5 rounded-sm hover:bg-white/10 text-slate-400 hover:text-white transition-colors"
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
          <span class="font-mono text-sm text-slate-400 bg-slate-800/50 px-2 py-1 rounded-sm">
            {formattedTime(
              (new Date(selectedHistoryJob.finished_at).getTime() -
                new Date(selectedHistoryJob.started_at).getTime()) /
                1000,
            )}
          </span>
        {/if}
        <span class="text-xs text-slate-500 bg-slate-800/30 px-2 py-1 rounded-sm"> History </span>
      </div>
    </div>

    <!-- Terminal/Logs for history -->
    <div
      class="flex-1 bg-black/40 p-4 font-mono text-xs md:text-sm text-slate-300 overflow-y-auto custom-scrollbar"
      bind:this={logsContainer}
    >
      <pre class="whitespace-pre-wrap">{selectedHistoryJob.log_content ||
          'No logs available for this job.'}</pre>
    </div>
  {:else if !currentJobStatus?.job}
    <div class="absolute inset-0 flex flex-col items-center justify-center text-slate-500 gap-4">
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
    <div class="p-4 border-b border-white/5 bg-slate-800/30 flex items-center justify-between">
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
        <span class="font-mono text-sm text-blue-300 bg-blue-900/30 px-2 py-1 rounded-sm">
          {formattedTime(currentJobStatus.elapsed_seconds)}
        </span>
        {#if isRunning}
          <button
            onclick={onstop}
            class="btn-glass hover:bg-yellow-500/20 text-yellow-300 border-yellow-500/30 text-xs py-1.5"
            >Stop</button
          >
          <button onclick={onkill} class="btn-glass btn-danger text-xs py-1.5">Kill</button>
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
      <pre class="whitespace-pre-wrap">{currentJobStatus.logs || 'Waiting for output...'}</pre>

      <!-- Auto-scroll toggle overlay -->
      <button
        onclick={() => (autoScroll = autoScroll !== true)}
        class={`absolute bottom-4 right-4 p-2 rounded-lg backdrop-blur-sm border transition-all ${autoScroll === true ? 'bg-blue-500/20 text-blue-400 border-blue-500/30' : 'bg-slate-800/50 text-slate-400 border-white/10'}`}
        title="Toggle Auto-scroll"
      >
        <svg class="w-4 h-4" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"
          ><path d="M12 5v14M19 12l-7 7-7-7" /></svg
        >
      </button>
    </div>
  {/if}
</div>
