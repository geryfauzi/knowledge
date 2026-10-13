<script lang="ts">
  import type { Snippet } from 'svelte';
  import { cn } from '../../../utils';

  interface Props {
    variant?: 'default' | 'secondary' | 'outline' | 'ghost' | 'destructive';
    size?: 'default' | 'sm' | 'lg' | 'icon';
    class?: string;
    type?: 'button' | 'submit' | 'reset';
    disabled?: boolean;
    onclick?: (e: MouseEvent) => void;
    children?: Snippet;
    [key: string]: any;
  }

  let {
    variant = 'default',
    size = 'default',
    class: className = '',
    type = 'button',
    disabled = false,
    onclick,
    children,
    ...restProps
  }: Props = $props();

  const variantStyles = {
    default: 'bg-zinc-800 text-zinc-100 hover:bg-zinc-700 border border-zinc-700/80 shadow-sm',
    secondary: 'bg-zinc-900 text-zinc-300 hover:bg-zinc-800 border border-zinc-800',
    outline: 'bg-transparent text-zinc-300 border border-zinc-700 hover:bg-zinc-800/80 hover:text-zinc-100',
    ghost: 'bg-transparent text-zinc-400 hover:bg-zinc-800 hover:text-zinc-200',
    destructive: 'bg-red-950/50 text-red-400 border border-red-900/60 hover:bg-red-900/60'
  };

  const sizeStyles = {
    default: 'h-8 px-3 text-xs',
    sm: 'h-7 px-2.5 text-[11px]',
    lg: 'h-9 px-4 text-sm',
    icon: 'h-7 w-7 p-0'
  };
</script>

<button
  {type}
  {disabled}
  {onclick}
  class={cn(
    'inline-flex items-center justify-center rounded font-medium transition-colors select-none focus:outline-none focus:ring-1 focus:ring-zed-accent disabled:opacity-50 disabled:pointer-events-none',
    variantStyles[variant],
    sizeStyles[size],
    className
  )}
  {...restProps}
>
  {#if children}
    {@render children()}
  {/if}
</button>
