<script lang="ts">
  import type { Snippet } from 'svelte';

  interface Props {
    text: string;
    position?: 'top' | 'bottom' | 'left' | 'right';
    children: Snippet;
    class?: string;
  }

  const { text, position = 'top', children, class: className = '' }: Props = $props();

  const positionClasses = {
    top: 'bottom-full left-1/2 -translate-x-1/2 mb-2',
    bottom: 'top-full left-1/2 -translate-x-1/2 mt-2',
    left: 'right-full top-1/2 -translate-y-1/2 mr-2',
    right: 'left-full top-1/2 -translate-y-1/2 ml-2',
  };

  const arrowClasses = {
    top: 'top-full left-1/2 -translate-x-1/2 border-t-slate-800',
    bottom: 'bottom-full left-1/2 -translate-x-1/2 border-b-slate-800',
    left: 'left-full top-1/2 -translate-y-1/2 border-l-slate-800',
    right: 'right-full top-1/2 -translate-y-1/2 border-r-slate-800',
  };
</script>

<div class="group relative inline-block {className}">
  {@render children()}

  <div
    class="pointer-events-none absolute z-50 opacity-0 transition-opacity duration-200 group-hover:opacity-100 {positionClasses[
      position
    ]}"
  >
    <div
      class="whitespace-nowrap rounded bg-slate-800 px-2 py-1 text-xs font-medium text-slate-200 shadow-lg border border-slate-700/50 backdrop-blur-sm"
    >
      {text}
    </div>
    <!-- Arrow -->
    <div class="absolute h-0 w-0 border-4 border-transparent {arrowClasses[position]}"></div>
  </div>
</div>
