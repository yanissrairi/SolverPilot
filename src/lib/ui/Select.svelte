<script lang="ts">
  interface Option {
    value: string;
    label: string;
  }

  interface Props {
    value: string;
    options: Option[];
    placeholder?: string;
    disabled?: boolean;
    class?: string;
    // eslint-disable-next-line @typescript-eslint/no-explicit-any
    [key: string]: any;
  }

   
  let {
    value,
    options,
    placeholder = 'Select an option',
    disabled = false,
    class: className = '',
    ...rest
  }: Props = $props();
</script>

<div class="relative {className}">
  <select
    bind:value
    {disabled}
    class="w-full appearance-none rounded-lg bg-slate-900/50 border border-slate-700/50 py-2 pl-3 pr-10 text-sm text-slate-200 shadow-sm focus:border-blue-500 focus:outline-none focus:ring-1 focus:ring-blue-500 disabled:cursor-not-allowed disabled:opacity-50 hover:bg-slate-800/50 transition-colors cursor-pointer"
    {...rest}
  >
    {#if placeholder}
      <option value="" disabled selected>{placeholder}</option>
    {/if}
    {#each options as option (option.value)}
      <option value={option.value}>{option.label}</option>
    {/each}
  </select>

  <div class="pointer-events-none absolute inset-y-0 right-0 flex items-center pr-2">
    <svg
      class="h-4 w-4 text-slate-400"
      fill="none"
      stroke="currentColor"
      viewBox="0 0 24 24"
    >
      <path
        stroke-linecap="round"
        stroke-linejoin="round"
        stroke-width="2"
        d="M19 9l-7 7-7-7"
      />
    </svg>
  </div>
</div>
