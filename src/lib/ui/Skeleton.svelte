<script lang="ts">
  interface Props {
    variant?: 'text' | 'circle' | 'rect';
    width?: string;
    height?: string;
    class?: string;
  }

  const { variant = 'text', width, height, class: className = '' }: Props = $props();

  const variantClasses = {
    text: 'rounded',
    circle: 'rounded-full',
    rect: 'rounded-md',
  };

  const defaultDimensions = {
    text: { width: 'w-full', height: 'h-4' },
    circle: { width: 'w-12', height: 'h-12' },
    rect: { width: 'w-full', height: 'h-32' },
  };

  // Determine width and height classes or inline styles
  const style = $derived(
    [
      width !== undefined && !width.startsWith('w-') ? `width: ${width}` : '',
      height !== undefined && !height.startsWith('h-') ? `height: ${height}` : '',
    ]
      .filter(Boolean)
      .join('; '),
  );

  const finalWidth = $derived(
    (width?.startsWith('w-') ?? false)
      ? width
      : width === undefined || width === ''
        ? defaultDimensions[variant].width
        : '',
  );

  const finalHeight = $derived(
    (height?.startsWith('h-') ?? false)
      ? height
      : height === undefined || height === ''
        ? defaultDimensions[variant].height
        : '',
  );
</script>

<div
  class="animate-pulse bg-slate-800/50 {variantClasses[
    variant
  ]} {finalWidth} {finalHeight} {className}"
  {style}
></div>
