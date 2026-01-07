<script lang="ts">
  import type { Snippet } from 'svelte';

  interface Props {
    position: 'left' | 'right';
    minWidth?: number;
    maxWidth?: number;
    width?: number;
    children?: Snippet;
  }

  let {
    position,
    minWidth = 200,
    maxWidth = 600,
    width = $bindable(),
    children
  }: Props = $props();

  let isResizing = $state(false);

  function startResize(event: MouseEvent) {
    event.preventDefault();
    isResizing = true;
    
    window.addEventListener('mousemove', onMouseMove);
    window.addEventListener('mouseup', stopResize);
    // Add a class to body to prevent text selection and ensure cursor consistency
    document.body.style.cursor = 'col-resize';
    document.body.style.userSelect = 'none';
  }

  function stopResize() {
    isResizing = false;
    window.removeEventListener('mousemove', onMouseMove);
    window.removeEventListener('mouseup', stopResize);
    document.body.style.cursor = '';
    document.body.style.userSelect = '';
  }

  function onMouseMove(event: MouseEvent) {
    if (!isResizing) return;

    if (position === 'left') {
      const newWidth = event.clientX;
      if (newWidth >= minWidth && newWidth <= maxWidth) {
        width = newWidth;
      }
    } else {
      const newWidth = window.innerWidth - event.clientX;
      if (newWidth >= minWidth && newWidth <= maxWidth) {
        width = newWidth;
      }
    }
  }
</script>

<div 
  class="glass-panel relative flex flex-col h-full overflow-hidden transition-all duration-75 ease-out"
  style="width: {width}px;"
  class:transition-none={isResizing}
>
  {@render children?.()}

  <!-- Drag Handle -->
  <!-- svelte-ignore a11y_no_static_element_interactions -->
  <div
    class="absolute top-0 bottom-0 w-1 cursor-col-resize hover:bg-blue-500/50 active:bg-blue-500 transition-colors z-50 group"
    class:right-0={position === 'left'}
    class:left-0={position === 'right'}
    onmousedown={startResize}
  >
    <!-- Visual indicator -->
    <div class="absolute top-1/2 -translate-y-1/2 w-1 h-8 bg-slate-600/50 rounded-full group-hover:bg-blue-400/50 transition-colors mx-auto left-0 right-0"></div>
  </div>
</div>
