import type { FileNode } from '../types';
import { vaultService } from './vault';

export interface SearchResult {
  path: string;
  name: string;
  matches: string[];
}

export interface BacklinkItem {
  sourcePath: string;
  sourceName: string;
  context: string;
}

class IndexerService {
  // Map: Target Note Name (lowercase) -> Set of source file paths that link to it
  private backlinks: Map<string, Set<string>> = new Map();
  // Map: Tag Name (e.g. "project") -> Set of file paths containing it
  private tagIndex: Map<string, Set<string>> = new Map();
  // List of all known markdown note titles for autocomplete
  public noteTitles: { title: string; path: string }[] = [];

  /**
   * Reindex all markdown notes in the vault
   */
  async reindexVault(): Promise<void> {
    this.backlinks.clear();
    this.tagIndex.clear();
    this.noteTitles = [];

    const files = await vaultService.getFiles();
    await this.traverseAndIndex(files);
  }

  private async traverseAndIndex(nodes: FileNode[]) {
    for (const node of nodes) {
      if (node.isDirectory && node.children) {
        await this.traverseAndIndex(node.children);
      } else if (!node.isDirectory && node.name.endsWith('.md')) {
        const cleanTitle = node.name.replace(/\.md$/, '');
        this.noteTitles.push({ title: cleanTitle, path: node.path });

        const content = await vaultService.readFile(node.path);
        this.indexFileContent(node.path, cleanTitle, content);
      }
    }
  }

  /**
   * Index wikilinks and tags from a file
   */
  indexFileContent(filePath: string, fileTitle: string, content: string): void {
    // 1. Extract Wikilinks: [[Target Note]] or [[Target Note|Custom Label]]
    const wikilinkRegex = /\[\[([^\]|#]+)(?:[#|][^\]]+)?\]\]/g;
    let match: RegExpExecArray | null;

    while ((match = wikilinkRegex.exec(content)) !== null) {
      const targetTitle = match[1].trim().toLowerCase();
      if (!this.backlinks.has(targetTitle)) {
        this.backlinks.set(targetTitle, new Set());
      }
      this.backlinks.get(targetTitle)!.add(filePath);
    }

    // 2. Extract Tags: #tag (excluding markdown headings like # Heading)
    const tagRegex = /(?:^|\s)#([a-zA-Z0-9_\-]+)(?=\s|$|[.,;:!?])/g;
    while ((match = tagRegex.exec(content)) !== null) {
      const tag = match[1].toLowerCase();
      if (!this.tagIndex.has(tag)) {
        this.tagIndex.set(tag, new Set());
      }
      this.tagIndex.get(tag)!.add(filePath);
    }
  }

  /**
   * Get all notes linking to the given note
   */
  async getBacklinksForFile(filePath: string, fileName: string): Promise<BacklinkItem[]> {
    const cleanTitle = fileName.replace(/\.md$/, '').toLowerCase();
    const sources = this.backlinks.get(cleanTitle);
    if (!sources || sources.size === 0) return [];

    const results: BacklinkItem[] = [];

    for (const sourcePath of sources) {
      if (sourcePath === filePath) continue; // Skip self references

      const content = await vaultService.readFile(sourcePath);
      const sourceName = sourcePath.split('/').pop() || sourcePath;

      // Extract a preview sentence containing the link
      const lines = content.split('\n');
      const matchingLine = lines.find(l => l.toLowerCase().includes(`[[${cleanTitle}`)) || lines[0] || '';

      results.push({
        sourcePath,
        sourceName,
        context: matchingLine.trim().slice(0, 120)
      });
    }

    return results;
  }

  /**
   * Get all indexed tags with their file counts
   */
  getAllTags(): { tag: string; count: number; files: string[] }[] {
    const list: { tag: string; count: number; files: string[] }[] = [];
    for (const [tag, fileSet] of this.tagIndex.entries()) {
      list.push({
        tag,
        count: fileSet.size,
        files: Array.from(fileSet)
      });
    }
    return list.sort((a, b) => b.count - a.count);
  }

  /**
   * Resolve a wikilink text to a real file path
   */
  resolveLink(linkText: string): string | null {
    const clean = linkText.trim().toLowerCase();
    const found = this.noteTitles.find(n => n.title.toLowerCase() === clean);
    return found ? found.path : null;
  }

  /**
   * Fast full-text search across all notes in the vault
   */
  async search(query: string): Promise<SearchResult[]> {
    if (!query.trim()) return [];

    const lower = query.toLowerCase();
    const results: SearchResult[] = [];
    const files = await vaultService.getFiles();

    const searchRecursive = async (nodes: FileNode[]) => {
      for (const node of nodes) {
        if (node.isDirectory && node.children) {
          await searchRecursive(node.children);
        } else if (!node.isDirectory && node.name.endsWith('.md')) {
          const content = await vaultService.readFile(node.path);
          const nameMatches = node.name.toLowerCase().includes(lower);
          const lines = content.split('\n');
          const matchingLines = lines
            .filter(l => l.toLowerCase().includes(lower))
            .map(l => l.trim().slice(0, 100))
            .slice(0, 3);

          if (nameMatches || matchingLines.length > 0) {
            results.push({
              path: node.path,
              name: node.name,
              matches: matchingLines.length > 0 ? matchingLines : ['Matches in title']
            });
          }
        }
      }
    };

    await searchRecursive(files);
    return results;
  }
}

export const indexerService = new IndexerService();
