<script lang="ts">
  import type { Snippet } from 'svelte';
  import { X } from 'lucide-svelte';
  import { cn } from '../../../utils';

  interface Props {
    open: boolean;
    title?: string;
    description?: string;
    class?: string;
    onClose: () => void;
    children?: Snippet;
    footer?: Snippet;
  }

  let {
    open,
    title,
    description,
    class: className = '',
    onClose,
    children,
    footer
  }: Props = $props();

  function handleBackdropClick(e: MouseEvent) {
    if (e.target === e.currentTarget) {
      onClose();
    }
  }

  function handleKeyDown(e: KeyboardEvent) {
    if (e.key === 'Escape') {
      onClose();
    }
  }
</script>

{#if open}
  <div
    role="dialog"
    aria-modal="true"
    tabindex="-1"
    onclick={handleBackdropClick}
    onkeydown={handleKeyDown}
    class="fixed inset-0 z-50 flex items-center justify-center p-4 bg-black/60 backdrop-blur-[2px] animate-in fade-in duration-150"
  >
    <div
      class={cn(
        'relative w-full max-w-md bg-zed-bg-surface border border-zed-border rounded-lg shadow-2xl p-5 space-y-4 text-zed-text-primary z-50 animate-in zoom-in-95 duration-150',
        className
      )}
    >
      <!-- Close button -->
      <button
        onclick={onClose}
        class="absolute right-3.5 top-3.5 p-1 rounded-sm text-zinc-400 hover:text-zinc-100 hover:bg-zinc-800 transition-colors"
        aria-label="Close"
      >
        <X size={15} />
      </button>

      <!-- Header -->
      {#if title || description}
        <div class="space-y-1 pr-6">
          {#if title}
            <h2 class="text-sm font-semibold text-zinc-100 tracking-tight">
              {title}
            </h2>
          {/if}
          {#if description}
            <p class="text-xs text-zinc-400">
              {description}
            </p>
          {/if}
        </div>
      {/if}

      <!-- Body -->
      {#if children}
        <div class="py-1">
          {@render children()}
        </div>
      {/if}

      <!-- Footer -->
      {#if footer}
        <div class="flex items-center justify-end space-x-2 pt-2 border-t border-zed-border/60">
          {@render footer()}
        </div>
      {/if}
    </div>
  </div>
{/if}
