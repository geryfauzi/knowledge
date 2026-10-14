<script lang="ts">
  import Dialog from './Dialog.svelte';
  import Button from '../button/Button.svelte';
  import Input from '../input/Input.svelte';

  interface Props {
    open: boolean;
    title: string;
    description?: string;
    placeholder?: string;
    initialValue?: string;
    confirmText?: string;
    cancelText?: string;
    isAlertOnly?: boolean;
    danger?: boolean;
    onConfirm: (val: string) => void;
    onCancel?: () => void;
    onClose: () => void;
  }

  let {
    open,
    title,
    description = '',
    placeholder = '',
    initialValue = '',
    confirmText = 'Confirm',
    cancelText = 'Cancel',
    isAlertOnly = false,
    danger = false,
    onConfirm,
    onCancel,
    onClose
  }: Props = $props();

  let inputValue = $state('');

  $effect(() => {
    if (open) {
      inputValue = initialValue;
    }
  });

  function handleSubmit() {
    onConfirm(inputValue);
    onClose();
  }

  function handleCancel() {
    if (onCancel) {
      onCancel();
    }
    onClose();
  }

  function handleKeyDown(e: KeyboardEvent) {
    if (e.key === 'Enter') {
      handleSubmit();
    }
  }
</script>

<Dialog
  {open}
  {title}
  {description}
  {onClose}
>
  {#snippet children()}
    {#if !isAlertOnly}
      <div class="space-y-2">
        <Input
          bind:value={inputValue}
          {placeholder}
          onkeydown={handleKeyDown}
          class="text-xs"
        />
      </div>
    {/if}
  {/snippet}

  {#snippet footer()}
    {#if cancelText}
      <Button variant="ghost" size="sm" onclick={handleCancel}>
        {cancelText}
      </Button>
    {/if}
    <Button
      variant="default"
      size="sm"
      onclick={handleSubmit}
      class={danger ? 'bg-red-600 text-white font-semibold hover:bg-red-700' : 'bg-zed-accent text-zinc-950 font-semibold hover:bg-zed-accent-hover'}
    >
      {confirmText}
    </Button>
  {/snippet}
</Dialog>
