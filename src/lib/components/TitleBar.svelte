<script lang="ts">
  import { PanelLeft, PanelRight, Plus, Cloud, Check, HardDrive, Search } from 'lucide-svelte';
  import Button from './ui/button/Button.svelte';

  interface Props {
    vaultName: string;
    currentPath: string;
    sidebarOpen: boolean;
    rightSidebarOpen: boolean;
    onToggleSidebar: () => void;
    onToggleRightSidebar: () => void;
    onOpenSearch: () => void;
    onNewNote: () => void;
    onSelectFolder: () => void;
    isSyncing?: boolean;
  }

  let {
    vaultName,
    currentPath,
    sidebarOpen,
    rightSidebarOpen,
    onToggleSidebar,
    onToggleRightSidebar,
    onOpenSearch,
    onNewNote,
    onSelectFolder,
    isSyncing = false
  }: Props = $props();

  let breadcrumbs = $derived.by(() => {
    if (!currentPath) return [];
    return currentPath.split('/').filter(Boolean);
  });
</script>

<header class="h-10 bg-zed-bg-surface border-b border-zed-border flex items-center justify-between px-3 text-xs select-none z-10">
  <!-- Left: Sidebar toggle & Vault name -->
  <div class="flex items-center space-x-2">
    <Button
      variant="ghost"
      size="icon"
      onclick={onToggleSidebar}
      class="text-zed-text-secondary hover:text-zed-text-primary hover:bg-zed-bg-hover"
      title="Toggle Left Project Panel (Ctrl+\)"
      aria-label="Toggle Project Panel"
    >
      <PanelLeft size={14} class={sidebarOpen ? 'text-zed-accent' : ''} />
    </Button>

    <div
      role="button"
      tabindex="0"
      onclick={onSelectFolder}
      onkeydown={(e) => e.key === 'Enter' && onSelectFolder()}
      class="flex items-center space-x-1.5 text-zed-text-muted hover:text-zed-text-primary cursor-pointer px-1.5 py-0.5 rounded hover:bg-zinc-800 transition-colors"
      title="Click to Choose or Switch Vault Folder on disk"
    >
      <HardDrive size={13} class="text-zed-accent" />
      <span class="font-medium text-zed-text-secondary hover:text-zed-text-primary transition-colors">
        {vaultName}
      </span>
    </div>
  </div>

  <!-- Center: Zed-style breadcrumb & Quick search trigger -->
  <div class="flex items-center space-x-2">
    <button
      onclick={onOpenSearch}
      class="flex items-center space-x-2 px-3 py-1 rounded bg-zinc-900/80 border border-zinc-800 text-zinc-400 hover:text-zinc-200 hover:border-zinc-700 transition-all font-mono text-[11px]"
      title="Search Vault (Ctrl+P)"
    >
      <Search size={12} class="text-zinc-500" />
      <span>Search notes...</span>
      <kbd class="text-[9.5px] px-1 py-0.2 rounded bg-zinc-800 text-zinc-500 border border-zinc-700">Ctrl+P</kbd>
    </button>
  </div>

  <!-- Right: Quick Actions, Sync status & Right sidebar toggle -->
  <div class="flex items-center space-x-2">
    <Button
      variant="secondary"
      size="sm"
      onclick={onNewNote}
      class="flex items-center space-x-1 px-2.5 text-[11px]"
      title="Create New Note (Ctrl+N)"
    >
      <Plus size={13} />
      <span>New Note</span>
    </Button>

    <div
      class="flex items-center space-x-1.5 px-2 py-1 rounded text-[11px] text-zinc-400 bg-zinc-900/60 border border-zinc-800"
      title="Google Drive Sync configured"
    >
      {#if isSyncing}
        <Cloud size={13} class="text-amber-400 animate-pulse" />
        <span class="text-amber-400">Syncing...</span>
      {:else}
        <Check size={13} class="text-emerald-400" />
        <span class="text-zinc-300">Drive Ready</span>
      {/if}
    </div>

    <Button
      variant="ghost"
      size="icon"
      onclick={onToggleRightSidebar}
      class="text-zed-text-secondary hover:text-zed-text-primary hover:bg-zed-bg-hover"
      title="Toggle Backlinks & Metadata Panel"
      aria-label="Toggle Metadata Panel"
    >
      <PanelRight size={14} class={rightSidebarOpen ? 'text-zed-accent' : ''} />
    </Button>
  </div>
</header>
