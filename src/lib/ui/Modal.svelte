<script lang="ts">
  import { trapFocus } from '../utils/focus-trap';
  import type { Snippet } from 'svelte';

  let {
    open = $bindable(false),
    title,
    size = 'md',
    closable = true,
    children,
    footer,
  }: {
    open?: boolean;
    title: string;
    size?: 'sm' | 'md' | 'lg' | 'xl' | 'full';
    closable?: boolean;
    children?: Snippet;
    footer?: Snippet;
  } = $props();

  function close() {
    if (closable) {
      open = false;
    }
  }

  function handleKeydown(event: KeyboardEvent) {
    if (event.key === 'Escape' && closable) {
      close();
      event.stopPropagation();
    }
  }

  const sizeClasses = {
    sm: 'max-w-sm',
    md: 'max-w-md',
    lg: 'max-w-lg',
    xl: 'max-w-xl',
    full: 'max-w-[calc(100vw-2rem)]',
  };
</script>

{#if open}
  <!-- Backdrop -->
  <div
    class="fixed inset-0 z-50 flex items-center justify-center bg-slate-950/80 backdrop-blur-xs p-4"
    role="dialog"
    aria-modal="true"
    aria-labelledby="modal-title"
  >
    <!-- Overlay click handler -->
    <div class="absolute inset-0" onclick={close} role="presentation"></div>

    <!-- Modal Panel -->
    <!-- svelte-ignore a11y_no_noninteractive_element_interactions -->
    <div
      class="glass-panel w-full {sizeClasses[size]} relative flex flex-col shadow-2xl max-h-[90vh]"
      use:trapFocus
      onkeydown={handleKeydown}
      tabindex="-1"
      role="document"
    >
      <!-- Header -->
      <div class="flex items-center justify-between p-4 border-b border-white/10 bg-white/5">
        <h2 id="modal-title" class="text-lg font-semibold text-slate-100 tracking-wide">
          {title}
        </h2>
        {#if closable}
          <button
            onclick={close}
            class="text-slate-400 hover:text-white transition-colors p-1.5 rounded-lg hover:bg-white/10 focus:outline-hidden focus:ring-2 focus:ring-white/20"
            aria-label="Fermer"
          >
            <svg
              xmlns="http://www.w3.org/2000/svg"
              class="h-5 w-5"
              viewBox="0 0 20 20"
              fill="currentColor"
            >
              <path
                fill-rule="evenodd"
                d="M4.293 4.293a1 1 0 011.414 0L10 8.586l4.293-4.293a1 1 0 111.414 1.414L11.414 10l4.293 4.293a1 1 0 01-1.414 1.414L10 11.414l-4.293 4.293a1 1 0 01-1.414-1.414L8.586 10 4.293 5.707a1 1 0 010-1.414z"
                clip-rule="evenodd"
              />
            </svg>
          </button>
        {/if}
      </div>

      <!-- Body -->
      <div class="p-6 overflow-y-auto text-slate-300">
        {@render children?.()}
      </div>

      <!-- Footer -->
      {#if footer}
        <div
          class="p-4 border-t border-white/10 bg-slate-900/40 flex justify-end gap-3 rounded-b-2xl"
        >
          {@render footer()}
        </div>
      {/if}
    </div>
  </div>
{/if}
