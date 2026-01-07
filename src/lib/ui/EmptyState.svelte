<script lang="ts">
  import type { Snippet } from 'svelte';

  interface Props {
    icon?: Snippet;
    title: string;
    description?: string;
    action?: Snippet;
    class?: string;
  }

  const { icon, title, description, action, class: className = '' }: Props = $props();
</script>

<div class="flex flex-col items-center justify-center p-8 text-center {className}">
  {#if icon}
    <div
      class="mb-4 text-slate-600 dark:text-slate-500 [&>svg]:w-12 [&>svg]:h-12 [&>svg]:stroke-[1.5]"
    >
      {@render icon()}
    </div>
  {/if}

  <h3 class="text-lg font-medium text-slate-200 mb-1">
    {title}
  </h3>

  {#if description}
    <p class="text-sm text-slate-400 max-w-sm mb-6">
      {description}
    </p>
  {/if}

  {#if action}
    <div>
      {@render action()}
    </div>
  {/if}
</div>
