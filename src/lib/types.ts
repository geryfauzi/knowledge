export interface FileNode {
  path: string;
  name: string;
  isDirectory: boolean;
  children?: FileNode[];
  content?: string;
  updatedAt?: number;
}

export interface TabItem {
  id: string;
  path: string;
  name: string;
  content: string;
  isDirty: boolean;
  cursorLine?: number;
  cursorCol?: number;
}

export interface EditorStats {
  line: number;
  col: number;
  characters: number;
  words: number;
  selectedChars: number;
}
