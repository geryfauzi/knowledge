<script lang="ts">
  import { onMount, untrack } from 'svelte';
  import EditorToolbar from './editor/EditorToolbar.svelte';
  import WysiwygEditor from './editor/WysiwygEditor.svelte';
  import MarkdownEditor from './editor/MarkdownEditor.svelte';
  import { markdownToHtml, htmlToMarkdown } from '../services/markdownConverter';
  import type { EditorStats } from '../types';

  interface Props {
    content: string;
    path: string;
    isDirty?: boolean;
    onChange: (newMarkdown: string) => void;
    onSave: () => void;
    onStatsChange: (stats: EditorStats) => void;
  }

  let {
    content,
    path,
    isDirty = false,
    onChange,
    onSave,
    onStatsChange
  }: Props = $props();

  let mode = $state<'wysiwyg' | 'markdown'>('wysiwyg');
  let htmlContent = $state('');
  let activeStates = $state<Record<string, boolean>>({});

  let wysiwygRef = $state<any>(null);
  let markdownRef = $state<any>(null);
  let currentLoadedPath = $state('');

  function syncMarkdownToHtml(md: string) {
    htmlContent = markdownToHtml(md);
  }

  function handleToggleMode(newMode: 'wysiwyg' | 'markdown') {
    if (newMode === mode) return;

    if (newMode === 'wysiwyg') {
      // Switching from Pure MD to WYSIWYG: Parse MD to HTML
      syncMarkdownToHtml(content);
    }
    mode = newMode;
  }

  function handleWysiwygChange(newHtml: string) {
    htmlContent = newHtml;
    const newMarkdown = htmlToMarkdown(newHtml);
    onChange(newMarkdown);
  }

  function handleMarkdownChange(newMd: string) {
    onChange(newMd);
  }

  function handleToolbarAction(action: string, param?: any) {
    if (action === 'save') {
      onSave();
      return;
    }

    if (mode === 'wysiwyg' && wysiwygRef) {
      wysiwygRef.executeAction(action, param);
    } else if (mode === 'markdown' && markdownRef) {
      markdownRef.executeAction(action, param);
    }
  }

  $effect(() => {
    if (path !== currentLoadedPath) {
      currentLoadedPath = path;
      const md = untrack(() => content);
      syncMarkdownToHtml(md);
    }
  });

  onMount(() => {
    currentLoadedPath = path;
    syncMarkdownToHtml(content);
  });
</script>

<div class="h-full w-full flex flex-col bg-zed-bg-base overflow-hidden">
  <!-- Top Formatting Toolbar (WYSIWYG + Markdown toggle) -->
  <EditorToolbar
    {mode}
    onToggleMode={handleToggleMode}
    onAction={handleToolbarAction}
    {isDirty}
    {activeStates}
  />

  <!-- Active Canvas Mode -->
  <div class="flex-1 overflow-hidden relative">
    {#if mode === 'wysiwyg'}
      <WysiwygEditor
        bind:this={wysiwygRef}
        {htmlContent}
        {path}
        onChange={handleWysiwygChange}
        {onStatsChange}
        onActiveStatesChange={(states) => (activeStates = states)}
      />
    {:else}
      <MarkdownEditor
        bind:this={markdownRef}
        {content}
        {path}
        onChange={handleMarkdownChange}
        {onSave}
        {onStatsChange}
      />
    {/if}
  </div>
</div>
