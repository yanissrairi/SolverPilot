<script lang="ts">
  import type { Job } from '../../types';

  const { history, selectedHistoryJob, onselect, onrefresh } = $props<{
    history: Job[];
    selectedHistoryJob: Job | null;
    onselect: (job: Job) => void;
    onrefresh: () => void;
  }>();

  const formattedTime = (seconds: number) => {
    const m = Math.floor(seconds / 60);
    const s = Math.floor(seconds % 60);
    return `${m.toString().padStart(2, '0')}:${s.toString().padStart(2, '0')}`;
  };
</script>

<div class="flex-1 flex flex-col min-h-0">
  <div class="p-3 border-b border-white/5 bg-slate-800/30 flex justify-between items-center">
    <h3 class="text-sm font-medium text-slate-300">Recent History</h3>
    <button onclick={onrefresh} class="text-xs text-slate-500 hover:text-white transition-colors"
      >Refresh</button
    >
  </div>
  <div class="flex-1 overflow-y-auto p-0 custom-scrollbar">
    <table class="w-full text-left text-sm text-slate-400">
      <thead
        class="bg-slate-900/50 text-xs uppercase font-semibold text-slate-500 sticky top-0 backdrop-blur-xs"
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
            onclick={() => onselect(job)}
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
                  (new Date(job.finished_at).getTime() - new Date(job.started_at).getTime()) / 1000,
                )}
              {:else}
                -
              {/if}
            </td>
            <td class="px-4 py-2 text-xs">
              {job.finished_at !== null ? new Date(job.finished_at).toLocaleTimeString() : '-'}
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
