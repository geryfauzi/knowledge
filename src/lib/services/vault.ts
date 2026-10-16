import type { FileNode } from '../types';

export const isTauri = typeof window !== 'undefined' && '__TAURI_INTERNALS__' in window;

class VaultService {
  public vaultName: string = 'No Folder Open';
  public vaultPath: string = '';
  public isVaultOpen: boolean = false;
  private vaultTree: FileNode[] = [];

  // Browser File System Access API handles (when running in web mode)
  private dirHandle: FileSystemDirectoryHandle | null = null;
  private fileHandles: Map<string, FileSystemFileHandle> = new Map();
  private dirHandles: Map<string, FileSystemDirectoryHandle> = new Map();

  /**
   * Prompt user to pick a REAL folder on their hard drive
   */
  async selectVaultFolder(): Promise<boolean> {
    if (isTauri) {
      try {
        const { open } = await import('@tauri-apps/plugin-dialog');
        const selected = await open({
          directory: true,
          multiple: false,
          title: 'Select Knowledge Vault Folder'
        });

        if (selected && typeof selected === 'string') {
          return await this.openVaultByPath(selected);
        }
      } catch (err) {
        console.error('Failed to open folder via Tauri dialog:', err);
      }
      return false;
    }

    // Web Browser: Native File System Access API (writes real files on disk!)
    if (typeof window !== 'undefined' && 'showDirectoryPicker' in window) {
      try {
        // @ts-ignore
        const handle = await window.showDirectoryPicker({
          mode: 'readwrite'
        });

        this.dirHandle = handle;
        this.vaultName = handle.name;
        this.vaultPath = `/${handle.name}`;
        this.isVaultOpen = true;

        await this.scanBrowserDirectory(handle);
        return true;
      } catch (err: any) {
        if (err.name !== 'AbortError') {
          console.error('Failed to select directory via File System Access API:', err);
        }
        return false;
      }
    }

    return false;
  }

  /**
   * Open an existing vault folder on disk and persist as last used vault
   */
  async openVaultByPath(folderPath: string): Promise<boolean> {
    if (isTauri) {
      try {
        const normalized = folderPath.replace(/\\/g, '/').replace(/\/+$/, '');
        this.vaultPath = normalized;
        this.vaultName = normalized.split('/').filter(Boolean).pop() || 'Vault';
        this.isVaultOpen = true;
        await this.scanTauriFolder(normalized);
        if (typeof window !== 'undefined' && window.localStorage) {
          localStorage.setItem('knowledge_last_vault', normalized);
        }
        return true;
      } catch (err) {
        console.error('Failed to open vault by path:', err);
        return false;
      }
    }
    return false;
  }

  /**
   * Automatically initializes vault: checks last used vault,
   * or provisions a default vault in ~/Documents/Knowledge
   */
  async initVault(): Promise<boolean> {
    if (!isTauri) {
      return false;
    }

    try {
      // 1. Try restoring last used vault from localStorage
      const lastVault = typeof window !== 'undefined' ? localStorage.getItem('knowledge_last_vault') : null;
      if (lastVault) {
        try {
          const { readDir } = await import('@tauri-apps/plugin-fs');
          await readDir(lastVault);
          return await this.openVaultByPath(lastVault);
        } catch (e) {
          console.warn('Last vault not accessible, falling back to default vault:', e);
        }
      }

      // 2. Provision default vault in user Documents directory (~/Documents/Knowledge)
      const { documentDir, homeDir, join } = await import('@tauri-apps/api/path');
      const { mkdir, exists, writeTextFile } = await import('@tauri-apps/plugin-fs');

      let docDir = '';
      try {
        docDir = await documentDir();
      } catch {
        docDir = await homeDir();
      }

      const defaultVaultPath = (await join(docDir, 'Knowledge')).replace(/\\/g, '/').replace(/\/+$/, '');

      // Ensure directory exists
      await mkdir(defaultVaultPath, { recursive: true });

      // Create starter Welcome note if empty
      const welcomePath = `${defaultVaultPath}/Welcome.md`;
      const welcomeExists = await exists(welcomePath).catch(() => false);
      if (!welcomeExists) {
        const welcomeContent = `# Welcome to Knowledge\n\nYour personal knowledge base is ready.\n\n- **Physical Markdown Files**: All notes are stored as real Markdown files in \`${defaultVaultPath}\`.\n- **Dual-Mode Editor**: Seamlessly switch between Markdown and WYSIWYG mode using the toolbar.\n- **Wikilinks**: Type \`[[\` to link notes together.\n- **Quick Search**: Press \`Ctrl + P\` to open the command palette.\n- **Keyboard Shortcuts**:\n  - \`Ctrl + N\`: New note\n  - \`Ctrl + S\`: Save note\n  - \`Ctrl + \\\`: Toggle project panel\n`;
        await writeTextFile(welcomePath, welcomeContent);
      }

      return await this.openVaultByPath(defaultVaultPath);
    } catch (err) {
      console.error('Failed to initialize default vault:', err);
      return false;
    }
  }

