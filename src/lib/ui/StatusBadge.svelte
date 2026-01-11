<script lang="ts">
  interface Props {
    status: 'pending' | 'running' | 'completed' | 'failed' | 'killed';
  }

  const { status }: Props = $props();

  const statusConfig = {
    pending: {
      color: 'bg-blue-500/20 text-blue-400 border-blue-500/30',
      icon: '⏳',
      label: 'Pending',
    },
    running: {
      color: 'bg-green-500/20 text-green-400 border-green-500/30',
      icon: '▶️',
      label: 'Running',
    },
    completed: {
      color: 'bg-gray-500/20 text-gray-400 border-gray-500/30',
      icon: '✓',
      label: 'Completed',
    },
    failed: {
      color: 'bg-red-500/20 text-red-400 border-red-500/30',
      icon: '✗',
      label: 'Failed',
    },
    killed: {
      color: 'bg-orange-500/20 text-orange-400 border-orange-500/30',
      icon: '⊗',
      label: 'Killed',
    },
  };

  const config = $derived(statusConfig[status]);
</script>

<span
  class="inline-flex items-center gap-1 px-3 py-1 rounded-full text-xs font-medium border {config.color}"
  aria-label={config.label}
  role="status"
>
  <span aria-hidden="true">{config.icon}</span>
  <span>{config.label}</span>
</span>
