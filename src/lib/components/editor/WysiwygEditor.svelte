<script lang="ts">
  import { onMount, onDestroy } from 'svelte';
  import { Editor } from '@tiptap/core';
  import StarterKit from '@tiptap/starter-kit';
  import Underline from '@tiptap/extension-underline';
  import TextAlign from '@tiptap/extension-text-align';
  import type { EditorStats } from '../../types';

  interface Props {
    htmlContent: string;
    path: string;
    onChange: (html: string) => void;
    onStatsChange: (stats: EditorStats) => void;
    onActiveStatesChange: (states: Record<string, boolean>) => void;
  }

  let {
    htmlContent,
    path,
    onChange,
    onStatsChange,
    onActiveStatesChange
  }: Props = $props();

  let element = $state<HTMLDivElement | null>(null);
  let editor: Editor | null = null;
  let currentPath = $state('');
  let currentHtml = $state('');
  let isSelfUpdating = false;

  export function executeAction(action: string, param?: any) {
    if (!editor) return;

    const chain = editor.chain().focus();

    switch (action) {
      case 'bold':
        chain.toggleBold().run();
        break;
      case 'italic':
        chain.toggleItalic().run();
        break;
      case 'underline':
        chain.toggleUnderline().run();
        break;
      case 'strike':
        chain.toggleStrike().run();
        break;
      case 'code':
        chain.toggleCode().run();
        break;
      case 'heading':
        chain.toggleHeading({ level: param }).run();
        break;
      case 'align':
        chain.setTextAlign(param).run();
        break;
      case 'bulletList':
        chain.toggleBulletList().run();
        break;
      case 'orderedList':
        chain.toggleOrderedList().run();
        break;
      case 'blockquote':
        chain.toggleBlockquote().run();
        break;
      case 'horizontalRule':
        chain.setHorizontalRule().run();
        break;
      case 'wikilink':
        chain.insertContent('[[' + (param || 'Note') + ']]').run();
        break;
    }
    updateActiveStates();
  }

  function updateActiveStates() {
    if (!editor) return;
    onActiveStatesChange({
      bold: editor.isActive('bold'),
      italic: editor.isActive('italic'),
      underline: editor.isActive('underline'),
      strike: editor.isActive('strike'),
      code: editor.isActive('code'),
      heading1: editor.isActive('heading', { level: 1 }),
      heading2: editor.isActive('heading', { level: 2 }),
      heading3: editor.isActive('heading', { level: 3 }),
      bulletList: editor.isActive('bulletList'),
      orderedList: editor.isActive('orderedList'),
      blockquote: editor.isActive('blockquote'),
      alignLeft: editor.isActive({ textAlign: 'left' }),
      alignCenter: editor.isActive({ textAlign: 'center' }),
      alignRight: editor.isActive({ textAlign: 'right' })
    });
  }

  function calculateStats() {
    if (!editor) return;
    const text = editor.getText();
    const words = (text.match(/\b\w+\b/g) || []).length;
    const characters = text.length;

    onStatsChange({
      line: 1,
      col: 1,
      words,
      characters,
      selectedChars: 0
    });
  }

  onMount(() => {
    if (!element) return;

    currentPath = path;
    currentHtml = htmlContent;

    editor = new Editor({
      element,
      extensions: [
        StarterKit,
        Underline,
        TextAlign.configure({
          types: ['heading', 'paragraph']
        })
      ],
      content: htmlContent,
      editorProps: {
        attributes: {
          class: 'prose-invert focus:outline-none min-h-[400px] max-w-[850px] mx-auto px-6 py-6 text-sm text-zinc-300 leading-relaxed font-sans'
        }
      },
      onTransaction: () => {
        updateActiveStates();
      },
      onUpdate: ({ editor: ed }) => {
        const newHtml = ed.getHTML();
        isSelfUpdating = true;
        currentHtml = newHtml;
        onChange(newHtml);
        calculateStats();
        queueMicrotask(() => {
          isSelfUpdating = false;
        });
      }
    });

    calculateStats();
    updateActiveStates();
  });

  onDestroy(() => {
    if (editor) {
      editor.destroy();
      editor = null;
    }
  });

  // Watch for external content changes (e.g. switching tabs)
  $effect(() => {
    if (!editor) return;

    // Never reset content if update originated from typing inside editor
    if (isSelfUpdating) return;

    // File switched (different path)
    if (path !== currentPath) {
      currentPath = path;
      currentHtml = htmlContent;
      editor.commands.setContent(htmlContent, { emitUpdate: false });
      calculateStats();
      return;
    }

    // External reload for same file (e.g. from mode toggle or sync)
    if (htmlContent !== currentHtml && editor.getHTML() !== htmlContent) {
      currentHtml = htmlContent;
      editor.commands.setContent(htmlContent, { emitUpdate: false });
      calculateStats();
    }
  });
</script>

<div class="h-full w-full bg-zed-bg-base overflow-y-auto wysiwyg-container select-text">
  <div bind:this={element}></div>
</div>

<style>
  :global(.wysiwyg-container .tiptap h1) {
    font-size: 1.75rem;
    font-weight: 700;
    color: #f4f4f5;
    margin-top: 1.5rem;
    margin-bottom: 0.75rem;
    line-height: 1.25;
  }
  :global(.wysiwyg-container .tiptap h2) {
    font-size: 1.4rem;
    font-weight: 600;
    color: #e4e4e7;
    margin-top: 1.25rem;
    margin-bottom: 0.5rem;
    line-height: 1.3;
  }
  :global(.wysiwyg-container .tiptap h3) {
    font-size: 1.15rem;
    font-weight: 600;
    color: #d4d4d8;
    margin-top: 1rem;
    margin-bottom: 0.5rem;
  }
  :global(.wysiwyg-container .tiptap p) {
    margin-top: 0.5rem;
    margin-bottom: 0.5rem;
    color: #d4d4d8;
    line-height: 1.65;
  }
  :global(.wysiwyg-container .tiptap blockquote) {
    border-left: 2px solid #38bdf8;
    padding-left: 1rem;
    margin-top: 1rem;
    margin-bottom: 1rem;
    color: #a1a1aa;
    font-style: italic;
  }
  :global(.wysiwyg-container .tiptap ul) {
    list-style-type: disc;
    padding-left: 1.5rem;
    margin-top: 0.5rem;
    margin-bottom: 0.5rem;
  }
  :global(.wysiwyg-container .tiptap ol) {
    list-style-type: decimal;
    padding-left: 1.5rem;
    margin-top: 0.5rem;
    margin-bottom: 0.5rem;
  }
  :global(.wysiwyg-container .tiptap li) {
    margin-top: 0.25rem;
    margin-bottom: 0.25rem;
  }
  :global(.wysiwyg-container .tiptap code) {
    background-color: rgba(255, 255, 255, 0.08);
    color: #fcd34d;
    padding: 0.15rem 0.35rem;
    border-radius: 0.25rem;
    font-family: 'JetBrains Mono', monospace;
    font-size: 0.82rem;
  }
  :global(.wysiwyg-container .tiptap pre) {
    background-color: #121215;
    border: 1px solid #27272a;
    border-radius: 0.375rem;
    padding: 0.75rem 1rem;
    margin-top: 0.75rem;
    margin-bottom: 0.75rem;
    overflow-x: auto;
  }
  :global(.wysiwyg-container .tiptap pre code) {
    background: transparent;
    color: #f4f4f5;
    padding: 0;
  }
  :global(.wysiwyg-container .tiptap hr) {
    border: none;
    border-top: 1px solid #27272a;
    margin: 1.5rem 0;
  }
</style>