  /**
   * Safely resolves a child file/folder path inside the vault
   */
  public resolveChildPath(childName: string, parentPath: string = ''): string {
    let base = parentPath && parentPath.trim() !== '' && parentPath !== '/'
      ? parentPath
      : this.vaultPath;
    base = base.replace(/\\/g, '/').replace(/\/+$/, '');
    return `${base}/${childName}`;
  }

  // --- TAURI NATIVE FILESYSTEM IMPLEMENTATION ---

  private async scanTauriFolder(folderPath: string) {
    try {
      const normalized = folderPath.replace(/\\/g, '/').replace(/\/+$/, '');
      const { readDir } = await import('@tauri-apps/plugin-fs');
      const entries = await readDir(normalized);

      const nodes: FileNode[] = [];
      for (const entry of entries) {
        if (entry.name.startsWith('.') || entry.name === 'node_modules' || entry.name === 'target') {
          continue;
        }

        const childPath = `${normalized}/${entry.name}`;
        if (entry.isDirectory) {
          nodes.push({
            path: childPath,
            name: entry.name,
            isDirectory: true,
            children: await this.scanTauriFolderRecursive(childPath)
          });
        } else if (entry.name.endsWith('.md')) {
          nodes.push({
            path: childPath,
            name: entry.name,
            isDirectory: false
          });
        }
      }
      this.vaultTree = nodes.sort((a, b) => {
        if (a.isDirectory === b.isDirectory) return a.name.localeCompare(b.name);
        return a.isDirectory ? -1 : 1;
      });
    } catch (err) {
      console.error('Error reading Tauri directory:', err);
      this.vaultTree = [];
    }
  }

  private async scanTauriFolderRecursive(folderPath: string): Promise<FileNode[]> {
    try {
      const normalized = folderPath.replace(/\\/g, '/').replace(/\/+$/, '');
      const { readDir } = await import('@tauri-apps/plugin-fs');
      const entries = await readDir(normalized);
      const nodes: FileNode[] = [];

      for (const entry of entries) {
        if (entry.name.startsWith('.') || entry.name === 'node_modules' || entry.name === 'target') {
          continue;
        }

        const childPath = `${normalized}/${entry.name}`;
        if (entry.isDirectory) {
          nodes.push({
            path: childPath,
            name: entry.name,
            isDirectory: true,
            children: await this.scanTauriFolderRecursive(childPath)
          });
        } else if (entry.name.endsWith('.md')) {
          nodes.push({
            path: childPath,
            name: entry.name,
            isDirectory: false
          });
        }
      }
      return nodes.sort((a, b) => {
        if (a.isDirectory === b.isDirectory) return a.name.localeCompare(b.name);
        return a.isDirectory ? -1 : 1;
      });
    } catch (err) {
      console.warn(`Could not read subdirectory "${folderPath}":`, err);
      return [];
    }
  }

  // --- BROWSER NATIVE FILE SYSTEM ACCESS API IMPLEMENTATION ---

  private async scanBrowserDirectory(dirHandle: FileSystemDirectoryHandle) {
    this.fileHandles.clear();
    this.dirHandles.clear();
    this.dirHandles.set('/', dirHandle);

    this.vaultTree = await this.readDirectoryHandle(dirHandle, '');
  }

  private async readDirectoryHandle(
    dirHandle: FileSystemDirectoryHandle,
    currentPath: string
  ): Promise<FileNode[]> {
    const nodes: FileNode[] = [];

    // @ts-ignore
    for await (const [name, handle] of dirHandle.entries()) {
      if (name.startsWith('.') || name === 'node_modules' || name === 'target') {
        continue;
      }

      const fullPath = `${currentPath}/${name}`;

      if (handle.kind === 'directory') {
        this.dirHandles.set(fullPath, handle);
        nodes.push({
          path: fullPath,
          name,
          isDirectory: true,
          children: await this.readDirectoryHandle(handle, fullPath)
        });
      } else if (name.endsWith('.md')) {
        this.fileHandles.set(fullPath, handle);
        nodes.push({
          path: fullPath,
          name,
          isDirectory: false
        });
      }
    }

    return nodes.sort((a, b) => {
      if (a.isDirectory === b.isDirectory) return a.name.localeCompare(b.name);
      return a.isDirectory ? -1 : 1;
    });
  }

  // --- COMMON OPERATIONS (PHYSICAL DISK ACCESS) ---

  /**
   * Rescans vault files directly from disk and updates in-memory vaultTree
   */
  async rescanVault(): Promise<FileNode[]> {
    if (isTauri && this.vaultPath) {
      await this.scanTauriFolder(this.vaultPath);
    } else if (this.dirHandle) {
      await this.scanBrowserDirectory(this.dirHandle);
    }
    return this.vaultTree;
  }

