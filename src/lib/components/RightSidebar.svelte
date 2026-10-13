<script lang="ts">
  import { onMount } from 'svelte';
  import { Link2, Hash, ListTree, ArrowUpRight, FileText } from 'lucide-svelte';
  import { indexerService, type BacklinkItem } from '../services/indexer';
  import Button from './ui/button/Button.svelte';

  interface Props {
    currentPath: string;
    currentName: string;
    onOpenFile: (path: string) => void;
  }

  let {
    currentPath,
    currentName,
    onOpenFile
  }: Props = $props();

  let activeTab = $state<'backlinks' | 'tags' | 'outline'>('backlinks');
  let backlinks = $state<BacklinkItem[]>([]);
  let allTags = $state<{ tag: string; count: number; files: string[] }[]>([]);

  async function loadMetadata() {
    if (!currentPath) {
      backlinks = [];
      return;
    }
    backlinks = await indexerService.getBacklinksForFile(currentPath, currentName);
    allTags = indexerService.getAllTags();
  }

  $effect(() => {
    if (currentPath) {
      loadMetadata();
    }
  });

  onMount(() => {
    loadMetadata();
  });
</script>

<aside class="w-64 h-full bg-zed-bg-surface border-l border-zed-border flex flex-col select-none text-xs">
  <!-- Right Sidebar Header Tabs -->
  <div class="h-9 border-b border-zed-border/70 flex items-center justify-around px-2 text-zed-text-muted">
    <button
      onclick={() => (activeTab = 'backlinks')}
      class="flex items-center space-x-1.5 py-1 px-2 rounded transition-colors {activeTab === 'backlinks' ? 'text-zed-accent font-medium bg-zinc-800/60' : 'text-zinc-400 hover:text-zinc-200'}"
      title="Backlinks (Linked Mentions)"
    >
      <Link2 size={13} />
      <span class="text-[11px]">Backlinks ({backlinks.length})</span>
    </button>

    <button
      onclick={() => (activeTab = 'tags')}
      class="flex items-center space-x-1.5 py-1 px-2 rounded transition-colors {activeTab === 'tags' ? 'text-zed-accent font-medium bg-zinc-800/60' : 'text-zinc-400 hover:text-zinc-200'}"
      title="Tags Index"
    >
      <Hash size={13} />
      <span class="text-[11px]">Tags</span>
    </button>
  </div>

  <!-- Content Panel -->
  <div class="flex-1 overflow-y-auto p-3 space-y-3">
    {#if activeTab === 'backlinks'}
      <div>
        <div class="text-[10px] uppercase tracking-wider font-semibold text-zinc-500 mb-2">
          Linked Mentions
        </div>

        {#if backlinks.length === 0}
          <div class="p-3 rounded bg-zinc-900/40 border border-zinc-800/60 text-center text-zinc-500 text-[11px] italic">
            No other notes link to this page yet.
          </div>
        {:else}
          <div class="space-y-2">
            {#each backlinks as link}
              <div
                role="button"
                tabindex="0"
                onclick={() => onOpenFile(link.sourcePath)}
                onkeydown={(e) => e.key === 'Enter' && onOpenFile(link.sourcePath)}
                class="p-2 rounded bg-zinc-900/70 border border-zinc-800/80 hover:border-zed-accent/40 cursor-pointer group transition-all"
              >
                <div class="flex items-center justify-between text-zinc-200 font-medium text-[11.5px] mb-1">
                  <div class="flex items-center space-x-1.5 truncate">
                    <FileText size={12} class="text-zed-accent" />
                    <span class="truncate">{link.sourceName}</span>
                  </div>
                  <ArrowUpRight size={12} class="text-zinc-500 group-hover:text-zed-accent opacity-0 group-hover:opacity-100 transition-opacity" />
                </div>
                {#if link.context}
                  <p class="text-[10.5px] text-zinc-400 font-mono line-clamp-2 bg-zinc-950/40 p-1 rounded">
                    "{link.context}"
                  </p>
                {/if}
              </div>
            {/each}
          </div>
        {/if}
      </div>
    {:else if activeTab === 'tags'}
      <div>
        <div class="text-[10px] uppercase tracking-wider font-semibold text-zinc-500 mb-2">
          Vault Tags
        </div>

        {#if allTags.length === 0}
          <div class="p-3 rounded bg-zinc-900/40 border border-zinc-800/60 text-center text-zinc-500 text-[11px] italic">
            No tags found. Add #tag in your notes.
          </div>
        {:else}
          <div class="flex flex-wrap gap-1.5">
            {#each allTags as tagItem}
              <span class="inline-flex items-center space-x-1 px-2 py-0.5 rounded bg-zinc-800/80 border border-zinc-700/60 text-[11px] text-amber-300">
                <span>#{tagItem.tag}</span>
                <span class="text-zinc-500 text-[9.5px]">({tagItem.count})</span>
              </span>
            {/each}
          </div>
        {/if}
      </div>
    {/if}
  </div>
</aside>
