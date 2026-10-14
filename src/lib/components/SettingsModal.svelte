<script lang="ts">
  import Dialog from './ui/dialog/Dialog.svelte';
  import Button from './ui/button/Button.svelte';
  import { driveSyncService } from '../services/driveSync.svelte';
  import {
    Cloud,
    CheckCircle2,
    RefreshCw,
    LogOut,
    AlertCircle,
    Check
  } from 'lucide-svelte';

  interface Props {
    open: boolean;
    vaultPath: string;
    onClose: () => void;
    onNotify?: (msg: string) => void;
    onSyncComplete?: () => Promise<void> | void;
  }

  let { open, vaultPath, onClose, onNotify, onSyncComplete }: Props = $props();

  let folderName = $state('Knowledge-Vault');
  let isConnecting = $state(false);
  let localError = $state<string | null>(null);

  async function handleConnect() {
    isConnecting = true;
    localError = null;

    try {
      await driveSyncService.startOAuth();
      onNotify?.(`Connected to Google Drive as ${driveSyncService.accountEmail || 'User'}`);

      // Automatically sync immediately after login
      if (vaultPath) {
        onNotify?.('Starting automatic sync with Google Drive...');
        const summary = await driveSyncService.syncNow(vaultPath, folderName);
        if (summary) {
          onNotify?.(
            `Initial sync complete: ↑${summary.uploaded} uploaded, ↓${summary.downloaded} downloaded`
          );
        }
        await onSyncComplete?.();
      }
    } catch (err: any) {
      localError = err?.toString() || 'Failed to connect Google Drive';
    } finally {
      isConnecting = false;
    }
  }

  async function handleDisconnect() {
    await driveSyncService.logout();
    onNotify?.('Disconnected from Google Drive');
  }

  async function handleManualSync() {
    if (!vaultPath) return;
    try {
      localError = null;
      const summary = await driveSyncService.syncNow(vaultPath, folderName);
      if (summary) {
        onNotify?.(
          `Sync complete: ↑${summary.uploaded} uploaded, ↓${summary.downloaded} downloaded`
        );
        await onSyncComplete?.();
      }
    } catch (err: any) {
      localError = err?.toString() || 'Sync failed';
    }
  }
</script>

