<script lang="ts">
  import { onMount } from 'svelte';
  import TitleBar from './lib/components/TitleBar.svelte';
  import FileTree from './lib/components/FileTree.svelte';
  import TabBar from './lib/components/TabBar.svelte';
  import Editor from './lib/components/Editor.svelte';
  import StatusBar from './lib/components/StatusBar.svelte';
  import RightSidebar from './lib/components/RightSidebar.svelte';
  import QuickSearchModal from './lib/components/QuickSearchModal.svelte';
  import PromptModal from './lib/components/ui/dialog/PromptModal.svelte';
  import Button from './lib/components/ui/button/Button.svelte';
  import { vaultService } from './lib/services/vault';
  import { indexerService } from './lib/services/indexer';
  import type { FileNode, TabItem, EditorStats } from './lib/types';
  import { FileText, Plus, FolderOpen } from 'lucide-svelte';

  let files = $state<FileNode[]>([]);
  let tabs = $state<TabItem[]>([]);
  let activeTabId = $state<string>('');
  let sidebarOpen = $state<boolean>(true);
  let rightSidebarOpen = $state<boolean>(true);
  let searchOpen = $state<boolean>(false);
  let isSyncing = $state<boolean>(false);

  // Modal dialog state (replacing native prompt and alert completely)
  let modalState = $state<{
    open: boolean;
    title: string;
    description: string;
    placeholder: string;
    initialValue?: string;
    confirmText: string;
    cancelText?: string;
    isAlertOnly: boolean;
    onConfirm: (val: string) => void;
  }>({
    open: false,
    title: '',
    description: '',
    placeholder: '',
    confirmText: 'Confirm',
    cancelText: 'Cancel',
    isAlertOnly: false,
    onConfirm: () => {}
  });

  let editorStats = $state<EditorStats>({
    line: 1,
    col: 1,
    characters: 0,
    words: 0,
    selectedChars: 0
  });

  let activeTab = $derived(tabs.find(t => t.id === activeTabId));

  function openCreateNoteDialog(parentPath: string = '') {
    modalState = {
      open: true,
      title: 'New Note',
      description: parentPath ? `Create note in ${parentPath}:` : 'Enter a title for your new note:',
      placeholder: 'e.g. Daily Reflection or Project Ideas',
      initialValue: '',
      confirmText: 'Create Note',
      cancelText: 'Cancel',
      isAlertOnly: false,
      onConfirm: (name) => {
        if (name.trim()) {
          handleCreateFile(name.trim(), parentPath);
        }
      }
    };
  }

  function openCreateFolderDialog(parentPath: string = '') {
    modalState = {
      open: true,
      title: 'New Folder',
      description: parentPath ? `Create folder in ${parentPath}:` : 'Enter a name for your new folder:',
      placeholder: 'e.g. Work, Personal, Archive',
      initialValue: '',
      confirmText: 'Create Folder',
      cancelText: 'Cancel',
      isAlertOnly: false,
      onConfirm: (name) => {
        if (name.trim()) {
          handleCreateFolder(name.trim(), parentPath);
        }
      }
    };
  }

  async function handleSelectFolder() {
    const success = await vaultService.selectVaultFolder();
    if (success) {
      files = await vaultService.getFiles();
      await indexerService.reindexVault();
      if (files.length > 0 && tabs.length === 0) {
        const firstFile = findFirstFile(files);
        if (firstFile) openFile(firstFile);
      }
    }
  }

  async function loadVault() {
    files = await vaultService.getFiles();
    if (files.length > 0) {
      await indexerService.reindexVault();
      if (tabs.length === 0) {
        const firstFile = findFirstFile(files);
        if (firstFile) openFile(firstFile);
      }
    }
  }

  function findFirstFile(nodes: FileNode[]): FileNode | null {
    for (const node of nodes) {
      if (!node.isDirectory) return node;
      if (node.children) {
        const found = findFirstFile(node.children);
        if (found) return found;
      }
    }
    return null;
  }

  function findNodeByPath(nodes: FileNode[], targetPath: string): FileNode | null {
    for (const node of nodes) {
      if (node.path === targetPath) return node;
      if (node.children) {
        const found = findNodeByPath(node.children, targetPath);
        if (found) return found;
      }
    }
    return null;
  }

  async function openFile(file: FileNode) {
    const existing = tabs.find(t => t.path === file.path);
    if (existing) {
      activeTabId = existing.id;
      return;
    }

    const content = await vaultService.readFile(file.path);
    const newTab: TabItem = {
      id: file.path,
      path: file.path,
      name: file.name,
      content,
      isDirty: false
    };

    tabs = [...tabs, newTab];
    activeTabId = newTab.id;
  }

  async function openFileByPath(path: string) {
    const found = findNodeByPath(files, path);
    if (found) {
      openFile(found);
    } else {
      // Create node dynamically if exists in index
      const name = path.split('/').pop() || path;
      const content = await vaultService.readFile(path);
      openFile({
        path,
        name,
        isDirectory: false,
        content
      });
    }
  }

  function handleSelectTab(id: string) {
    activeTabId = id;
  }

  function handleCloseTab(id: string) {
    const tabToClose = tabs.find(t => t.id === id);
    if (!tabToClose) return;

    if (tabToClose.isDirty) {
      modalState = {
        open: true,
        title: 'Unsaved Changes',
        description: `Do you want to save changes to "${tabToClose.name}" before closing?`,
        placeholder: '',
        initialValue: '',
        confirmText: 'Save & Close',
        cancelText: 'Discard & Close',
        isAlertOnly: true,
        onConfirm: async () => {
          await vaultService.writeFile(tabToClose.path, tabToClose.content);
          closeTabDirectly(id);
        }
      };
      return;
    }

    closeTabDirectly(id);
  }

  function closeTabDirectly(id: string) {
    const index = tabs.findIndex(t => t.id === id);
    if (index === -1) return;

    tabs = tabs.filter(t => t.id !== id);

    if (activeTabId === id) {
      if (tabs.length > 0) {
        activeTabId = tabs[Math.max(0, index - 1)].id;
      } else {
        activeTabId = '';
      }
    }
  }

  function handleEditorChange(newContent: string) {
    if (!activeTab) return;
    activeTab.content = newContent;
    activeTab.isDirty = true;
  }

  async function handleSaveCurrent() {
    if (!activeTab) return;
    await vaultService.writeFile(activeTab.path, activeTab.content);
    activeTab.isDirty = false;

    // Update in-memory index
    indexerService.indexFileContent(activeTab.path, activeTab.name, activeTab.content);
  }

  async function handleCreateFile(name: string, parentPath?: string) {
    const newFile = await vaultService.createFile(name, parentPath);
    files = await vaultService.getFiles();
    await indexerService.reindexVault();
    openFile(newFile);
  }

  async function handleCreateFolder(name: string, parentPath?: string) {
    await vaultService.createFolder(name, parentPath);
    files = await vaultService.getFiles();
  }

  function handleKeyDown(e: KeyboardEvent) {
    // Ctrl+S: Save
    if ((e.ctrlKey || e.metaKey) && e.key.toLowerCase() === 's') {
      e.preventDefault();
      handleSaveCurrent();
      return;
    }
    // Ctrl+P / Cmd+P: Quick Search Command Palette
    if ((e.ctrlKey || e.metaKey) && e.key.toLowerCase() === 'p') {
      e.preventDefault();
      searchOpen = true;
      return;
    }
    // Ctrl+N: New Note
    if ((e.ctrlKey || e.metaKey) && e.key.toLowerCase() === 'n') {
      e.preventDefault();
      openCreateNoteDialog();
      return;
    }
    // Toggle Left Sidebar: Ctrl+\ or Alt+B
    if (((e.ctrlKey || e.metaKey) && e.key === '\\') || (e.altKey && e.key.toLowerCase() === 'b')) {
      e.preventDefault();
      sidebarOpen = !sidebarOpen;
      return;
    }
  }

  onMount(() => {
    loadVault();
    window.addEventListener('keydown', handleKeyDown);
    return () => {
      window.removeEventListener('keydown', handleKeyDown);
    };
  });
