<script lang="ts">
  import { toast, type Toast } from '../stores/toast.svelte.ts';

  interface Props {
    item: Toast;
  }

  const { item }: Props = $props();

  const variantClasses = {
    success: 'border-l-4 border-emerald-500 text-emerald-100',
    error: 'border-l-4 border-red-500 text-red-100',
    warning: 'border-l-4 border-amber-500 text-amber-100',
    info: 'border-l-4 border-blue-500 text-blue-100',
  };

  const iconPaths = {
    success: 'M9 12l2 2 4-4m6 2a9 9 0 11-18 0 9 9 0 0118 0z',
    error: 'M10 14l2-2m0 0l2-2m-2 2l-2-2m2 2l2 2m7-2a9 9 0 11-18 0 9 9 0 0118 0z',
    warning:
      'M12 9v2m0 4h.01m-6.938 4h13.856c1.54 0 2.502-1.667 1.732-3L13.732 4c-.77-1.333-2.694-1.333-3.464 0L3.34 16c-.77 1.333.192 3 1.732 3z',
    info: 'M13 16h-1v-4h-1m1-4h.01M21 12a9 9 0 11-18 0 9 9 0 0118 0z',
  };

  const colors = {
    success: 'text-emerald-500',
    error: 'text-red-500',
    warning: 'text-amber-500',
    info: 'text-blue-500',
  };
</script>

<div
  class="pointer-events-auto w-full max-w-sm overflow-hidden rounded-lg bg-slate-800 shadow-lg ring-1 ring-black ring-opacity-5 transition-all {variantClasses[
    item.type
  ]}"
  role="alert"
>
  <div class="p-4">
    <div class="flex items-start">
      <div class="shrink-0">
        <svg
          class="h-6 w-6 {colors[item.type]}"
          fill="none"
          viewBox="0 0 24 24"
          stroke-width="1.5"
          stroke="currentColor"
          aria-hidden="true"
        >
          <path stroke-linecap="round" stroke-linejoin="round" d={iconPaths[item.type]} />
        </svg>
      </div>
      <div class="ml-3 w-0 flex-1 pt-0.5">
        <p class="text-sm font-medium">{item.message}</p>
        {#if item.actions && item.actions.length > 0}
          <div class="mt-3 flex gap-2">
            {#each item.actions as action, idx (idx)}
              <button
                type="button"
                class="text-sm font-medium px-3 py-1.5 rounded border transition-colors {item.type ===
                'warning'
                  ? 'border-amber-600/50 bg-amber-500/10 text-amber-100 hover:bg-amber-500/20'
                  : 'border-slate-600 bg-slate-700/50 text-slate-200 hover:bg-slate-700'}"
                onclick={() => {
                  action.onClick();
                  toast.remove(item.id);
                }}
              >
                {action.label}
              </button>
            {/each}
          </div>
        {/if}
      </div>
      <div class="ml-4 flex shrink-0">
        <button
          type="button"
          class="inline-flex rounded-md bg-slate-800 text-slate-400 hover:text-slate-200 focus:outline-hidden focus:ring-2 focus:ring-slate-500 focus:ring-offset-2"
          onclick={() => toast.remove(item.id)}
        >
          <span class="sr-only">Close</span>
          <svg class="h-5 w-5" viewBox="0 0 20 20" fill="currentColor" aria-hidden="true">
            <path
              d="M6.28 5.22a.75.75 0 00-1.06 1.06L8.94 10l-3.72 3.72a.75.75 0 101.06 1.06L10 11.06l3.72 3.72a.75.75 0 101.06-1.06L11.06 10l3.72-3.72a.75.75 0 00-1.06-1.06L10 8.94 6.28 5.22z"
            />
          </svg>
        </button>
      </div>
    </div>
  </div>
</div>