<Dialog {open} title="Google Drive Synchronization" description="Connect your Google account for automatic two-way cloud sync" {onClose}>
  {#snippet children()}
    <div class="space-y-4 py-1 text-xs">
      {#if driveSyncService.isAuthenticated}
        <!-- Connected Account Card -->
        <div class="p-4 rounded-xl bg-zinc-900/90 border border-zinc-800 space-y-4">
          <div class="flex items-center justify-between">
            <div class="flex items-center space-x-3">
              <div class="w-10 h-10 rounded-xl bg-emerald-500/10 border border-emerald-500/20 flex items-center justify-center text-emerald-400">
                <Cloud size={20} />
              </div>
              <div>
                <div class="flex items-center space-x-2">
                  <h4 class="font-semibold text-zinc-100 text-sm">
                    {driveSyncService.accountEmail || 'Google Account'}
                  </h4>
                  <span class="inline-flex items-center space-x-1 px-2 py-0.5 rounded-full bg-emerald-500/15 text-emerald-400 text-[10.5px] font-medium border border-emerald-500/30">
                    <CheckCircle2 size={11} />
                    <span>Connected</span>
                  </span>
                </div>
                <p class="text-[11px] text-zinc-500 mt-0.5">
                  Remote Vault: <span class="font-mono text-zinc-400">/{folderName}</span>
                </p>
              </div>
            </div>
          </div>

          {#if localError || driveSyncService.syncError}
            <div class="flex items-start space-x-2 p-2.5 rounded bg-red-500/10 border border-red-500/20 text-red-300 text-[11px]">
              <AlertCircle size={14} class="shrink-0 mt-0.5 text-red-400" />
              <span class="leading-relaxed">{localError || driveSyncService.syncError}</span>
            </div>
          {/if}

          <!-- Sync Status and Quick Actions -->
          <div class="pt-3 border-t border-zinc-800/80 flex items-center justify-between">
            <div class="text-[11px] text-zinc-400 flex items-center space-x-1.5">
              {#if driveSyncService.isSyncing}
                <RefreshCw size={12} class="animate-spin text-amber-400" />
                <span class="text-amber-400 font-medium">Syncing with Google Drive...</span>
              {:else if driveSyncService.lastSyncTime}
                <Check size={12} class="text-emerald-400" />
                <span>Last synced: <span class="text-zinc-200 font-mono">{driveSyncService.lastSyncTime}</span></span>
              {:else}
                <span>Ready to sync</span>
              {/if}
            </div>

            <div class="flex items-center space-x-2">
              <Button
                variant="outline"
                size="sm"
                onclick={handleManualSync}
                disabled={driveSyncService.isSyncing}
                class="h-8 text-xs border-zinc-700 bg-zinc-800/80 hover:bg-zinc-700 hover:text-zinc-100"
              >
                <RefreshCw size={12} class={driveSyncService.isSyncing ? 'animate-spin mr-1.5 text-zed-accent' : 'mr-1.5 text-zinc-400'} />
                <span>{driveSyncService.isSyncing ? 'Syncing...' : 'Sync Now'}</span>
              </Button>

              <Button
                variant="ghost"
                size="sm"
                onclick={handleDisconnect}
                class="h-8 text-xs text-red-400 hover:text-red-300 hover:bg-red-500/10"
              >
                <LogOut size={12} class="mr-1.5" />
                <span>Sign Out</span>
              </Button>
            </div>
          </div>
        </div>
      {:else}
        <!-- Login Card -->
        <div class="p-5 rounded-xl bg-zinc-900/90 border border-zinc-800 flex flex-col items-center text-center space-y-4">
          <div class="w-12 h-12 rounded-2xl bg-zinc-800/80 border border-zinc-700/60 flex items-center justify-center text-zinc-300 shadow-inner">
            <Cloud size={24} class="text-zed-accent" />
          </div>

          <div class="space-y-1 max-w-sm">
            <h3 class="text-sm font-semibold text-zinc-100">Sign in to Google Drive</h3>
            <p class="text-xs text-zinc-400 leading-relaxed">
              Connect your Google account to automatically synchronize your markdown vault notes to Google Drive.
            </p>
          </div>

          {#if localError || driveSyncService.syncError}
            <div class="w-full flex items-start space-x-2 p-2.5 rounded bg-red-500/10 border border-red-500/20 text-red-300 text-[11px] text-left">
              <AlertCircle size={14} class="shrink-0 mt-0.5 text-red-400" />
              <span class="leading-relaxed">{localError || driveSyncService.syncError}</span>
            </div>
          {/if}

          <!-- Google Sign In Button -->
          <button
            type="button"
            onclick={handleConnect}
            disabled={isConnecting}
            class="w-full max-w-xs h-10 px-4 bg-white hover:bg-zinc-100 active:bg-zinc-200 text-zinc-900 font-semibold text-xs rounded-lg shadow flex items-center justify-center space-x-2.5 transition-all cursor-pointer disabled:opacity-60 disabled:cursor-not-allowed"
          >
            {#if isConnecting}
              <RefreshCw size={14} class="animate-spin text-zinc-700" />
              <span>Connecting in browser...</span>
            {:else}
              <!-- Official Google "G" Logo -->
              <svg class="w-4 h-4 shrink-0" viewBox="0 0 24 24">
                <path fill="#4285F4" d="M22.56 12.25c0-.78-.07-1.53-.2-2.25H12v4.26h5.92c-.26 1.37-1.04 2.53-2.21 3.31v2.77h3.57c2.08-1.92 3.28-4.74 3.28-8.09z"/>
                <path fill="#34A853" d="M12 23c2.97 0 5.46-.98 7.28-2.66l-3.57-2.77c-.98.66-2.23 1.06-3.71 1.06-2.86 0-5.29-1.93-6.16-4.53H2.18v2.84C3.99 20.53 7.7 23 12 23z"/>
                <path fill="#FBBC05" d="M5.84 14.09c-.22-.66-.35-1.36-.35-2.09s.13-1.43.35-2.09V7.06H2.18C1.43 8.55 1 10.22 1 12s.43 3.45 1.18 4.94l2.85-2.22.81-.63z"/>
                <path fill="#EA4335" d="M12 5.38c1.62 0 3.06.56 4.21 1.64l3.15-3.15C17.45 2.09 14.97 1 12 1 7.7 1 3.99 3.47 2.18 7.06l3.66 2.84c.87-2.6 3.3-4.52 6.16-4.52z"/>
              </svg>
              <span>Sign in with Google</span>
            {/if}
          </button>

          <p class="text-[10.5px] text-zinc-500">
            A secure browser window will open to authorize read/write access to your vault folder.
          </p>
        </div>
      {/if}

    </div>
  {/snippet}

  {#snippet footer()}
    <Button variant="ghost" size="sm" onclick={onClose}>
      Close
    </Button>
  {/snippet}
</Dialog>
