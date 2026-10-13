<script lang="ts">
  import { onMount } from 'svelte';
  import { Search, FileText, ArrowRight } from 'lucide-svelte';
  import { indexerService, type SearchResult } from '../services/indexer';
  import Dialog from './ui/dialog/Dialog.svelte';
  import Input from './ui/input/Input.svelte';

  interface Props {
    open: boolean;
    onClose: () => void;
    onSelectFile: (path: string) => void;
  }

  let {
    open,
    onClose,
    onSelectFile
  }: Props = $props();

  let query = $state('');
  let results = $state<SearchResult[]>([]);
  let selectedIndex = $state(0);

  async function handleSearch(newQuery: string) {
    query = newQuery;
    if (!query.trim()) {
      // Default: show all known notes
      results = indexerService.noteTitles.map((n) => ({
        path: n.path,
        name: `${n.title}.md`,
        matches: ['Recent note']
      }));
      selectedIndex = 0;
      return;
    }
    results = await indexerService.search(query);
    selectedIndex = 0;
  }

  function handleKeyDown(e: KeyboardEvent) {
    if (e.key === 'ArrowDown') {
      e.preventDefault();
      selectedIndex = (selectedIndex + 1) % Math.max(1, results.length);
    } else if (e.key === 'ArrowUp') {
      e.preventDefault();
      selectedIndex = (selectedIndex - 1 + results.length) % Math.max(1, results.length);
    } else if (e.key === 'Enter') {
      e.preventDefault();
      if (results[selectedIndex]) {
        onSelectFile(results[selectedIndex].path);
        onClose();
      }
    }
  }

  $effect(() => {
    if (open) {
      handleSearch('');
    }
  });
</script>

<Dialog
  {open}
  {onClose}
  class="max-w-xl p-0 overflow-hidden bg-zed-bg-surface border border-zed-border shadow-2xl rounded-lg"
>
  <!-- Search input bar -->
  <div class="p-3 border-b border-zed-border/80 flex items-center space-x-2 bg-zinc-900/60">
    <Search size={15} class="text-zinc-400 shrink-0" />
    <input
      type="text"
      bind:value={query}
      oninput={(e) => handleSearch((e.target as HTMLInputElement).value)}
      onkeydown={handleKeyDown}
      placeholder="Search notes or file names (Ctrl+P)..."
      class="w-full bg-transparent border-none outline-none text-xs text-zinc-100 placeholder:text-zinc-500 font-sans"
    />
    <kbd class="text-[10px] text-zinc-500 font-mono px-1.5 py-0.5 rounded bg-zinc-800 border border-zinc-700">ESC</kbd>
  </div>

  <!-- Results List -->
  <div class="max-h-80 overflow-y-auto p-2 space-y-1">
    {#if results.length === 0}
      <div class="p-4 text-center text-zinc-500 text-xs italic">
        No matching notes found.
      </div>
    {:else}
      {#each results as result, idx}
        <div
          role="button"
          tabindex="0"
          onclick={() => { onSelectFile(result.path); onClose(); }}
          onkeydown={(e) => e.key === 'Enter' && (onSelectFile(result.path), onClose())}
          class="flex flex-col p-2 rounded cursor-pointer transition-colors {idx === selectedIndex ? 'bg-zinc-800 text-zinc-100 border-l-2 border-zed-accent' : 'text-zinc-300 hover:bg-zinc-800/50'}"
        >
          <div class="flex items-center justify-between text-xs font-medium">
            <div class="flex items-center space-x-2">
              <FileText size={13} class={idx === selectedIndex ? 'text-zed-accent' : 'text-zinc-500'} />
              <span>{result.name}</span>
            </div>
            <span class="text-[10px] text-zinc-500 font-mono">{result.path}</span>
          </div>

          {#if result.matches.length > 0 && result.matches[0] !== 'Recent note'}
            <p class="text-[11px] text-zinc-400 font-mono mt-1 pl-5 truncate">
              {result.matches[0]}
            </p>
          {/if}
        </div>
      {/each}
    {/if}
  </div>
</Dialog>
