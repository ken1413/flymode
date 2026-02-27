import { useState, useEffect, useCallback } from 'preact/hooks';
import { invoke } from '@tauri-apps/api/core';
import type { SyncState, P2PConfig, PeerDevice } from '../App';
import { toast } from './Toast';

export function SyncTab() {
  const [syncState, setSyncState] = useState<SyncState | null>(null);
  const [p2pConfig, setP2pConfig] = useState<P2PConfig | null>(null);
  const [syncing, setSyncing] = useState(false);
  const [exportData, setExportData] = useState<string | null>(null);

  const loadState = useCallback(async () => {
    try {
      const state = await invoke<SyncState>('get_sync_state');
      setSyncState(state);
    } catch (e) {
      toast.error('Failed to load sync state');
    }
  }, []);

  const loadP2PConfig = useCallback(async () => {
    try {
      const cfg = await invoke<P2PConfig>('get_p2p_config');
      setP2pConfig(cfg);
    } catch (e) {
      toast.error('Failed to load P2P config');
    }
  }, []);

  useEffect(() => {
    loadState();
    loadP2PConfig();
    const interval = setInterval(loadState, 2000);
    return () => clearInterval(interval);
  }, [loadState, loadP2PConfig]);

  const syncAll = async () => {
    setSyncing(true);
    try {
      await invoke('sync_all_peers');
      await loadState();
    } catch (e) {
      toast.error('Sync failed: ' + e);
    } finally {
      setSyncing(false);
    }
  };

  const syncWithPeer = async (peer: PeerDevice) => {
    setSyncing(true);
    try {
      await invoke('sync_with_peer', { peer });
      await loadState();
    } catch (e) {
      toast.error('Sync failed: ' + e);
    } finally {
      setSyncing(false);
    }
  };

  const toggleAutoSync = async () => {
    if (!p2pConfig) return;
    const updated = { ...p2pConfig, sync_enabled: !p2pConfig.sync_enabled };
    await invoke('save_p2p_config', { config: updated });
    setP2pConfig(updated);
  };

  const updateSyncInterval = async (seconds: number) => {
    if (!p2pConfig) return;
    const updated = { ...p2pConfig, sync_interval_seconds: seconds };
    try {
      await invoke('save_p2p_config', { config: updated });
      setP2pConfig(updated);
    } catch (e) {
      toast.error('Failed to save sync interval: ' + e);
    }
  };

  const handleExport = async () => {
    try {
      const data = await invoke<string>('export_notes');
      setExportData(data);
    } catch (e) {
      toast.error('Export failed: ' + e);
    }
  };

  const handleImport = async (json: string) => {
    try {
      const count = await invoke<number>('import_notes', { json });
      toast.success(`Imported ${count} notes`);
    } catch (e) {
      toast.error('Import failed: ' + e);
    }
  };

  const getStatusColor = (status: string): string => {
    switch (status) {
      case 'Success': return 'var(--success)';
      case 'Error': return 'var(--danger)';
      case 'Syncing': return 'var(--primary)';
      default: return 'var(--text-muted)';
    }
  };

  const formatDuration = (ms: number): string => {
    if (ms < 1000) return `${ms}ms`;
    return `${(ms / 1000).toFixed(1)}s`;
  };

  const trustedPeers = p2pConfig?.peers.filter(p => p.is_trusted) || [];

  return (
    <div>
      <div class="card">
        <div class="card-header">
          <span class="card-title">Sync Status</span>
          <button
            class="btn btn-primary"
            onClick={syncAll}
            disabled={syncing || syncState?.status === 'Syncing' || trustedPeers.length === 0}
          >
            {syncing || syncState?.status === 'Syncing' ? 'Syncing...' : 'Sync Now'}
          </button>
        </div>

        <div class="sync-status-grid">
          <div class="sync-status-item">
            <span class="sync-label">Status</span>
            <span class="sync-value" style={{ color: getStatusColor(syncState?.status || 'Idle') }}>
              {syncState?.status || 'Idle'}
            </span>
          </div>
          <div class="sync-status-item">
            <span class="sync-label">Last Sync</span>
            <span class="sync-value">
              {syncState?.last_sync
                ? new Date(syncState.last_sync).toLocaleString()
                : 'Never'}
            </span>
          </div>
          <div class="sync-status-item">
            <span class="sync-label">Trusted Devices</span>
            <span class="sync-value">{trustedPeers.length}</span>
          </div>
          <div class="sync-status-item">
            <span class="sync-label">Auto Sync</span>
            <div
              class={`toggle ${p2pConfig?.sync_enabled ? 'on' : ''}`}
              onClick={toggleAutoSync}
            />
          </div>
        </div>
      </div>

      <div class="card">
        <div class="card-header">
          <span class="card-title">Sync Settings</span>
        </div>

        <div class="form-group">
          <label>Sync Interval</label>
          <select
            class="form-control"
            value={p2pConfig?.sync_interval_seconds || 300}
            onChange={e => updateSyncInterval(parseInt(e.currentTarget.value))}
            disabled={!p2pConfig?.sync_enabled}
          >
            <option value={60}>Every minute</option>
            <option value={300}>Every 5 minutes</option>
            <option value={600}>Every 10 minutes</option>
            <option value={1800}>Every 30 minutes</option>
            <option value={3600}>Every hour</option>
          </select>
        </div>

        <p style={{ fontSize: '12px', color: 'var(--text-muted)' }}>
          Auto-sync only works with trusted devices. Mark devices as trusted in the Devices tab.
        </p>
      </div>

      <div class="card">
        <div class="card-header">
          <span class="card-title">Manual Sync</span>
        </div>

        {trustedPeers.length === 0 ? (
          <div class="empty-state">
            <p>No trusted devices</p>
            <p style={{ fontSize: '12px', color: 'var(--text-muted)', marginTop: '8px' }}>
              Add devices and mark them as trusted to enable sync
            </p>
          </div>
        ) : (
          trustedPeers.map(peer => (
            <div class="sync-peer-item" key={peer.id}>
              <div class="peer-info">
                <div class="peer-name">🔒 {peer.name}</div>
                <div class="peer-details">{peer.ip_address}</div>
              </div>
              <button
                class="btn btn-primary btn-sm"
                onClick={() => syncWithPeer(peer)}
                disabled={syncing}
              >
                Sync
              </button>
            </div>
          ))
        )}
      </div>

      <div class="card">
        <div class="card-header">
          <span class="card-title">Backup & Restore</span>
        </div>

        <div style={{ display: 'flex', gap: '12px' }}>
          <button class="btn btn-primary" onClick={handleExport}>
            📤 Export Notes
          </button>
          <label class="btn btn-icon" style={{ cursor: 'pointer' }}>
            📥 Import Notes
            <input
              type="file"
              accept=".json"
              style={{ display: 'none' }}
              onChange={e => {
                const file = e.currentTarget.files?.[0];
                if (file) {
                  const reader = new FileReader();
                  reader.onload = (ev) => {
                    const content = ev.target?.result as string;
                    handleImport(content);
                  };
                  reader.readAsText(file);
                }
              }}
            />
          </label>
        </div>

        {exportData && (
          <div style={{ marginTop: '16px' }}>
            <textarea
              class="form-control"
              value={exportData}
              readOnly
              rows={10}
              onClick={e => e.currentTarget.select()}
            />
            <button
              class="btn btn-icon btn-sm"
              style={{ marginTop: '8px' }}
              onClick={() => {
                navigator.clipboard.writeText(exportData);
                toast.success('Copied to clipboard');
              }}
            >
              Copy to Clipboard
            </button>
          </div>
        )}
      </div>

      {syncState && syncState.results.length > 0 && (
        <div class="card">
          <div class="card-header">
            <span class="card-title">Recent Sync History</span>
          </div>

          {syncState.results.slice(-10).reverse().map((result, i) => (
            <div class="sync-result-item" key={i}>
              <div class="sync-result-info">
                <span class="sync-peer-name">{result.peer_name}</span>
                <span class="sync-time">{new Date(result.timestamp).toLocaleString()}</span>
              </div>
              <div class="sync-result-details">
                <span style={{ color: getStatusColor(result.status) }}>
                  {result.status}
                </span>
                {' • '}
                <span>{result.notes_synced} notes</span>
                {' • '}
                <span>{formatDuration(result.duration_ms)}</span>
                {result.error_message && (
                  <div class="sync-error">{result.error_message}</div>
                )}
              </div>
            </div>
          ))}
        </div>
      )}
    </div>
  );
}
