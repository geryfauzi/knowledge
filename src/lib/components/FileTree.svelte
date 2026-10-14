<script lang="ts">
  import type { FileNode } from '../types';
  import {
    ChevronRight,
    ChevronDown,
    FileText,
    Folder,
    FolderOpen,
    Plus,
    FolderPlus,
    Trash2,
    Copy
  } from 'lucide-svelte';
  import Button from './ui/button/Button.svelte';
  import Input from './ui/input/Input.svelte';
  import { copyToClipboard, getRelativePath } from '../utils';
  import { vaultService } from '../services/vault';

  interface Props {
    files: FileNode[];
    activePath: string;
    onSelectFile: (file: FileNode) => void;
    onCreateFile: (name: string, parentPath?: string) => void;
    onCreateFolder: (name: string, parentPath?: string) => void;
    onDeleteNode?: (node: FileNode) => void;
    onNotify?: (msg: string) => void;
  }

  let {
    files,
    activePath,
    onSelectFile,
    onCreateFile,
    onCreateFolder,
    onDeleteNode,
    onNotify
  }: Props = $props();

  let expandedFolders = $state<Record<string, boolean>>({
    '/Ideas': true
  });

  let isCreating = $state<'file' | 'folder' | null>(null);
  let newItemName = $state('');
  let targetParentPath = $state('');
  let contextMenu = $state<{
    x: number;
    y: number;
    node: FileNode;
  } | null>(null);

  function handleContextMenu(e: MouseEvent, node: FileNode) {
    e.preventDefault();
    e.stopPropagation();
    const menuWidth = 160;
    const menuHeight = node.isDirectory ? 180 : 150;
    const x = Math.min(e.clientX, window.innerWidth - menuWidth - 8);
    const y = Math.min(e.clientY, window.innerHeight - menuHeight - 8);
    contextMenu = { x, y, node };
  }

  function closeContextMenu() {
    contextMenu = null;
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
            onkeydown={(e) => {
              if (e.key === 'Enter') toggleFolder(node.path, e as any);
              if (e.key === 'Delete') onDeleteNode?.(node);
            }}
            oncontextmenu={(e) => handleContextMenu(e, node)}
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
            <div class="opacity-0 group-hover:opacity-100 flex items-center space-x-0.5 text-zinc-500 shrink-0">
              <button
                onclick={(e) => { e.stopPropagation(); startCreate('file', node.path); }}
                class="hover:text-zinc-200 p-0.5 rounded transition-colors"
                title="New file in this folder"
              >
                <Plus size={11} />
              </button>
              <button
                onclick={(e) => { e.stopPropagation(); startCreate('folder', node.path); }}
                class="hover:text-zinc-200 p-0.5 rounded transition-colors"
                title="New folder in this folder"
              >
                <FolderPlus size={11} />
              </button>
              <button
                onclick={(e) => { e.stopPropagation(); onDeleteNode?.(node); }}
                class="hover:text-red-400 p-0.5 rounded transition-colors"
                title="Delete folder"
              >
                <Trash2 size={11} />
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
        <div
          role="button"
          tabindex="0"
          onclick={() => onSelectFile(node)}
          onkeydown={(e) => {
            if (e.key === 'Enter') onSelectFile(node);
            if (e.key === 'Delete') onDeleteNode?.(node);
          }}
          oncontextmenu={(e) => handleContextMenu(e, node)}
          class="w-full text-left flex items-center justify-between px-2 py-1 rounded cursor-pointer transition-colors group {activePath === node.path ? 'bg-zed-bg-active text-zed-text-primary font-medium' : 'text-zinc-400 hover:text-zinc-200 hover:bg-zed-bg-hover'}"
          style="padding-left: {level * 12 + 20}px;"
        >
          <div class="flex items-center space-x-1.5 truncate">
            <FileText size={12} class={activePath === node.path ? 'text-zed-accent' : 'text-zinc-500'} />
            <span class="truncate text-[11.5px]">{node.name}</span>
          </div>

          <!-- File hover quick actions -->
          <div class="opacity-0 group-hover:opacity-100 flex items-center space-x-0.5 text-zinc-500 shrink-0">
            <button
              onclick={(e) => { e.stopPropagation(); onDeleteNode?.(node); }}
              class="hover:text-red-400 p-0.5 rounded transition-colors"
              title="Delete note"
            >
              <Trash2 size={11} />
            </button>
          </div>
        </div>
      {/if}
    {/snippet}

    {#each files as rootNode}
      {@render renderNode(rootNode, 0)}
    {/each}
  </div>

  <!-- Context Menu (Right Click) -->
  {#if contextMenu}
    <div
      class="fixed inset-0 z-50 bg-transparent"
      role="presentation"
      onclick={closeContextMenu}
      oncontextmenu={(e) => { e.preventDefault(); closeContextMenu(); }}
    ></div>

    <div
      class="fixed z-50 min-w-[160px] bg-zinc-900 border border-zinc-700/80 rounded-md shadow-xl py-1 text-xs text-zinc-300 font-sans"
      style="top: {contextMenu.y}px; left: {contextMenu.x}px;"
    >
      {#if contextMenu.node.isDirectory}
        <button
          onclick={() => { const p = contextMenu!.node.path; closeContextMenu(); startCreate('file', p); }}
          class="w-full px-2.5 py-1.5 flex items-center space-x-2 hover:bg-zinc-800 hover:text-zinc-100 text-left transition-colors"
        >
          <Plus size={12} class="text-zinc-400" />
          <span>New Note</span>
        </button>
        <button
          onclick={() => { const p = contextMenu!.node.path; closeContextMenu(); startCreate('folder', p); }}
          class="w-full px-2.5 py-1.5 flex items-center space-x-2 hover:bg-zinc-800 hover:text-zinc-100 text-left transition-colors"
        >
          <FolderPlus size={12} class="text-zinc-400" />
          <span>New Folder</span>
        </button>
        <div class="h-px bg-zinc-800 my-1"></div>
        <button
          onclick={() => { const p = contextMenu!.node.path; closeContextMenu(); handleCopyPath(p); }}
          class="w-full px-2.5 py-1.5 flex items-center space-x-2 hover:bg-zinc-800 hover:text-zinc-100 text-left transition-colors"
        >
          <Copy size={12} class="text-zinc-400" />
          <span>Copy as Path</span>
        </button>
        <button
          onclick={() => { const p = contextMenu!.node.path; closeContextMenu(); handleCopyRelativePath(p); }}
          class="w-full px-2.5 py-1.5 flex items-center space-x-2 hover:bg-zinc-800 hover:text-zinc-100 text-left transition-colors"
        >
          <Copy size={12} class="text-zinc-400" />
          <span>Copy Relative Path</span>
        </button>
        <div class="h-px bg-zinc-800 my-1"></div>
        <button
          onclick={() => { const n = contextMenu!.node; closeContextMenu(); onDeleteNode?.(n); }}
          class="w-full px-2.5 py-1.5 flex items-center space-x-2 hover:bg-red-500/20 text-red-400 text-left transition-colors"
        >
          <Trash2 size={12} />
          <span>Delete Folder</span>
        </button>
      {:else}
        <button
          onclick={() => { const n = contextMenu!.node; closeContextMenu(); onSelectFile(n); }}
          class="w-full px-2.5 py-1.5 flex items-center space-x-2 hover:bg-zinc-800 hover:text-zinc-100 text-left transition-colors"
        >
          <FileText size={12} class="text-zinc-400" />
          <span>Open Note</span>
        </button>
        <div class="h-px bg-zinc-800 my-1"></div>
        <button
          onclick={() => { const p = contextMenu!.node.path; closeContextMenu(); handleCopyPath(p); }}
          class="w-full px-2.5 py-1.5 flex items-center space-x-2 hover:bg-zinc-800 hover:text-zinc-100 text-left transition-colors"
        >
          <Copy size={12} class="text-zinc-400" />
          <span>Copy as Path</span>
        </button>
        <button
          onclick={() => { const p = contextMenu!.node.path; closeContextMenu(); handleCopyRelativePath(p); }}
          class="w-full px-2.5 py-1.5 flex items-center space-x-2 hover:bg-zinc-800 hover:text-zinc-100 text-left transition-colors"
        >
          <Copy size={12} class="text-zinc-400" />
          <span>Copy Relative Path</span>
        </button>
        <div class="h-px bg-zinc-800 my-1"></div>
        <button
          onclick={() => { const n = contextMenu!.node; closeContextMenu(); onDeleteNode?.(n); }}
          class="w-full px-2.5 py-1.5 flex items-center space-x-2 hover:bg-red-500/20 text-red-400 text-left transition-colors"
        >
          <Trash2 size={12} />
          <span>Delete Note</span>
        </button>
      {/if}
    </div>
  {/if}
</aside>
