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

  let scrollContainer = $state<HTMLDivElement | null>(null);
  let element = $state<HTMLDivElement | null>(null);
  let editor: Editor | null = null;
  let currentPath = $state('');
  let currentHtml = $state('');
  let isSelfUpdating = false;
  let isNavigatingOutline = false;

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

  export function scrollToHeading(headingText: string, headingIndex?: number) {
    if (!element) return;
    const headingElements = Array.from(element.querySelectorAll<HTMLElement>('h1, h2, h3, h4, h5, h6'));
    if (headingElements.length === 0) return;

    let target: HTMLElement | null = null;
    if (headingIndex !== undefined && headingIndex >= 0 && headingIndex < headingElements.length) {
      target = headingElements[headingIndex];
    }

    if (!target) {
      const stripMd = (s: string) => s.replace(/[*_`#~\[\]\(\)]/g, '').replace(/\s+/g, ' ').trim().toLowerCase();
      const cleanTarget = stripMd(headingText);

      for (const h of headingElements) {
        const hText = stripMd(h.textContent || '');
        if (hText && (hText.includes(cleanTarget) || cleanTarget.includes(hText))) {
          target = h;
          break;
        }
      }
    }

    if (!target) return;

    isNavigatingOutline = true;

    if (editor) {
      try {
        let targetPos = -1;
        let currentIdx = 0;
        editor.state.doc.descendants((node, pos) => {
          if (targetPos >= 0) return false;
          if (node.type.name === 'heading') {
            if (headingIndex !== undefined && headingIndex === currentIdx) {
              targetPos = pos + 1;
              return false;
            }
            currentIdx++;
          }
        });

        if (targetPos >= 0) {
          editor.commands.setTextSelection(targetPos);
          editor.commands.focus();
        } else {
          editor.commands.focus();
        }
      } catch {
        editor.commands.focus();
      }
    }

    const doScroll = () => {
      if (scrollContainer && target) {
        const containerRect = scrollContainer.getBoundingClientRect();
        const targetRect = target.getBoundingClientRect();
        const targetScrollTop = scrollContainer.scrollTop + (targetRect.top - containerRect.top) - 24;
        scrollContainer.scrollTo({
          top: Math.max(0, targetScrollTop),
          behavior: 'smooth'
        });
      }
    };

    doScroll();
    setTimeout(() => {
      doScroll();
      isNavigatingOutline = false;
    }, 150);
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
        if (isNavigatingOutline) return;
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

  // Watch for external content changes (e.g. mode toggle or sync)
  $effect(() => {
    if (!editor) return;

    // Never reset content if update originated from typing inside editor
    if (isSelfUpdating) return;

    // External reload for same file (e.g. from mode toggle or sync)
    if (htmlContent !== currentHtml && editor.getHTML() !== htmlContent) {
      currentHtml = htmlContent;
      editor.commands.setContent(htmlContent, { emitUpdate: false });
      calculateStats();
    }
  });
</script>

<div bind:this={scrollContainer} class="h-full w-full bg-zed-bg-base overflow-y-auto wysiwyg-container select-text">
  <div bind:this={element}></div>
</div>
