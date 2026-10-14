import { isTauri } from './vault';

export interface AuthStatus {
  is_authenticated: boolean;
  email: string | null;
  expires_at: number | null;
}

export interface SyncSummary {
  uploaded: number;
  downloaded: number;
  conflicts: number;
  deleted: number;
  errors: string[];
}

export const DEFAULT_GOOGLE_CLIENT_ID =
  import.meta.env.VITE_GOOGLE_CLIENT_ID || '';
export const DEFAULT_GOOGLE_CLIENT_SECRET =
  import.meta.env.VITE_GOOGLE_CLIENT_SECRET || '';

class DriveSyncService {
  public isAuthenticated = $state<boolean>(false);
  public accountEmail = $state<string | null>(null);
  public isSyncing = $state<boolean>(false);
  public lastSyncTime = $state<string | null>(null);
  public lastSummary = $state<SyncSummary | null>(null);
  public syncError = $state<string | null>(null);

  /**
   * Check whether user is currently authenticated with Google
   */
  async checkAuthStatus(): Promise<boolean> {
    if (!isTauri) return false;

    try {
      const { invoke } = await import('@tauri-apps/api/core');
      const status: AuthStatus = await invoke('google_get_auth_status');
      this.isAuthenticated = status.is_authenticated;
      this.accountEmail = status.email;
      return status.is_authenticated;
    } catch (err: any) {
      console.warn('Failed to check Google auth status:', err);
      return false;
    }
  }

  /**
   * Initiate OAuth2 PKCE login flow via system browser & loopback listener
   */
  async startOAuth(
    clientId: string = DEFAULT_GOOGLE_CLIENT_ID,
    clientSecret: string = DEFAULT_GOOGLE_CLIENT_SECRET
  ): Promise<boolean> {
    if (!isTauri) return false;

    try {
      this.syncError = null;
      const { invoke } = await import('@tauri-apps/api/core');
      const status: AuthStatus = await invoke('google_start_oauth', {
        clientId: clientId.trim(),
        clientSecret: clientSecret.trim()
      });
      this.isAuthenticated = status.is_authenticated;
      this.accountEmail = status.email;
      return status.is_authenticated;
    } catch (err: any) {
      this.syncError = err?.toString() || 'OAuth login failed';
      console.error('Google OAuth error:', err);
      throw err;
    }
  }

  /**
   * Disconnect and remove saved Google tokens
   */
  async logout(): Promise<void> {
    if (!isTauri) return;

    try {
      const { invoke } = await import('@tauri-apps/api/core');
      await invoke('google_logout');
      this.isAuthenticated = false;
      this.accountEmail = null;
      this.lastSyncTime = null;
      this.lastSummary = null;
    } catch (err: any) {
      console.error('Logout error:', err);
    }
  }

  /**
   * Trigger full two-way synchronization
   */
  async syncNow(
    vaultPath: string,
    folderName: string = 'Knowledge-Vault'
  ): Promise<SyncSummary | null> {
    if (!isTauri || !this.isAuthenticated || !vaultPath) return null;

    this.isSyncing = true;
    this.syncError = null;

    try {
      const { invoke } = await import('@tauri-apps/api/core');
      const summary: SyncSummary = await invoke('google_sync_now', {
        vaultPath,
        folderName
      });

      this.lastSummary = summary;
      const now = new Date();
      this.lastSyncTime = now.toLocaleTimeString([], {
        hour: '2-digit',
        minute: '2-digit'
      });

      if (summary.errors.length > 0) {
        this.syncError = summary.errors.join('; ');
      }

      return summary;
    } catch (err: any) {
      this.syncError = err?.toString() || 'Sync failed';
      console.error('Sync error:', err);
      throw err;
    } finally {
      this.isSyncing = false;
    }
  }

  /**
   * Fast background sync when saving a single file
   */
  async syncFileOnSave(vaultPath: string, filePath: string): Promise<void> {
    if (!isTauri || !this.isAuthenticated || !vaultPath || !filePath) return;

    try {
      const { invoke } = await import('@tauri-apps/api/core');
      await invoke('google_sync_file', { vaultPath, filePath });
    } catch (err) {
      console.warn('Background sync file failed:', err);
    }
  }

  /**
   * Fast background trash when deleting a file
   */
  async syncDeleteFile(vaultPath: string, filePath: string): Promise<void> {
    if (!isTauri || !this.isAuthenticated || !vaultPath || !filePath) return;

    try {
      const { invoke } = await import('@tauri-apps/api/core');
      await invoke('google_delete_file', { vaultPath, filePath });
    } catch (err) {
      console.warn('Background delete file sync failed:', err);
    }
  }
}

export const driveSyncService = new DriveSyncService();
