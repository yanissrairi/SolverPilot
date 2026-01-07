<script lang="ts">
  import type { Snippet } from 'svelte';
  import ProjectSelector from '../ProjectSelector.svelte';
  import type { Project } from '../types';

  interface Props {
      activeProject: Project | null;
      onProjectChange: (project: Project | null) => void;
      children?: Snippet;
  }

  const { activeProject, onProjectChange, children }: Props = $props();
</script>

<header class="glass-header h-16 flex items-center justify-between px-4 shrink-0 z-40">
  <!-- Left: Logo/Title -->
  <div class="flex items-center gap-3">
    <div class="w-8 h-8 rounded-lg bg-gradient-to-br from-blue-500 to-indigo-600 flex items-center justify-center shadow-lg shadow-blue-500/20">
      <svg xmlns="http://www.w3.org/2000/svg" class="w-5 h-5 text-white" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5" stroke-linecap="round" stroke-linejoin="round">
        <path d="M12 2L2 7l10 5 10-5-10-5zM2 17l10 5 10-5M2 12l10 5 10-5"/>
      </svg>
    </div>
    <h1 class="text-lg font-bold bg-gradient-to-r from-white to-slate-400 bg-clip-text text-transparent">
      SolverPilot
    </h1>
  </div>

  <!-- Center: Project Selector -->
  <div class="flex-1 flex justify-center max-w-xl mx-auto">
    <ProjectSelector {activeProject} {onProjectChange} />
  </div>

  <!-- Right: SSH Status & Actions -->
  <div class="flex items-center gap-2">
      <!-- Placeholder for SSH Status -->
      <!-- <div class="flex items-center gap-2 px-3 py-1.5 rounded-full bg-slate-800/50 border border-slate-700/50 text-xs font-medium text-slate-400">
          <div class="w-2 h-2 rounded-full bg-slate-600"></div>
          SSH Offline
      </div> -->
      
      {@render children?.()}
  </div>
</header>
