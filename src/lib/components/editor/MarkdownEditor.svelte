<script lang="ts">
  import { onMount, onDestroy } from 'svelte';
  import { EditorView, keymap, lineNumbers, highlightActiveLineGutter, highlightActiveLine } from '@codemirror/view';
  import { EditorState, Prec } from '@codemirror/state';
  import { defaultKeymap, history, historyKeymap, indentWithTab } from '@codemirror/commands';
  import { markdown } from '@codemirror/lang-markdown';
  import { zedThemeExtension } from '../../editor/zedTheme';
  import { createWikilinkAutocomplete } from '../../editor/wikilinkExtension';
  import type { EditorStats } from '../../types';

  interface Props {
    content: string;
    path: string;
    onChange: (newContent: string) => void;
    onSave: () => void;
    onStatsChange: (stats: EditorStats) => void;
  }

  let {
    content,
    path,
    onChange,
    onSave,
    onStatsChange
  }: Props = $props();

  let editorElement = $state<HTMLDivElement | null>(null);
  let view: EditorView | null = null;
  let currentPath = $state('');
  let isInternalUpdate = false;

  export function executeAction(action: string, param?: any) {
    if (!view) return;

    const state = view.state;
    const { from, to } = state.selection.main;
    const selectedText = state.sliceDoc(from, to);

    let insertText = '';
    let newCursorPos = to;

    switch (action) {
      case 'bold':
        insertText = `**${selectedText || 'bold text'}**`;
        newCursorPos = selectedText ? from + insertText.length : from + 2;
        break;
      case 'italic':
        insertText = `*${selectedText || 'italic text'}*`;
        newCursorPos = selectedText ? from + insertText.length : from + 1;
        break;
      case 'underline':
        insertText = `<u>${selectedText || 'underlined'}</u>`;
        newCursorPos = selectedText ? from + insertText.length : from + 3;
        break;
      case 'strike':
        insertText = `~~${selectedText || 'strikethrough'}~~`;
        newCursorPos = selectedText ? from + insertText.length : from + 2;
        break;
      case 'code':
        insertText = `\`${selectedText || 'code'}\``;
        newCursorPos = selectedText ? from + insertText.length : from + 1;
        break;
      case 'wikilink':
        insertText = `[[${selectedText || ''}`;
        newCursorPos = from + insertText.length;
        break;
      case 'heading': {
        const line = state.doc.lineAt(from);
        const hashes = '#'.repeat(Number(param) || 1) + ' ';
        view.dispatch({
          changes: { from: line.from, to: line.from, insert: hashes },
          selection: { anchor: from + hashes.length }
        });
        return;
      }
      case 'bulletList': {
        const line = state.doc.lineAt(from);
        view.dispatch({
          changes: { from: line.from, to: line.from, insert: '- ' },
          selection: { anchor: from + 2 }
        });
        return;
      }
      case 'orderedList': {
        const line = state.doc.lineAt(from);
        view.dispatch({
          changes: { from: line.from, to: line.from, insert: '1. ' },
          selection: { anchor: from + 3 }
        });
        return;
      }
      case 'blockquote': {
        const line = state.doc.lineAt(from);
        view.dispatch({
          changes: { from: line.from, to: line.from, insert: '> ' },
          selection: { anchor: from + 2 }
        });
        return;
      }
      case 'horizontalRule':
        insertText = '\n---\n';
        break;
      default:
        return;
    }

    if (insertText) {
      view.dispatch({
        changes: { from, to, insert: insertText },
        selection: { anchor: newCursorPos }
      });
      view.focus();
    }
  }

  export function scrollToHeading(headingText: string, lineIndex?: number) {
    if (!view) return;
    const doc = view.state.doc;
    let targetPos = -1;

    if (lineIndex !== undefined && lineIndex > 0 && lineIndex <= doc.lines) {
      targetPos = doc.line(lineIndex).from;
    } else {
      const cleanTarget = headingText.trim().toLowerCase();
      for (let i = 1; i <= doc.lines; i++) {
        const line = doc.line(i);
        if (line.text.toLowerCase().includes(cleanTarget)) {
          targetPos = line.from;
          break;
        }
      }
    }

    if (targetPos >= 0) {
      view.dispatch({
        selection: { anchor: targetPos },
        effects: EditorView.scrollIntoView(targetPos, { y: 'start', yMargin: 24 })
      });
      view.focus();
    }
  }

  function calculateStats(state: EditorState) {
    const doc = state.doc;
    const text = doc.toString();
    const selection = state.selection.main;
    const line = doc.lineAt(selection.head);
    const col = selection.head - line.from + 1;
    const words = (text.match(/\b\w+\b/g) || []).length;
    const characters = text.length;
    const selectedChars = Math.abs(selection.to - selection.from);

    onStatsChange({
      line: line.number,
      col,
      words,
      characters,
      selectedChars
    });
  }

  function initEditor() {
    if (!editorElement) return;

    if (view) {
      view.destroy();
    }

    const state = EditorState.create({
      doc: content,
      extensions: [
        lineNumbers(),
        highlightActiveLineGutter(),
        highlightActiveLine(),
        EditorView.lineWrapping,
        history(),
        markdown(),
        zedThemeExtension,
        createWikilinkAutocomplete(),
        Prec.highest(
          keymap.of([
            {
              key: 'Mod-s',
              run: () => {
                onSave();
                return true;
              }
            }
          ])
        ),
        keymap.of([
          ...defaultKeymap,
          ...historyKeymap,
          indentWithTab
        ]),
        EditorView.updateListener.of((update) => {
          if (update.docChanged) {
            const newDoc = update.state.doc.toString();
            isInternalUpdate = true;
            onChange(newDoc);
            queueMicrotask(() => {
              isInternalUpdate = false;
            });
          }
          if (update.docChanged || update.selectionSet) {
            calculateStats(update.state);
          }
        })
      ]
    });

    view = new EditorView({
      state,
      parent: editorElement
    });

    calculateStats(state);
    currentPath = path;
  }

  $effect(() => {
    if (path !== currentPath && editorElement) {
      initEditor();
    } else if (view && !isInternalUpdate) {
      const currentDoc = view.state.doc.toString();
      if (content !== currentDoc) {
        view.dispatch({
          changes: { from: 0, to: currentDoc.length, insert: content }
        });
        calculateStats(view.state);
      }
    }
  });

  onMount(() => {
    initEditor();
  });

  onDestroy(() => {
    if (view) {
      view.destroy();
      view = null;
    }
  });
</script>

<div class="h-full w-full bg-zed-bg-base relative overflow-hidden flex flex-col">
  <div bind:this={editorElement} class="h-full w-full overflow-auto"></div>
</div>
