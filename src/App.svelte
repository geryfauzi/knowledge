<script lang="ts">
  import { onMount } from 'svelte';
  import TitleBar from './lib/components/TitleBar.svelte';
  import FileTree from './lib/components/FileTree.svelte';
  import TabBar from './lib/components/TabBar.svelte';
  import Editor from './lib/components/Editor.svelte';
  import StatusBar from './lib/components/StatusBar.svelte';
  import RightSidebar from './lib/components/RightSidebar.svelte';
  import QuickSearchModal from './lib/components/QuickSearchModal.svelte';
  import SettingsModal from './lib/components/SettingsModal.svelte';
  import GraphView from './lib/components/GraphView.svelte';
  import PromptModal from './lib/components/ui/dialog/PromptModal.svelte';
  import Button from './lib/components/ui/button/Button.svelte';
  import { vaultService } from './lib/services/vault';
  import { indexerService } from './lib/services/indexer';
  import { driveSyncService } from './lib/services/driveSync.svelte';
  import type { FileNode, TabItem, EditorStats } from './lib/types';
  import { FileText, Plus, FolderOpen, Check } from 'lucide-svelte';

  let files = $state<FileNode[]>([]);
  let tabs = $state<TabItem[]>([]);
  let activeTabId = $state<string>('');
  let sidebarOpen = $state<boolean>(true);
  let rightSidebarOpen = $state<boolean>(true);
  let searchOpen = $state<boolean>(false);
  let settingsOpen = $state<boolean>(false);
  let isGraphOpen = $state<boolean>(false);
  let vaultName = $state<string>(vaultService.vaultName);
  let isVaultOpen = $state<boolean>(vaultService.isVaultOpen);
  let vaultPath = $state<string>(vaultService.vaultPath);
  let toastMessage = $state<string | null>(null);
  let toastTimer: any = null;
  let editorRef = $state<any>(null);

  function handleScrollToHeading(headingText: string, line?: number, index?: number) {
    if (editorRef) {
      editorRef.scrollToHeading(headingText, line, index);
    }
  }

  function showToast(message: string) {
    toastMessage = message;
    clearTimeout(toastTimer);
    toastTimer = setTimeout(() => {
      toastMessage = null;
    }, 2200);
  }

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
    danger?: boolean;
    onConfirm: (val: string) => void;
    onCancel?: () => void;
  }>({
    open: false,
    title: '',
    description: '',
    placeholder: '',
    confirmText: 'Confirm',
    cancelText: 'Cancel',
    isAlertOnly: false,
    danger: false,
    onConfirm: () => {},
    onCancel: undefined
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

  function getSessionStorageKey(): string {
    if (!vaultPath) return 'knowledge_workspace_session_default';
    return `knowledge_workspace_session_${vaultPath.replace(/[^a-zA-Z0-9]/g, '_')}`;
  }

  function saveWorkspaceSession() {
    if (typeof window === 'undefined' || !window.localStorage || !vaultPath) return;
    try {
      const session = {
        vaultPath,
        openTabPaths: tabs.map(t => t.path),
        activeTabPath: activeTabId
      };
      localStorage.setItem(getSessionStorageKey(), JSON.stringify(session));
    } catch (e) {
      console.warn('Failed to save workspace session:', e);
    }
  }

  async function restoreWorkspaceSession(): Promise<boolean> {
    if (typeof window === 'undefined' || !window.localStorage || !vaultPath) return false;
    try {
      const raw = localStorage.getItem(getSessionStorageKey());
      if (!raw) return false;
      const session = JSON.parse(raw);
      if (!session || !Array.isArray(session.openTabPaths) || session.openTabPaths.length === 0) {
        return false;
      }

      const restoredTabs: TabItem[] = [];
      for (const filePath of session.openTabPaths) {
        try {
          const content = await vaultService.readFile(filePath);
          const name = filePath.split('/').pop() || filePath;
          restoredTabs.push({
            id: filePath,
            path: filePath,
            name,
            content,
            isDirty: false
          });
        } catch {
          // File might have been deleted outside, skip
        }
      }

      if (restoredTabs.length > 0) {
        tabs = restoredTabs;
        const targetActive = session.activeTabPath && restoredTabs.some(t => t.id === session.activeTabPath)
          ? session.activeTabPath
          : restoredTabs[restoredTabs.length - 1].id;
        activeTabId = targetActive;
        return true;
      }
    } catch (e) {
      console.warn('Failed to restore workspace session:', e);
    }
    return false;
  }

  async function handleSelectFolder() {
    const success = await vaultService.selectVaultFolder();
    if (success) {
      vaultName = vaultService.vaultName;
      isVaultOpen = vaultService.isVaultOpen;
      vaultPath = vaultService.vaultPath;
      tabs = [];
      activeTabId = '';
      files = await vaultService.getFiles();
      await indexerService.reindexVault();
      if (files.length > 0) {
        const restored = await restoreWorkspaceSession();
        if (!restored) {
          const firstFile = findFirstFile(files);
          if (firstFile) openFile(firstFile);
        }
      }
    }
  }

  async function refreshVaultState() {
    files = await vaultService.rescanVault();
    vaultName = vaultService.vaultName;
    isVaultOpen = vaultService.isVaultOpen;
    vaultPath = vaultService.vaultPath;
    await indexerService.reindexVault();

    // 1. Reload open tabs if updated on disk and not dirty
    for (const tab of tabs) {
      if (!tab.isDirty) {
        try {
          const freshContent = await vaultService.readFile(tab.path);
          if (freshContent !== tab.content) {
            tab.content = freshContent;
          }
        } catch {
          // File might have been removed on remote
        }
      }
    }

    // 2. Close tabs for files that were deleted on disk
    const previousTabsCount = tabs.length;
    tabs = tabs.filter(tab => !!findNodeByPath(files, tab.path));
    if (tabs.length !== previousTabsCount) {
      if (tabs.length > 0 && !tabs.some(t => t.id === activeTabId)) {
        activeTabId = tabs[0].id;
      } else if (tabs.length === 0) {
        activeTabId = '';
      }
      saveWorkspaceSession();
    }
  }

  async function loadVault() {
    await vaultService.initVault();
    vaultName = vaultService.vaultName;
    isVaultOpen = vaultService.isVaultOpen;
    vaultPath = vaultService.vaultPath;
    files = await vaultService.getFiles();
    if (files.length > 0) {
      await indexerService.reindexVault();
      if (tabs.length === 0) {
        const restored = await restoreWorkspaceSession();
        if (!restored) {
          const firstFile = findFirstFile(files);
          if (firstFile) openFile(firstFile);
        }
      }
    }

    // Check Google Drive sync
    driveSyncService.checkAuthStatus().then(async (authed) => {
      if (authed && vaultService.vaultPath) {
        try {
          await driveSyncService.syncNow(vaultService.vaultPath);
          await refreshVaultState();
        } catch (e) {
          console.warn('Initial sync error:', e);
        }
      }
    });
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
      saveWorkspaceSession();
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
    saveWorkspaceSession();
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
    saveWorkspaceSession();
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
        },
        onCancel: () => {
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
    saveWorkspaceSession();
  }

  function handleCloseOtherTabs(keepId: string) {
    const keepTab = tabs.find(t => t.id === keepId);
    if (!keepTab) return;
    tabs = [keepTab];
    activeTabId = keepTab.id;
    saveWorkspaceSession();
  }

  function handleEditorChange(filePath: string, newContent: string) {
    const targetTab = tabs.find(t => t.path === filePath);
    if (!targetTab) return;

    if (targetTab.content === newContent) return;

    // Do not mark as dirty if the difference is only whitespace or trailing newline normalization
    if (!targetTab.isDirty && targetTab.content.trim() === newContent.trim()) {
      targetTab.content = newContent;
      return;
    }

    targetTab.content = newContent;
    targetTab.isDirty = true;
  }

  async function handleSaveCurrent() {
    if (!activeTab) return;
    await vaultService.writeFile(activeTab.path, activeTab.content);
    activeTab.isDirty = false;

    // Update in-memory index
    indexerService.indexFileContent(activeTab.path, activeTab.name, activeTab.content);

    // Sync to Google Drive in background if connected
    driveSyncService.syncFileOnSave(vaultService.vaultPath, activeTab.path);
  }

  async function handleCreateFile(name: string, parentPath?: string) {
    const newFile = await vaultService.createFile(name, parentPath);
    await refreshVaultState();
    openFile(newFile);
  }

  async function handleCreateFolder(name: string, parentPath?: string) {
    await vaultService.createFolder(name, parentPath);
    await refreshVaultState();
  }

  function closeTabsUnderPath(targetPath: string, isDirectory: boolean) {
    const normTarget = targetPath.replace(/\\/g, '/').replace(/\/+$/, '');
    const remainingTabs = tabs.filter(t => {
      const normTab = t.path.replace(/\\/g, '/');
      if (isDirectory) {
        return !(normTab === normTarget || normTab.startsWith(normTarget + '/'));
      }
      return normTab !== normTarget;
    });

    if (remainingTabs.length !== tabs.length) {
      tabs = remainingTabs;
      if (!tabs.some(t => t.id === activeTabId)) {
        activeTabId = tabs.length > 0 ? tabs[tabs.length - 1].id : '';
      }
      saveWorkspaceSession();
    }
  }

  function promptDeleteNode(node: FileNode) {
    const isDir = node.isDirectory;
    modalState = {
      open: true,
      title: isDir ? 'Delete Folder' : 'Delete Note',
      description: isDir
        ? `Are you sure you want to permanently delete folder "${node.name}" and all of its contents from disk? This action cannot be undone.`
        : `Are you sure you want to permanently delete note "${node.name}" from disk? This action cannot be undone.`,
      placeholder: '',
      confirmText: isDir ? 'Delete Folder' : 'Delete Note',
      cancelText: 'Cancel',
      isAlertOnly: true,
      danger: true,
      onConfirm: async () => {
        await handleDeleteNode(node);
      }
    };
  }

  async function handleDeleteNode(node: FileNode) {
    await vaultService.deleteNode(node.path);
    closeTabsUnderPath(node.path, node.isDirectory);
    await refreshVaultState();

    // Trash on Google Drive in background if connected
    driveSyncService.syncDeleteFile(vaultService.vaultPath, node.path);
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
    // Ctrl+G: Toggle Graph View
    if ((e.ctrlKey || e.metaKey) && e.key.toLowerCase() === 'g') {
      e.preventDefault();
      isGraphOpen = !isGraphOpen;
      return;
    }
    // Escape: Close Graph View if open
    if (e.key === 'Escape' && isGraphOpen) {
      e.preventDefault();
      isGraphOpen = false;
      return;
    }
  }

  let lastSyncTimestamp = 0;

  async function handleQuickSync() {
    if (!vaultPath) return;
    if (driveSyncService.isSyncing) return;
    try {
      showToast('Syncing with Google Drive...');
      const summary = await driveSyncService.syncNow(vaultPath);
      if (summary) {
        await refreshVaultState();
        if (summary.uploaded > 0 || summary.downloaded > 0 || summary.deleted > 0) {
          showToast(`Synced: ↑${summary.uploaded} ↓${summary.downloaded} 🗑${summary.deleted}`);
        } else {
          showToast('All notes up to date');
        }
      }
    } catch (err: any) {
      showToast(`Sync error: ${err?.toString() || 'Failed'}`);
    }
  }

  async function triggerAutoSync() {
    if (!driveSyncService.isAuthenticated || !vaultPath || driveSyncService.isSyncing) return;
    const now = Date.now();
    // Throttle to at most once per 45 seconds to avoid unnecessary network calls
    if (now - lastSyncTimestamp < 45_000) return;
    lastSyncTimestamp = now;

    try {
      const summary = await driveSyncService.syncNow(vaultPath);
      if (summary && (summary.uploaded > 0 || summary.downloaded > 0 || summary.deleted > 0)) {
        await refreshVaultState();
        showToast(`Auto-synced: ↑${summary.uploaded} ↓${summary.downloaded}`);
      }
    } catch (e) {
      console.warn('Background auto-sync failed:', e);
    }
  }

  onMount(() => {
    loadVault();
    window.addEventListener('keydown', handleKeyDown);

    const onWindowFocus = () => {
      triggerAutoSync();
    };
    window.addEventListener('focus', onWindowFocus);

    const onBeforeUnload = () => {
      saveWorkspaceSession();
    };
    window.addEventListener('beforeunload', onBeforeUnload);

    // Periodic background sync every 5 minutes
    const intervalTimer = setInterval(() => {
      triggerAutoSync();
    }, 5 * 60 * 1000);

    return () => {
      window.removeEventListener('keydown', handleKeyDown);
      window.removeEventListener('focus', onWindowFocus);
      window.removeEventListener('beforeunload', onBeforeUnload);
      clearInterval(intervalTimer);
    };
  });
</script>

<div class="h-screen w-screen flex flex-col bg-zed-bg-base text-zed-text-primary overflow-hidden select-none font-sans">
  <!-- Top Navigation & Breadcrumbs -->
  <TitleBar
    {vaultName}
    currentPath={activeTab?.path ?? ''}
    {sidebarOpen}
    {rightSidebarOpen}
    {isGraphOpen}
    onToggleSidebar={() => (sidebarOpen = !sidebarOpen)}
    onToggleRightSidebar={() => (rightSidebarOpen = !rightSidebarOpen)}
    onOpenSearch={() => (searchOpen = true)}
    onNewNote={() => openCreateNoteDialog()}
    onSelectFolder={handleSelectFolder}
    onOpenSettings={() => (settingsOpen = true)}
    onQuickSync={handleQuickSync}
    onToggleGraph={() => (isGraphOpen = !isGraphOpen)}
    isSyncing={driveSyncService.isSyncing}
    isConnected={driveSyncService.isAuthenticated}
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
        onDeleteNode={promptDeleteNode}
        onNotify={showToast}
      />
    {/if}

    <!-- Center Workspace -->
    <main class="flex-1 flex flex-col overflow-hidden bg-zed-bg-base relative">
      {#if isGraphOpen}
        <GraphView
          activePath={activeTab?.path}
          onOpenFile={(path) => {
            openFileByPath(path);
            isGraphOpen = false;
          }}
          onClose={() => (isGraphOpen = false)}
        />
      {:else if tabs.length > 0}
        <TabBar
          {tabs}
          {activeTabId}
          onSelectTab={handleSelectTab}
          onCloseTab={handleCloseTab}
          onCloseOtherTabs={handleCloseOtherTabs}
          onNotify={showToast}
        />

        {#if activeTab}
          <div class="flex-1 overflow-hidden relative">
            {#key activeTab.path}
              <Editor
                bind:this={editorRef}
                content={activeTab.content}
                path={activeTab.path}
                isDirty={activeTab.isDirty}
                onChange={handleEditorChange}
                onSave={handleSaveCurrent}
                onStatsChange={(stats) => (editorStats = stats)}
              />
            {/key}
          </div>
        {/if}
      {:else}
        <!-- Empty Workspace (Zed Minimalist Style) -->
        <div class="flex-1 flex flex-col items-center justify-center text-center p-6 text-zinc-500">
          <div class="w-12 h-12 rounded-xl bg-zinc-900 border border-zinc-800 flex items-center justify-center text-zinc-400 mb-4 shadow-sm">
            <FileText size={22} />
          </div>
          <h2 class="text-sm font-semibold text-zinc-200 mb-1">
            {isVaultOpen ? vaultName : 'No Vault Open'}
          </h2>
          <p class="text-xs text-zinc-500 max-w-sm mb-6">
            {isVaultOpen 
              ? 'Select a note from the project panel or create a new file to start writing.'
              : 'Choose a folder on your computer to store and edit real markdown files.'}
          </p>

          {#if !isVaultOpen}
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
        currentContent={activeTab?.content ?? ''}
        onOpenFile={openFileByPath}
        onScrollToHeading={handleScrollToHeading}
      />
    {/if}
  </div>

  <!-- Bottom Status Bar -->
  <StatusBar
    stats={editorStats}
    {vaultName}
    isSyncing={driveSyncService.isSyncing}
    isDirty={activeTab?.isDirty ?? false}
  />

  <!-- Settings Modal (Google Drive OAuth & Sync) -->
  <SettingsModal
    open={settingsOpen}
    vaultPath={vaultPath}
    onClose={() => (settingsOpen = false)}
    onNotify={showToast}
    onSyncComplete={async () => {
      await refreshVaultState();
    }}
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
    danger={modalState.danger}
    onConfirm={modalState.onConfirm}
    onCancel={modalState.onCancel}
    onClose={() => (modalState.open = false)}
  />

  <!-- Quick Search Modal (Command Palette Ctrl+P) -->
  <QuickSearchModal
    open={searchOpen}
    onClose={() => (searchOpen = false)}
    onSelectFile={openFileByPath}
  />

  <!-- Toast Notification (Copied to clipboard, etc.) -->
  {#if toastMessage}
    <div class="fixed bottom-9 right-4 z-50 flex items-center space-x-2 px-3 py-1.5 bg-zinc-900/95 border border-zinc-700 text-zinc-200 text-xs rounded-lg shadow-2xl pointer-events-none transition-all">
      <Check size={13} class="text-emerald-400" />
      <span class="truncate max-w-sm font-medium">{toastMessage}</span>
    </div>
  {/if}
</div>
