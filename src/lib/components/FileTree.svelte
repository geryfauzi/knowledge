<script lang="ts">
  import type { FileNode } from '../types';
  import {
    ChevronRight,
    ChevronDown,
    FileText,
    Folder,
    FolderOpen,
    Plus,
    FolderPlus
  } from 'lucide-svelte';
  import Button from './ui/button/Button.svelte';
  import Input from './ui/input/Input.svelte';

  interface Props {
    files: FileNode[];
    activePath: string;
    onSelectFile: (file: FileNode) => void;
    onCreateFile: (name: string, parentPath?: string) => void;
    onCreateFolder: (name: string, parentPath?: string) => void;
  }

  let {
    files,
    activePath,
    onSelectFile,
    onCreateFile,
    onCreateFolder
  }: Props = $props();

  let expandedFolders = $state<Record<string, boolean>>({
    '/Ideas': true
  });

  let isCreating = $state<'file' | 'folder' | null>(null);
  let newItemName = $state('');
  let targetParentPath = $state('');

  function toggleFolder(path: string, e: MouseEvent) {
    e.stopPropagation();
    expandedFolders[path] = !expandedFolders[path];
  }

  function startCreate(type: 'file' | 'folder', parentPath: string = '') {
    isCreating = type;
    targetParentPath = parentPath;
    newItemName = '';
  }

  function submitCreate() {
    if (!newItemName.trim()) {
      isCreating = null;
      return;
    }
    if (isCreating === 'file') {
      onCreateFile(newItemName.trim(), targetParentPath);
    } else if (isCreating === 'folder') {
      onCreateFolder(newItemName.trim(), targetParentPath);
    }
    isCreating = null;
    newItemName = '';
  }

  function handleKeyDown(e: KeyboardEvent) {
    if (e.key === 'Enter') {
      submitCreate();
    } else if (e.key === 'Escape') {
      isCreating = null;
      newItemName = '';
    }
  }
</script>

<aside class="w-60 h-full bg-zed-bg-surface border-r border-zed-border flex flex-col select-none text-xs">
  <!-- Zed Project Header with shadcn buttons -->
  <div class="h-9 px-3 border-b border-zed-border/60 flex items-center justify-between text-zed-text-muted">
    <span class="text-[10px] uppercase font-semibold tracking-wider text-zinc-400">Project</span>
    <div class="flex items-center space-x-0.5">
      <Button
        variant="ghost"
        size="icon"
        onclick={() => startCreate('file', '')}
        class="h-6 w-6 text-zinc-400 hover:text-zinc-100 hover:bg-zed-bg-hover"
        title="New Note in Root"
      >
        <Plus size={13} />
      </Button>
      <Button
        variant="ghost"
        size="icon"
        onclick={() => startCreate('folder', '')}
        class="h-6 w-6 text-zinc-400 hover:text-zinc-100 hover:bg-zed-bg-hover"
        title="New Folder in Root"
      >
        <FolderPlus size={13} />
      </Button>
    </div>
  </div>

  <!-- File Tree List -->
  <div class="flex-1 overflow-y-auto p-1.5 space-y-0.5">
    <!-- Top-level inline creation form with shadcn Input -->
    {#if isCreating && targetParentPath === ''}
      <div class="flex items-center space-x-1.5 px-2 py-1 bg-zinc-800/60 rounded border border-zed-accent/40 mb-1">
        {#if isCreating === 'file'}
          <FileText size={13} class="text-zed-accent shrink-0" />
        {:else}
          <Folder size={13} class="text-amber-400 shrink-0" />
        {/if}
        <Input
          bind:value={newItemName}
          onkeydown={handleKeyDown}
          onblur={submitCreate}
          placeholder={isCreating === 'file' ? 'note-name' : 'folder-name'}
          class="h-6 px-1.5 py-0 border-none bg-transparent shadow-none focus:ring-0 focus:border-none"
        />
      </div>
    {/if}

    {#snippet renderNode(node: FileNode, level: number)}
      {#if node.isDirectory}
        <div class="flex flex-col">
          <div
            role="button"
            tabindex="0"
            onclick={(e) => toggleFolder(node.path, e)}
            onkeydown={(e) => e.key === 'Enter' && toggleFolder(node.path, e as any)}
            class="flex items-center justify-between px-2 py-1 rounded cursor-pointer hover:bg-zed-bg-hover text-zinc-400 hover:text-zinc-200 group transition-colors"
            style="padding-left: {level * 12 + 8}px;"
          >
            <div class="flex items-center space-x-1.5 truncate">
              {#if expandedFolders[node.path]}
                <ChevronDown size={12} class="text-zinc-500" />
                <FolderOpen size={13} class="text-amber-400/90" />
              {:else}
                <ChevronRight size={12} class="text-zinc-500" />
                <Folder size={13} class="text-amber-400/80" />
              {/if}
              <span class="truncate font-medium text-[11.5px]">{node.name}</span>
            </div>

            <!-- Folder hover quick actions -->
            <div class="opacity-0 group-hover:opacity-100 flex items-center space-x-0.5 text-zinc-500">
              <button
                onclick={(e) => { e.stopPropagation(); startCreate('file', node.path); }}
                class="hover:text-zinc-200 p-0.5"
                title="New file in this folder"
              >
                <Plus size={11} />
              </button>
            </div>
          </div>

          {#if expandedFolders[node.path]}
            <!-- Inline create in folder -->
            {#if isCreating && targetParentPath === node.path}
              <div
                class="flex items-center space-x-1.5 px-2 py-1 bg-zinc-800/60 rounded border border-zed-accent/40 my-0.5"
                style="margin-left: {(level + 1) * 12 + 8}px;"
              >
                {#if isCreating === 'file'}
                  <FileText size={12} class="text-zed-accent shrink-0" />
                {:else}
                  <Folder size={12} class="text-amber-400 shrink-0" />
                {/if}
                <Input
                  bind:value={newItemName}
                  onkeydown={handleKeyDown}
                  onblur={submitCreate}
                  placeholder={isCreating === 'file' ? 'note-name' : 'folder-name'}
                  class="h-6 px-1.5 py-0 border-none bg-transparent shadow-none focus:ring-0 focus:border-none"
                />
              </div>
            {/if}

            {#if node.children}
              {#each node.children as child}
                {@render renderNode(child, level + 1)}
              {/each}
            {/if}
          {/if}
        </div>
      {:else}
        <button
          onclick={() => onSelectFile(node)}
          class="w-full text-left flex items-center space-x-1.5 px-2 py-1 rounded cursor-pointer transition-colors {activePath === node.path ? 'bg-zed-bg-active text-zed-text-primary font-medium' : 'text-zinc-400 hover:text-zinc-200 hover:bg-zed-bg-hover'}"
          style="padding-left: {level * 12 + 20}px;"
        >
          <FileText size={12} class={activePath === node.path ? 'text-zed-accent' : 'text-zinc-500'} />
          <span class="truncate text-[11.5px]">{node.name}</span>
        </button>
      {/if}
    {/snippet}

    {#each files as rootNode}
      {@render renderNode(rootNode, 0)}
    {/each}
  </div>
</aside>
