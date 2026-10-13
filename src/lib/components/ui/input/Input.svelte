<script lang="ts">
  import { cn } from '../../../utils';

  interface Props {
    type?: string;
    value?: string;
    placeholder?: string;
    class?: string;
    disabled?: boolean;
    onkeydown?: (e: KeyboardEvent) => void;
    oninput?: (e: Event) => void;
    [key: string]: any;
  }

  let {
    type = 'text',
    value = $bindable(''),
    placeholder = '',
    class: className = '',
    disabled = false,
    onkeydown,
    oninput,
    ...restProps
  }: Props = $props();
</script>

<input
  {type}
  bind:value
  {placeholder}
  {disabled}
  {onkeydown}
  {oninput}
  class={cn(
    'flex h-8 w-full rounded bg-zinc-900/90 border border-zinc-700/80 px-3 py-1 text-xs text-zinc-100 placeholder:text-zinc-500 shadow-sm transition-colors focus:border-zed-accent focus:outline-none focus:ring-1 focus:ring-zed-accent disabled:cursor-not-allowed disabled:opacity-50 font-sans',
    className
  )}
  {...restProps}
/>
