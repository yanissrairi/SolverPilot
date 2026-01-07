<script lang="ts">
  import type { Snippet } from 'svelte';

  interface Props {
    title: string;
    variant?: 'primary' | 'secondary' | 'danger' | 'ghost';
    size?: 'sm' | 'md' | 'lg';
    disabled?: boolean;
    class?: string;
    children?: Snippet;
    onclick?: (event: MouseEvent) => void;
    [key: string]: unknown;
  }

  const {
    title,
    variant = 'ghost',
    size = 'md',
    disabled = false,
    class: className = '',
    children,
    onclick,
    ...rest
  }: Props = $props();

  const baseClasses =
    'inline-flex items-center justify-center rounded-lg transition-colors duration-200 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-offset-slate-900 disabled:opacity-50 disabled:pointer-events-none active:scale-95';

  const variantClasses = {
    primary: 'bg-blue-600 hover:bg-blue-500 text-white shadow-lg shadow-blue-900/20',
    secondary: 'bg-slate-800 hover:bg-slate-700 text-slate-200 border border-slate-700',
    danger: 'bg-red-600/80 hover:bg-red-500 text-white shadow-lg shadow-red-900/20',
    ghost: 'text-slate-400 hover:text-white hover:bg-white/5',
  };

  const sizeClasses = {
    sm: 'p-1.5 w-8 h-8',
    md: 'p-2 w-10 h-10',
    lg: 'p-3 w-12 h-12',
  };
</script>

<div class="relative group">
  <button
    type="button"
    class="{baseClasses} {variantClasses[variant]} {sizeClasses[size]} {className}"
    {disabled}
    {onclick}
    aria-label={title}
    {...rest}
  >
    {@render children?.()}
  </button>

  <div
    class="absolute bottom-full left-1/2 -translate-x-1/2 mb-2 px-2 py-1 text-xs font-medium text-white bg-slate-800 rounded shadow-lg border border-slate-700 opacity-0 group-hover:opacity-100 transition-opacity pointer-events-none whitespace-nowrap z-50"
  >
    {title}
    <div
      class="absolute top-full left-1/2 -translate-x-1/2 -mt-1 border-4 border-transparent border-t-slate-800"
    ></div>
  </div>
</div>