  async getFiles(forceRescan: boolean = false): Promise<FileNode[]> {
    if (forceRescan) {
      return await this.rescanVault();
    }
    return this.vaultTree;
  }

  async readFile(path: string): Promise<string> {
    if (isTauri) {
      const { readTextFile } = await import('@tauri-apps/plugin-fs');
      return await readTextFile(path);
    }

    const fileHandle = this.fileHandles.get(path);
    if (fileHandle) {
      const file = await fileHandle.getFile();
      return await file.text();
    }
    return '';
  }

  async writeFile(path: string, content: string): Promise<void> {
    if (isTauri) {
      const { writeTextFile } = await import('@tauri-apps/plugin-fs');
      await writeTextFile(path, content);
      return;
    }

    const fileHandle = this.fileHandles.get(path);
    if (fileHandle) {
      // @ts-ignore
      const writable = await fileHandle.createWritable();
      await writable.write(content);
      await writable.close();
    }
  }

  async createFile(name: string, parentPath: string = ''): Promise<FileNode> {
    const filename = name.endsWith('.md') ? name : `${name}.md`;
    const fullPath = this.resolveChildPath(filename, parentPath);
    const initialContent = `# ${filename.replace('.md', '')}\n\n`;

    if (isTauri) {
      try {
        const { writeTextFile, mkdir } = await import('@tauri-apps/plugin-fs');
        const lastSlash = fullPath.lastIndexOf('/');
        if (lastSlash > 0) {
          const parentDir = fullPath.substring(0, lastSlash);
          await mkdir(parentDir, { recursive: true });
        }
        await writeTextFile(fullPath, initialContent);
        await this.scanTauriFolder(this.vaultPath);
        const baseName = filename.split('/').pop() || filename;
        return {
          path: fullPath,
          name: baseName,
          isDirectory: false,
          content: initialContent
        };
      } catch (err) {
        console.error(`Failed to create file at "${fullPath}":`, err);
        throw err;
      }
    }

    // Browser File System Access API
    const cleanParent = parentPath && parentPath !== '/' ? parentPath.replace(/\/+$/, '') : '';
    const targetDir = this.dirHandles.get(cleanParent || '/') || this.dirHandle;
    if (targetDir) {
      // @ts-ignore
      const newFileHandle = await targetDir.getFileHandle(filename, { create: true });
      // @ts-ignore
      const writable = await newFileHandle.createWritable();
      await writable.write(initialContent);
      await writable.close();

      this.fileHandles.set(fullPath, newFileHandle);
      if (this.dirHandle) {
        await this.scanBrowserDirectory(this.dirHandle);
      }
    }

    return {
      path: fullPath,
      name: filename,
      isDirectory: false,
      content: initialContent
    };
  }

  async createFolder(name: string, parentPath: string = ''): Promise<FileNode> {
    const fullPath = this.resolveChildPath(name, parentPath);

    if (isTauri) {
      try {
        const { mkdir } = await import('@tauri-apps/plugin-fs');
        await mkdir(fullPath, { recursive: true });
        await this.scanTauriFolder(this.vaultPath);
      } catch (err) {
        console.error(`Failed to create folder at "${fullPath}":`, err);
        throw err;
      }
    } else {
      const cleanParent = parentPath && parentPath !== '/' ? parentPath.replace(/\/+$/, '') : '';
      const targetDir = this.dirHandles.get(cleanParent || '/') || this.dirHandle;
      if (targetDir) {
        // @ts-ignore
        const newDirHandle = await targetDir.getDirectoryHandle(name, { create: true });
        this.dirHandles.set(fullPath, newDirHandle);
        if (this.dirHandle) {
          await this.scanBrowserDirectory(this.dirHandle);
        }
      }
    }

    return {
      path: fullPath,
      name,
      isDirectory: true,
      children: []
    };
  }

  async deleteNode(path: string): Promise<void> {
    if (isTauri) {
      try {
        const { remove } = await import('@tauri-apps/plugin-fs');
        await remove(path, { recursive: true });
        await this.scanTauriFolder(this.vaultPath);
        return;
      } catch (err) {
        console.error(`Failed to delete node at "${path}":`, err);
        throw err;
      }
    }

    // In Browser
    const parts = path.split('/').filter(Boolean);
    const itemName = parts.pop();
    const parentPath = '/' + parts.join('/');
    const parentDir = this.dirHandles.get(parentPath || '/') || this.dirHandle;

    if (parentDir && itemName) {
      // @ts-ignore
      await parentDir.removeEntry(itemName, { recursive: true });
      this.fileHandles.delete(path);
      this.dirHandles.delete(path);
      if (this.dirHandle) {
        await this.scanBrowserDirectory(this.dirHandle);
      }
    }
  }
}

export const vaultService = new VaultService();
