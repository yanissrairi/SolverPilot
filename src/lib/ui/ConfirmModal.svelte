<script lang="ts">
  import { trapFocus } from '../utils/focus-trap';
  import Button from './Button.svelte';

  interface Props {
    open?: boolean;
    title: string;
    message: string;
    confirmText?: string;
    cancelText?: string;
    variant?: 'danger' | 'primary';
    loading?: boolean;
    onConfirm: () => void;
    onCancel: () => void;
  }

  const {
    open = false,
    title,
    message,
    confirmText = 'Confirm',
    cancelText = 'Cancel',
    variant = 'danger',
    loading = false,
    onConfirm,
    onCancel,
  }: Props = $props();

  function handleKeydown(event: KeyboardEvent) {
    if (event.key === 'Escape' && !loading) {
      onCancel();
      event.stopPropagation();
    }
  }

  function handleBackdropClick() {
    if (!loading) {
      onCancel();
    }
  }
</script>

{#if open}
  <!-- Backdrop -->
  <div
    class="fixed inset-0 z-50 flex items-center justify-center bg-slate-950/80 backdrop-blur-xs p-4"
    role="dialog"
    aria-modal="true"
    aria-labelledby="confirm-modal-title"
  >
    <!-- Overlay click handler -->
    <div class="absolute inset-0" onclick={handleBackdropClick} role="presentation"></div>

    <!-- Modal Panel -->
    <!-- svelte-ignore a11y_no_noninteractive_element_interactions -->
    <div
      class="glass-panel w-full max-w-md relative flex flex-col shadow-2xl"
      use:trapFocus
      onkeydown={handleKeydown}
      tabindex="-1"
      role="document"
    >
      <!-- Header -->
      <div class="flex items-center justify-between p-4 border-b border-white/10 bg-white/5">
        <h2 id="confirm-modal-title" class="text-lg font-semibold text-slate-100 tracking-wide">
          {title}
        </h2>
      </div>

      <!-- Body -->
      <div class="p-6 text-sm text-slate-300">
        {message}
      </div>

      <!-- Footer -->
      <div
        class="p-4 border-t border-white/10 bg-slate-900/40 flex justify-end gap-3 rounded-b-2xl"
      >
        <Button {variant} onclick={onCancel} disabled={loading}>
          {cancelText}
        </Button>
        <Button variant={variant === 'danger' ? 'danger' : 'primary'} onclick={onConfirm} {loading}>
          {confirmText}
        </Button>
      </div>
    </div>
  </div>
{/if}
