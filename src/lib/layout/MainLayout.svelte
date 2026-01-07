<script lang="ts">
  import type { Snippet } from 'svelte';
  import Header from './Header.svelte';
  import ResizablePanel from './ResizablePanel.svelte';
  import { panelStore } from '../stores/panels.svelte.ts';
  import type { Project } from '../types';

  interface Props {
    activeProject: Project | null;
    onProjectChange: (project: Project | null) => void;
    headerChildren?: Snippet;
    leftPanel?: Snippet;
    middlePanel?: Snippet;
    rightPanel?: Snippet;
  }

  const {
    activeProject,
    onProjectChange,
    headerChildren,
    leftPanel,
    middlePanel,
    rightPanel,
  }: Props = $props();

  let isMobile = $state(false);
  let activeMobileTab = $state<'left' | 'middle' | 'right'>('middle');

  function checkMobile() {
    isMobile = window.innerWidth < 1024; // lg breakpoint
  }

  $effect(() => {
    checkMobile();
    window.addEventListener('resize', checkMobile);
    return () => window.removeEventListener('resize', checkMobile);
  });

  // Persist panel widths to localStorage
  $effect(() => {
    if (typeof localStorage !== 'undefined') {
      localStorage.setItem('panel-left-width', panelStore.leftWidth.toString());
      localStorage.setItem('panel-right-width', panelStore.rightWidth.toString());
    }
  });
</script>

<div
  class="h-screen flex flex-col overflow-hidden bg-linear-to-br from-slate-950 via-slate-900 to-black text-slate-200"
>
  <Header {activeProject} {onProjectChange}>
    {@render headerChildren?.()}
  </Header>

  <div class="flex-1 flex min-h-0 overflow-hidden relative">
    {#if isMobile}
      <!-- Mobile Layout: Tabs/Swiper style or stacked? Requested: "collapse les panneaux" -->
      <!-- Simple tab view for mobile to allow access to all 3 panels -->
      <div class="flex-1 flex flex-col min-h-0">
        <div class="flex-1 overflow-hidden relative">
          {#if activeMobileTab === 'left'}
            <div class="absolute inset-0 p-4 overflow-y-auto">
              {@render leftPanel?.()}
            </div>
          {:else if activeMobileTab === 'middle'}
            <div class="absolute inset-0 p-4 overflow-y-auto">
              {@render middlePanel?.()}
            </div>
          {:else}
            <div class="absolute inset-0 p-4 overflow-y-auto">
              {@render rightPanel?.()}
            </div>
          {/if}
        </div>

        <!-- Mobile Navigation -->
        <div class="glass-header border-t border-slate-700/50 flex justify-around p-2 shrink-0">
          <button
            class="p-2 rounded-lg {activeMobileTab === 'left'
              ? 'bg-blue-500/20 text-blue-400'
              : 'text-slate-400'}"
            onclick={() => (activeMobileTab = 'left')}
          >
            Benchmarks
          </button>
          <button
            class="p-2 rounded-lg {activeMobileTab === 'middle'
              ? 'bg-blue-500/20 text-blue-400'
              : 'text-slate-400'}"
            onclick={() => (activeMobileTab = 'middle')}
          >
            Jobs
          </button>
          <button
            class="p-2 rounded-lg {activeMobileTab === 'right'
              ? 'bg-blue-500/20 text-blue-400'
              : 'text-slate-400'}"
            onclick={() => (activeMobileTab = 'right')}
          >
            Deps
          </button>
        </div>
      </div>
    {:else}
      <!-- Desktop Layout -->

      <!-- LEFT PANEL -->
      <ResizablePanel
        position="left"
        bind:width={panelStore.leftWidth}
        minWidth={200}
        maxWidth={500}
      >
        <div class="h-full overflow-hidden flex flex-col p-4 pr-2">
          {@render leftPanel?.()}
        </div>
      </ResizablePanel>

      <!-- MIDDLE PANEL (Fluid) -->
      <div class="flex-1 min-w-0 h-full overflow-hidden p-4 px-2">
        {@render middlePanel?.()}
      </div>

      <!-- RIGHT PANEL (Dependencies - compact) -->
      <ResizablePanel
        position="right"
        bind:width={panelStore.rightWidth}
        minWidth={260}
        maxWidth={450}
      >
        <div class="h-full overflow-hidden flex flex-col p-4 pl-2">
          {@render rightPanel?.()}
        </div>
      </ResizablePanel>
    {/if}
  </div>
</div>
