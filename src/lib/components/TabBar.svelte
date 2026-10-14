<script lang="ts">
  import type { TabItem } from '../types';
  import { X, FileText, Copy } from 'lucide-svelte';
  import { copyToClipboard, getRelativePath } from '../utils';
  import { vaultService } from '../services/vault';

  interface Props {
    tabs: TabItem[];
    activeTabId: string;
    onSelectTab: (id: string) => void;
    onCloseTab: (id: string) => void;
    onCloseOtherTabs?: (id: string) => void;
    onNotify?: (msg: string) => void;
  }

  let {
    tabs,
    activeTabId,
    onSelectTab,
    onCloseTab,
    onCloseOtherTabs,
    onNotify
  }: Props = $props();

  let tabContextMenu = $state<{
    x: number;
    y: number;
    tab: TabItem;
  } | null>(null);

  function handleContextMenu(e: MouseEvent, tab: TabItem) {
    e.preventDefault();
    e.stopPropagation();
    const menuWidth = 160;
    const menuHeight = 140;
    const x = Math.min(e.clientX, window.innerWidth - menuWidth - 8);
    const y = Math.min(e.clientY, window.innerHeight - menuHeight - 8);
    tabContextMenu = { x, y, tab };
  }

  function closeContextMenu() {
    tabContextMenu = null;
  }

  async function handleCopyPath(path: string) {
    const success = await copyToClipboard(path);
    if (success) {
      onNotify?.(`Copied path: ${path}`);
    }
  }

  async function handleCopyRelativePath(path: string) {
    const rel = getRelativePath(path, vaultService.vaultPath);
    const success = await copyToClipboard(rel);
    if (success) {
      onNotify?.(`Copied relative path: ${rel}`);
    }
  }
</script>

<div class="h-9 bg-zed-bg-surface border-b border-zed-border flex items-center overflow-x-auto select-none no-scrollbar">
  {#each tabs as tab (tab.id)}
    <div
      role="tab"
      tabindex="0"
      aria-selected={activeTabId === tab.id}
      onclick={() => onSelectTab(tab.id)}
      onauxclick={(e) => { if (e.button === 1) { e.preventDefault(); e.stopPropagation(); onCloseTab(tab.id); } }}
      onkeydown={(e) => e.key === 'Enter' && onSelectTab(tab.id)}
      oncontextmenu={(e) => handleContextMenu(e, tab)}
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

{#if tabContextMenu}
  <div
    class="fixed inset-0 z-50 bg-transparent"
    role="presentation"
    onclick={closeContextMenu}
    oncontextmenu={(e) => { e.preventDefault(); closeContextMenu(); }}
  ></div>

  <div
    class="fixed z-50 min-w-[160px] bg-zinc-900 border border-zinc-700/80 rounded-md shadow-xl py-1 text-xs text-zinc-300 font-sans"
    style="top: {tabContextMenu.y}px; left: {tabContextMenu.x}px;"
  >
    <button
      onclick={() => { const p = tabContextMenu!.tab.path; closeContextMenu(); handleCopyPath(p); }}
      class="w-full px-2.5 py-1.5 flex items-center space-x-2 hover:bg-zinc-800 hover:text-zinc-100 text-left transition-colors"
    >
      <Copy size={12} class="text-zinc-400" />
      <span>Copy as Path</span>
    </button>
    <button
      onclick={() => { const p = tabContextMenu!.tab.path; closeContextMenu(); handleCopyRelativePath(p); }}
      class="w-full px-2.5 py-1.5 flex items-center space-x-2 hover:bg-zinc-800 hover:text-zinc-100 text-left transition-colors"
    >
      <Copy size={12} class="text-zinc-400" />
      <span>Copy Relative Path</span>
    </button>
    <div class="h-px bg-zinc-800 my-1"></div>
    <button
      onclick={() => { const id = tabContextMenu!.tab.id; closeContextMenu(); onCloseTab(id); }}
      class="w-full px-2.5 py-1.5 flex items-center space-x-2 hover:bg-zinc-800 hover:text-zinc-100 text-left transition-colors"
    >
      <X size={12} class="text-zinc-400" />
      <span>Close</span>
    </button>
    {#if tabs.length > 1}
      <button
        onclick={() => { const id = tabContextMenu!.tab.id; closeContextMenu(); onCloseOtherTabs?.(id); }}
        class="w-full px-2.5 py-1.5 flex items-center space-x-2 hover:bg-zinc-800 hover:text-zinc-100 text-left transition-colors"
      >
        <span class="w-3"></span>
        <span>Close Others</span>
      </button>
    {/if}
  </div>
{/if}
