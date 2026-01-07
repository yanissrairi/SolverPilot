<script lang="ts">
  import Spinner from './Spinner.svelte';
  import type { Snippet } from 'svelte';

  interface Props {
    variant?: 'primary' | 'secondary' | 'danger' | 'ghost';
    size?: 'sm' | 'md' | 'lg';
    loading?: boolean;
    disabled?: boolean;
    type?: 'button' | 'submit' | 'reset';
    class?: string;
    children?: Snippet;
    onclick?: (event: MouseEvent) => void;
    [key: string]: unknown;
  }

  const {
    variant = 'primary',
    size = 'md',
    loading = false,
    disabled = false,
    type = 'button',
    class: className = '',
    children,
    onclick,
    ...rest
  }: Props = $props();

  const baseClasses =
    'inline-flex items-center justify-center font-medium transition-all duration-200 focus:outline-hidden focus:ring-2 focus:ring-offset-2 focus:ring-offset-slate-900 rounded-lg disabled:opacity-50 disabled:pointer-events-none active:scale-95';

  const variantClasses = {
    primary:
      'bg-blue-600 hover:bg-blue-500 text-white shadow-lg shadow-blue-900/20 border border-transparent focus:ring-blue-500',
    secondary:
      'bg-slate-800 hover:bg-slate-700 text-slate-200 border border-slate-700 hover:border-slate-600 focus:ring-slate-500',
    danger:
      'bg-red-600/80 hover:bg-red-500 text-white shadow-lg shadow-red-900/20 border border-transparent focus:ring-red-500',
    ghost:
      'text-slate-400 hover:text-white hover:bg-white/5 border border-transparent focus:ring-slate-500',
  };

  const sizeClasses = {
    sm: 'px-3 py-1.5 text-sm',
    md: 'px-4 py-2 text-sm',
    lg: 'px-6 py-3 text-base',
  };
</script>

<button
  {type}
  class="{baseClasses} {variantClasses[variant]} {sizeClasses[size]} {className}"
  disabled={disabled || loading}
  {onclick}
  aria-disabled={disabled || loading}
  {...rest}
>
  {#if loading}
    <Spinner size="sm" class="mr-2" />
    <span>Loading...</span>
  {:else}
    {@render children?.()}
  {/if}
</button>
