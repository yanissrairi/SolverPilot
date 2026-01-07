<script lang="ts">
  import { toast } from '../stores/toast.svelte.ts';
  import ToastItem from './Toast.svelte';
  import { flip } from 'svelte/animate';
  import { fly } from 'svelte/transition';
</script>

<div
  aria-live="assertive"
  class="pointer-events-none fixed inset-0 z-50 flex flex-col items-end gap-2 px-4 py-6 sm:items-end sm:p-6"
>
  {#each toast.toasts as item (item.id)}
    <div
      animate:flip={{ duration: 300 }}
      in:fly={{ x: 200, duration: 300 }}
      out:fly={{ x: 200, duration: 300, opacity: 0 }}
      class="flex w-full flex-col items-center space-y-4 sm:items-end"
    >
      <ToastItem {item} />
    </div>
  {/each}
</div>
