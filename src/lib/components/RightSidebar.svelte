<script lang="ts">
  import { onMount } from 'svelte';
  import { Link2, Hash, ListTree, ArrowUpRight, FileText } from 'lucide-svelte';
  import { indexerService, type BacklinkItem } from '../services/indexer';
  import Button from './ui/button/Button.svelte';

  interface HeadingItem {
    level: number;
    text: string;
    line: number;
  }

  interface Props {
    currentPath: string;
    currentName: string;
    currentContent?: string;
    onOpenFile: (path: string) => void;
    onScrollToHeading?: (headingText: string, line?: number, index?: number) => void;
  }

  let {
    currentPath,
    currentName,
    currentContent = '',
    onOpenFile,
    onScrollToHeading
  }: Props = $props();

  let activeTab = $state<'backlinks' | 'tags' | 'outline'>('backlinks');
  let backlinks = $state<BacklinkItem[]>([]);
  let allTags = $state<{ tag: string; count: number; files: string[] }[]>([]);

  let headings = $derived.by(() => {
    if (!currentContent) return [];
    const lines = currentContent.split('\n');
    const result: HeadingItem[] = [];
    for (let i = 0; i < lines.length; i++) {
      const match = lines[i].match(/^(#{1,6})\s+(.+)$/);
      if (match) {
        result.push({
          level: match[1].length,
          text: match[2].trim(),
          line: i + 1
        });
      }
    }
    return result;
  });

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
      class="flex items-center space-x-1 py-1 px-1.5 rounded transition-colors {activeTab === 'backlinks' ? 'text-zed-accent font-medium bg-zinc-800/60' : 'text-zinc-400 hover:text-zinc-200'}"
      title="Backlinks (Linked Mentions)"
    >
      <Link2 size={12} />
      <span class="text-[10.5px]">Links ({backlinks.length})</span>
    </button>

    <button
      onclick={() => (activeTab = 'tags')}
      class="flex items-center space-x-1 py-1 px-1.5 rounded transition-colors {activeTab === 'tags' ? 'text-zed-accent font-medium bg-zinc-800/60' : 'text-zinc-400 hover:text-zinc-200'}"
      title="Tags Index"
    >
      <Hash size={12} />
      <span class="text-[10.5px]">Tags ({allTags.length})</span>
    </button>

    <button
      onclick={() => (activeTab = 'outline')}
      class="flex items-center space-x-1 py-1 px-1.5 rounded transition-colors {activeTab === 'outline' ? 'text-zed-accent font-medium bg-zinc-800/60' : 'text-zinc-400 hover:text-zinc-200'}"
      title="Document Outline"
    >
      <ListTree size={12} />
      <span class="text-[10.5px]">Outline ({headings.length})</span>
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
    {:else if activeTab === 'outline'}
      <div>
        <div class="text-[10px] uppercase tracking-wider font-semibold text-zinc-500 mb-2">
          Document Outline
        </div>

        {#if headings.length === 0}
          <div class="p-3 rounded bg-zinc-900/40 border border-zinc-800/60 text-center text-zinc-500 text-[11px] italic">
            No headings found in this note. Use # Heading to create structure.
          </div>
        {:else}
          <div class="space-y-1">
            {#each headings as h, index}
              <button
                type="button"
                onclick={() => onScrollToHeading?.(h.text, h.line, index)}
                class="w-full text-left py-1.5 px-2 rounded hover:bg-zinc-800/80 active:bg-zinc-700/80 text-zinc-300 hover:text-white transition-all truncate block text-[11px] font-mono cursor-pointer border border-transparent hover:border-zinc-700/60"
                style="padding-left: {(h.level - 1) * 12 + 8}px"
                title="Line {h.line}: H{h.level} - Click to jump and focus"
              >
                <span class="text-zed-accent/80 font-semibold text-[10px] mr-1.5 inline-block">H{h.level}</span>
                <span>{h.text}</span>
              </button>
            {/each}
          </div>
        {/if}
      </div>
    {/if}
  </div>
</aside>