</script>

<div class="h-screen w-screen flex flex-col bg-zed-bg-base text-zed-text-primary overflow-hidden select-none font-sans">
  <!-- Top Navigation & Breadcrumbs -->
  <TitleBar
    vaultName={vaultService.vaultName}
    currentPath={activeTab?.path ?? ''}
    {sidebarOpen}
    {rightSidebarOpen}
    onToggleSidebar={() => (sidebarOpen = !sidebarOpen)}
    onToggleRightSidebar={() => (rightSidebarOpen = !rightSidebarOpen)}
    onOpenSearch={() => (searchOpen = true)}
    onNewNote={() => openCreateNoteDialog()}
    onSelectFolder={handleSelectFolder}
    {isSyncing}
  />

  <!-- Main Body: Sidebar + Editor + Right Sidebar -->
  <div class="flex-1 flex overflow-hidden">
    <!-- Left Project Panel -->
    {#if sidebarOpen}
      <FileTree
        {files}
        activePath={activeTab?.path ?? ''}
        onSelectFile={openFile}
        onCreateFile={handleCreateFile}
        onCreateFolder={handleCreateFolder}
      />
    {/if}

    <!-- Center Workspace -->
    <main class="flex-1 flex flex-col overflow-hidden bg-zed-bg-base">
      {#if tabs.length > 0}
        <TabBar
          {tabs}
          {activeTabId}
          onSelectTab={handleSelectTab}
          onCloseTab={handleCloseTab}
        />

        {#if activeTab}
          <div class="flex-1 overflow-hidden relative">
            <Editor
              content={activeTab.content}
              path={activeTab.path}
              isDirty={activeTab.isDirty}
              onChange={handleEditorChange}
              onSave={handleSaveCurrent}
              onStatsChange={(stats) => (editorStats = stats)}
            />
          </div>
        {/if}
      {:else}
        <!-- Empty Workspace (Zed Minimalist Style) -->
        <div class="flex-1 flex flex-col items-center justify-center text-center p-6 text-zinc-500">
          <div class="w-12 h-12 rounded-xl bg-zinc-900 border border-zinc-800 flex items-center justify-center text-zinc-400 mb-4 shadow-sm">
            <FileText size={22} />
          </div>
          <h2 class="text-sm font-semibold text-zinc-200 mb-1">
            {vaultService.isVaultOpen ? vaultService.vaultName : 'No Vault Open'}
          </h2>
          <p class="text-xs text-zinc-500 max-w-sm mb-6">
            {vaultService.isVaultOpen 
              ? 'Select a note from the project panel or create a new file to start writing.'
              : 'Choose a folder on your computer to store and edit real markdown files.'}
          </p>

          {#if !vaultService.isVaultOpen}
            <Button
              variant="default"
              size="lg"
              onclick={handleSelectFolder}
              class="flex items-center space-x-2 bg-zed-accent text-zinc-950 hover:bg-zed-accent-hover font-semibold px-4 mb-4"
            >
              <FolderOpen size={16} />
              <span>Choose Vault Folder on Computer</span>
            </Button>
          {:else}
            <div class="flex flex-col space-y-2 text-xs font-mono text-zinc-400 mb-6">
              <div class="flex items-center space-x-3">
                <span class="px-2 py-0.5 rounded bg-zinc-800 border border-zinc-700 text-zinc-300">Ctrl + P</span>
                <span>Quick Search Notes</span>
              </div>
              <div class="flex items-center space-x-3">
                <span class="px-2 py-0.5 rounded bg-zinc-800 border border-zinc-700 text-zinc-300">Ctrl + N</span>
                <span>New Note</span>
              </div>
              <div class="flex items-center space-x-3">
                <span class="px-2 py-0.5 rounded bg-zinc-800 border border-zinc-700 text-zinc-300">Ctrl + \</span>
                <span>Toggle Project Panel</span>
              </div>
              <div class="flex items-center space-x-3">
                <span class="px-2 py-0.5 rounded bg-zinc-800 border border-zinc-700 text-zinc-300">Ctrl + S</span>
                <span>Save Changes</span>
              </div>
            </div>

            <Button
              variant="default"
              size="sm"
              onclick={() => openCreateNoteDialog()}
              class="flex items-center space-x-1.5 bg-zed-accent text-zinc-950 hover:bg-zed-accent-hover font-semibold"
            >
              <Plus size={13} />
              <span>Create Note in Vault</span>
            </Button>
          {/if}
        </div>
      {/if}
    </main>

    <!-- Right Metadata & Backlinks Panel -->
    {#if rightSidebarOpen}
      <RightSidebar
        currentPath={activeTab?.path ?? ''}
        currentName={activeTab?.name ?? ''}
        onOpenFile={openFileByPath}
      />
    {/if}
  </div>

  <!-- Bottom Status Bar -->
  <StatusBar
    stats={editorStats}
    vaultName={vaultService.vaultName}
    {isSyncing}
    isDirty={activeTab?.isDirty ?? false}
  />

  <!-- shadcn Modal / Dialog (Zero native browser prompts/alerts) -->
  <PromptModal
    open={modalState.open}
    title={modalState.title}
    description={modalState.description}
    placeholder={modalState.placeholder}
    initialValue={modalState.initialValue}
    confirmText={modalState.confirmText}
    cancelText={modalState.cancelText}
    isAlertOnly={modalState.isAlertOnly}
    onConfirm={modalState.onConfirm}
    onClose={() => (modalState.open = false)}
  />

  <!-- Quick Search Modal (Command Palette Ctrl+P) -->
  <QuickSearchModal
    open={searchOpen}
    onClose={() => (searchOpen = false)}
    onSelectFile={openFileByPath}
  />
</div>
