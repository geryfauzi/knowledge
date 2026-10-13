<script lang="ts">
  import {
    Bold,
    Italic,
    Underline as UnderlineIcon,
    Strikethrough,
    Code,
    Heading1,
    Heading2,
    Heading3,
    List,
    ListOrdered,
    Quote,
    Minus,
    AlignLeft,
    AlignCenter,
    AlignRight,
    FileText,
    Sparkles,
    Save,
    Link2
  } from 'lucide-svelte';
  import Button from '../ui/button/Button.svelte';

  interface Props {
    mode: 'wysiwyg' | 'markdown';
    onToggleMode: (newMode: 'wysiwyg' | 'markdown') => void;
    onAction: (action: string, param?: any) => void;
    isDirty?: boolean;
    activeStates?: Record<string, boolean>;
  }

  let {
    mode,
    onToggleMode,
    onAction,
    isDirty = false,
    activeStates = {}
  }: Props = $props();
</script>

<div class="h-9 bg-zed-bg-surface border-b border-zed-border flex items-center justify-between px-3 select-none text-xs gap-2 overflow-x-auto">
  <!-- Left formatting actions -->
  <div class="flex items-center space-x-0.5">
    <!-- Headings -->
    <Button
      variant="ghost"
      size="icon"
      onclick={() => onAction('heading', 1)}
      class="h-7 w-7 {activeStates.heading1 ? 'bg-zinc-800 text-zed-accent' : 'text-zinc-400 hover:text-zinc-200'}"
      title="Heading 1"
    >
      <Heading1 size={14} />
    </Button>
    <Button
      variant="ghost"
      size="icon"
      onclick={() => onAction('heading', 2)}
      class="h-7 w-7 {activeStates.heading2 ? 'bg-zinc-800 text-zed-accent' : 'text-zinc-400 hover:text-zinc-200'}"
      title="Heading 2"
    >
      <Heading2 size={14} />
    </Button>
    <Button
      variant="ghost"
      size="icon"
      onclick={() => onAction('heading', 3)}
      class="h-7 w-7 {activeStates.heading3 ? 'bg-zinc-800 text-zed-accent' : 'text-zinc-400 hover:text-zinc-200'}"
      title="Heading 3"
    >
      <Heading3 size={14} />
    </Button>

    <div class="h-4 w-px bg-zed-border mx-1"></div>

    <!-- Inline styles -->
    <Button
      variant="ghost"
      size="icon"
      onclick={() => onAction('bold')}
      class="h-7 w-7 {activeStates.bold ? 'bg-zinc-800 text-zed-accent' : 'text-zinc-400 hover:text-zinc-200'}"
      title="Bold (Ctrl+B)"
    >
      <Bold size={13} />
    </Button>
    <Button
      variant="ghost"
      size="icon"
      onclick={() => onAction('italic')}
      class="h-7 w-7 {activeStates.italic ? 'bg-zinc-800 text-zed-accent' : 'text-zinc-400 hover:text-zinc-200'}"
      title="Italic (Ctrl+I)"
    >
      <Italic size={13} />
    </Button>
    <Button
      variant="ghost"
      size="icon"
      onclick={() => onAction('underline')}
      class="h-7 w-7 {activeStates.underline ? 'bg-zinc-800 text-zed-accent' : 'text-zinc-400 hover:text-zinc-200'}"
      title="Underline (Ctrl+U)"
    >
      <UnderlineIcon size={13} />
    </Button>
    <Button
      variant="ghost"
      size="icon"
      onclick={() => onAction('strike')}
      class="h-7 w-7 {activeStates.strike ? 'bg-zinc-800 text-zed-accent' : 'text-zinc-400 hover:text-zinc-200'}"
      title="Strikethrough"
    >
      <Strikethrough size={13} />
    </Button>
    <Button
      variant="ghost"
      size="icon"
      onclick={() => onAction('code')}
      class="h-7 w-7 {activeStates.code ? 'bg-zinc-800 text-zed-accent' : 'text-zinc-400 hover:text-zinc-200'}"
      title="Inline Code"
    >
      <Code size={13} />
    </Button>
    <Button
      variant="ghost"
      size="icon"
      onclick={() => onAction('wikilink')}
      class="h-7 w-7 text-zinc-400 hover:text-zinc-200"
      title="Insert [[Wikilink]]"
    >
      <Link2 size={13} class="text-zed-accent" />
    </Button>

    <div class="h-4 w-px bg-zed-border mx-1"></div>

    <!-- Alignment (WYSIWYG Mode) -->
    <Button
      variant="ghost"
      size="icon"
      onclick={() => onAction('align', 'left')}
      class="h-7 w-7 {activeStates.alignLeft ? 'bg-zinc-800 text-zed-accent' : 'text-zinc-400 hover:text-zinc-200'}"
      title="Align Left"
    >
      <AlignLeft size={13} />
    </Button>
    <Button
      variant="ghost"
      size="icon"
      onclick={() => onAction('align', 'center')}
      class="h-7 w-7 {activeStates.alignCenter ? 'bg-zinc-800 text-zed-accent' : 'text-zinc-400 hover:text-zinc-200'}"
      title="Align Center"
    >
      <AlignCenter size={13} />
    </Button>
    <Button
      variant="ghost"
      size="icon"
      onclick={() => onAction('align', 'right')}
      class="h-7 w-7 {activeStates.alignRight ? 'bg-zinc-800 text-zed-accent' : 'text-zinc-400 hover:text-zinc-200'}"
      title="Align Right"
    >
      <AlignRight size={13} />
    </Button>

    <div class="h-4 w-px bg-zed-border mx-1"></div>

    <!-- Lists & Quotes -->
    <Button
      variant="ghost"
      size="icon"
      onclick={() => onAction('bulletList')}
      class="h-7 w-7 {activeStates.bulletList ? 'bg-zinc-800 text-zed-accent' : 'text-zinc-400 hover:text-zinc-200'}"
      title="Bullet List"
    >
      <List size={13} />
    </Button>
    <Button
      variant="ghost"
      size="icon"
      onclick={() => onAction('orderedList')}
      class="h-7 w-7 {activeStates.orderedList ? 'bg-zinc-800 text-zed-accent' : 'text-zinc-400 hover:text-zinc-200'}"
      title="Numbered List"
    >
      <ListOrdered size={13} />
    </Button>
    <Button
      variant="ghost"
      size="icon"
      onclick={() => onAction('blockquote')}
      class="h-7 w-7 {activeStates.blockquote ? 'bg-zinc-800 text-zed-accent' : 'text-zinc-400 hover:text-zinc-200'}"
      title="Quote"
    >
      <Quote size={13} />
    </Button>
    <Button
      variant="ghost"
      size="icon"
      onclick={() => onAction('horizontalRule')}
      class="h-7 w-7 text-zinc-400 hover:text-zinc-200"
      title="Divider Line"
    >
      <Minus size={13} />
    </Button>
  </div>

  <!-- Right: Save button & Mode Switcher -->
  <div class="flex items-center space-x-2 shrink-0">
    <!-- Save button -->
    <Button
      variant="ghost"
      size="sm"
      onclick={() => onAction('save')}
      class="h-7 px-2 flex items-center space-x-1 {isDirty ? 'text-amber-400 bg-amber-950/20' : 'text-zinc-400'}"
      title="Save Changes (Ctrl+S)"
    >
      <Save size={12} />
      <span class="text-[11px]">{isDirty ? 'Save*' : 'Saved'}</span>
    </Button>

    <div class="h-4 w-px bg-zed-border"></div>

    <!-- Mode Toggle (Visual vs Pure Markdown) -->
    <div class="flex items-center p-0.5 rounded bg-zinc-900 border border-zinc-800 text-[11px]">
      <button
        onclick={() => onToggleMode('wysiwyg')}
        class="flex items-center space-x-1 px-2 py-0.5 rounded transition-colors {mode === 'wysiwyg' ? 'bg-zinc-800 text-zed-accent font-medium shadow-sm' : 'text-zinc-400 hover:text-zinc-200'}"
      >
        <Sparkles size={11} />
        <span>WYSIWYG</span>
      </button>

      <button
        onclick={() => onToggleMode('markdown')}
        class="flex items-center space-x-1 px-2 py-0.5 rounded transition-colors {mode === 'markdown' ? 'bg-zinc-800 text-zed-accent font-medium shadow-sm' : 'text-zinc-400 hover:text-zinc-200'}"
      >
        <FileText size={11} />
        <span>Pure MD</span>
      </button>
    </div>
  </div>
</div>
