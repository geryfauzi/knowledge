<script lang="ts">
  import type { EditorStats } from '../types';
  import { CheckCircle2, RefreshCw } from 'lucide-svelte';

  interface Props {
    stats: EditorStats;
    vaultName: string;
    isSyncing?: boolean;
    isDirty?: boolean;
  }

  let {
    stats,
    vaultName,
    isSyncing = false,
    isDirty = false
  }: Props = $props();
</script>

<footer class="h-6 bg-zed-bg-surface border-t border-zed-border flex items-center justify-between px-3 text-[11px] font-mono text-zed-text-muted select-none">
  <!-- Left info -->
  <div class="flex items-center space-x-3">
    <span class="hover:text-zed-text-secondary cursor-pointer transition-colors">
      {vaultName}
    </span>

    <div class="flex items-center space-x-1.5">
      {#if isSyncing}
        <RefreshCw size={11} class="text-amber-400 animate-spin" />
        <span class="text-amber-400">Syncing with Drive...</span>
      {:else if isDirty}
        <span class="w-1.5 h-1.5 rounded-full bg-amber-400"></span>
        <span class="text-amber-300">Unsaved changes (Ctrl+S)</span>
      {:else}
        <CheckCircle2 size={11} class="text-emerald-400" />
        <span class="text-zinc-400">Up to date</span>
      {/if}
    </div>
  </div>

  <!-- Right stats -->
  <div class="flex items-center space-x-3 text-zinc-400">
    <span>
      Ln {stats.line}, Col {stats.col}
    </span>

    <span>
      {stats.words} {stats.words === 1 ? 'word' : 'words'}
    </span>

    <span class="text-zinc-500">
      UTF-8
    </span>

    <span class="text-zinc-300 font-sans text-[10.5px] px-1.5 py-0.5 rounded bg-zinc-800/80 border border-zinc-700/60">
      Markdown
    </span>
  </div>
</footer>
