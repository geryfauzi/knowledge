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
          this.vaultPath = selected;
          this.vaultName = selected.split(/[\\/]/).filter(Boolean).pop() || 'Vault';
          this.isVaultOpen = true;
          await this.scanTauriFolder(selected);
          return true;
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

  // --- TAURI NATIVE FILESYSTEM IMPLEMENTATION ---

  private async scanTauriFolder(folderPath: string) {
    try {
      const { readDir } = await import('@tauri-apps/plugin-fs');
      const entries = await readDir(folderPath);

      const nodes: FileNode[] = [];
      for (const entry of entries) {
        const childPath = `${folderPath}/${entry.name}`;
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
      this.vaultTree = nodes;
    } catch (err) {
      console.error('Error reading Tauri directory:', err);
      this.vaultTree = [];
    }
  }

  private async scanTauriFolderRecursive(folderPath: string): Promise<FileNode[]> {
    const { readDir } = await import('@tauri-apps/plugin-fs');
    const entries = await readDir(folderPath);
    const nodes: FileNode[] = [];

    for (const entry of entries) {
      const childPath = `${folderPath}/${entry.name}`;
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
    return nodes;
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

  async getFiles(): Promise<FileNode[]> {
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
    const cleanParent = parentPath.endsWith('/') && parentPath.length > 1
      ? parentPath.slice(0, -1)
      : parentPath;
    const fullPath = `${cleanParent}/${filename}`;
    const initialContent = `# ${filename.replace('.md', '')}\n\n`;

    if (isTauri) {
      const { writeTextFile } = await import('@tauri-apps/plugin-fs');
      await writeTextFile(fullPath, initialContent);
      await this.scanTauriFolder(this.vaultPath);
      return {
        path: fullPath,
        name: filename,
        isDirectory: false,
        content: initialContent
      };
    }

    // Browser File System Access API
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
    const cleanParent = parentPath.endsWith('/') && parentPath.length > 1
      ? parentPath.slice(0, -1)
      : parentPath;
    const fullPath = `${cleanParent}/${name}`;

    if (isTauri) {
      const { mkdir } = await import('@tauri-apps/plugin-fs');
      await mkdir(fullPath);
      await this.scanTauriFolder(this.vaultPath);
    } else {
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
      const { remove } = await import('@tauri-apps/plugin-fs');
      await remove(path);
      await this.scanTauriFolder(this.vaultPath);
      return;
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
