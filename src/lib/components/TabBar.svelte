<script lang="ts">
  import type { TabItem } from '../types';
  import { X, FileText } from 'lucide-svelte';

  interface Props {
    tabs: TabItem[];
    activeTabId: string;
    onSelectTab: (id: string) => void;
    onCloseTab: (id: string) => void;
  }

  let {
    tabs,
    activeTabId,
    onSelectTab,
    onCloseTab
  }: Props = $props();
</script>

<div class="h-9 bg-zed-bg-surface border-b border-zed-border flex items-center overflow-x-auto select-none no-scrollbar">
  {#each tabs as tab (tab.id)}
    <div
      role="tab"
      tabindex="0"
      aria-selected={activeTabId === tab.id}
      onclick={() => onSelectTab(tab.id)}
      onkeydown={(e) => e.key === 'Enter' && onSelectTab(tab.id)}
      class="group h-full flex items-center space-x-2 px-3 border-r border-zed-border/70 text-xs cursor-pointer transition-colors {activeTabId === tab.id ? 'bg-zed-bg-base text-zed-text-primary border-t border-t-zed-accent' : 'text-zed-text-muted hover:bg-zed-bg-hover hover:text-zed-text-secondary'}"
    >
      <FileText size={12} class={activeTabId === tab.id ? 'text-zed-accent' : 'text-zinc-600'} />
      <span class="truncate max-w-[140px] text-[11.5px] font-medium">{tab.name}</span>

      <!-- Dirty dot or Close button -->
      <div class="w-4 h-4 flex items-center justify-center">
        {#if tab.isDirty}
          <span class="w-1.5 h-1.5 rounded-full bg-zed-accent group-hover:hidden"></span>
        {/if}
        <button
          onclick={(e) => { e.stopPropagation(); onCloseTab(tab.id); }}
          class="p-0.5 rounded hover:bg-zinc-700/60 text-zinc-500 hover:text-zinc-200 {tab.isDirty ? 'hidden group-hover:block' : 'opacity-0 group-hover:opacity-100'} transition-opacity"
          title="Close tab"
        >
          <X size={11} />
        </button>
      </div>
    </div>
  {/each}
</div>

<style>
  .no-scrollbar::-webkit-scrollbar {
    display: none;
  }
  .no-scrollbar {
    -ms-overflow-style: none;
    scrollbar-width: none;
  }
</style>
