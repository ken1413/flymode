import { useState, useEffect, useCallback, useRef } from 'preact/hooks';
import { invoke } from '@tauri-apps/api/core';
import { RulesTab } from './components/RulesTab';
import { QuickActionsTab } from './components/QuickActionsTab';
import { NotesTab } from './components/NotesTab';
import { P2PTab } from './components/P2PTab';
import { SyncTab } from './components/SyncTab';
import { TransferTab } from './components/TransferTab';
import { SettingsTab } from './components/SettingsTab';
import { LockScreen } from './components/LockScreen';
import { ToastContainer, toast } from './components/Toast';

export interface ScheduleRule {
  id: string;
  name: string;
  enabled: boolean;
  action: ActionType;
  target: TargetType;
  start_time: string;
  end_time: string | null;
  days: number[];
  command: string | null;
}

export type ActionType = 'Enable' | 'Disable' | 'Toggle' | 'RunCommand';
export type TargetType = 'Wifi' | 'Bluetooth' | 'AirplaneMode' | 'CustomCommand';

export interface AppConfig {
  rules: ScheduleRule[];
  check_interval_seconds: number;
  show_notifications: boolean;
  minimize_to_tray: boolean;
  auto_start: boolean;
  require_password: boolean;
}

export interface WirelessStatus {
  wifi_enabled: boolean;
  bluetooth_enabled: boolean;
  airplane_mode: boolean;
}

export interface Note {
  id: string;
  title: string;
  content: string;
  color: NoteColor;
  category: NoteCategory;
  pinned: boolean;
  archived: boolean;
  created_at: string;
  updated_at: string;
  tags: string[];
  position_x: number;
  position_y: number;
  width: number;
  height: number;
  device_id: string;
  sync_hash: string | null;
  deleted: boolean;
}

export type NoteColor = 'Yellow' | 'Pink' | 'Blue' | 'Green' | 'Purple' | 'Orange' | 'White' | 'Gray';
export type NoteCategory = 'General' | 'Work' | 'Personal' | 'Ideas' | 'Tasks' | 'Important' | 'Archive';

export interface PeerDevice {
  id: string;
  name: string;
  hostname: string;
  ip_address: string;
  port: number;
  connection_type: ConnectionType;
  status: DeviceStatus;
  last_seen: string | null;
  ssh_user: string;
  ssh_key_path: string | null;
  ssh_password: string | null;
  is_trusted: boolean;
  tailscale_hostname: string | null;
  flymode_version: string | null;
}

export type ConnectionType = 'Tailscale' | 'LanDirect' | 'WanDirect';
export type DeviceStatus = 'Online' | 'Offline' | 'Unknown';

export interface P2PConfig {
  device_id: string;
  device_name: string;
  listen_port: number;
  peers: PeerDevice[];
  auto_discover_tailscale: boolean;
  sync_enabled: boolean;
  sync_interval_seconds: number;
}

export interface PeerInfo {
  device_id: string;
  device_name: string;
  hostname: string;
  ip_address: string;
  listen_port: number;
  tailscale_hostname: string | null;
  flymode_version: string | null;
}

export interface PairRequest {
  id: string;
  from: PeerInfo;
  received_at: string;
}

export interface SyncState {
  last_sync: string | null;
  status: SyncStatus;
  current_peer: string | null;
  results: SyncResult[];
}

export type SyncStatus = 'Idle' | 'Syncing' | 'Success' | 'Error';

export interface SyncResult {
  peer_id: string;
  peer_name: string;
  status: SyncStatus;
  notes_synced: number;
  files_synced: number;
  timestamp: string;
  error_message: string | null;
  duration_ms: number;
}

export interface TransferProgress {
  transfer_id: string;
  peer_id: string;
  peer_name: string;
  direction: TransferDirection;
  local_path: string;
  remote_path: string;
  file_name: string;
  total_bytes: number;
  transferred_bytes: number;
  status: TransferStatus;
  started_at: string | null;
  completed_at: string | null;
  error_message: string | null;
  speed_bps: number | null;
}

export type TransferDirection = 'Upload' | 'Download';
export type TransferStatus = 'Pending' | 'InProgress' | 'Completed' | 'Failed' | 'Cancelled';

export interface TransferQueue {
  transfers: TransferProgress[];
  max_concurrent: number;
}

export interface RemoteFileInfo {
  name: string;
  path: string;
  is_dir: boolean;
  size: number;
  modified: string | null;
}

export type TabType = 'rules' | 'quick' | 'notes' | 'p2p' | 'sync' | 'transfer' | 'settings';

export function App() {
  const [config, setConfig] = useState<AppConfig | null>(null);
  const [status, setStatus] = useState<WirelessStatus | null>(null);
  const [activeTab, setActiveTab] = useState<TabType>('notes');
  const [loading, setLoading] = useState(true);
  const [locked, setLocked] = useState(false);
  const [syncState, setSyncState] = useState<SyncState | null>(null);
  const hiddenAt = useRef<number>(0);

  const loadConfig = useCallback(async () => {
    try {
      const cfg = await invoke<AppConfig>('get_config');
      setConfig(cfg);
      return cfg;
    } catch (e) {
      toast.error('Failed to load config');
      return null;
    }
  }, []);

  const loadStatus = useCallback(async () => {
    try {
      const s = await invoke<WirelessStatus>('get_status');
      setStatus(s);
    } catch (e) {
      // Silent — status polling failure is expected when backend is busy
    }
  }, []);

  const loadSyncState = useCallback(async () => {
    try {
      const state = await invoke<SyncState>('get_sync_state');
      setSyncState(state);
    } catch (e) {
      // Silent — sync state polling failure is expected
    }
  }, []);

  useEffect(() => {
    loadConfig().then((cfg) => {
      setLoading(false);
      if (cfg?.require_password) {
        setLocked(true);
      }
    });
    loadStatus();
    loadSyncState();
    const interval = setInterval(loadStatus, 5000);
    const syncInterval = setInterval(loadSyncState, 3000);

    // Lock when window is restored from tray (hidden → visible)
    const handleVisibility = () => {
      if (document.visibilityState === 'hidden') {
        hiddenAt.current = Date.now();
      } else if (document.visibilityState === 'visible' && hiddenAt.current > 0) {
        const elapsed = Date.now() - hiddenAt.current;
        hiddenAt.current = 0;
        // Only lock if hidden for >1s (filters out brief minimize/restore)
        if (elapsed > 1000) {
          invoke<AppConfig>('get_config').then(cfg => {
            if (cfg?.require_password) {
              setLocked(true);
            }
          });
        }
      }
    };
    document.addEventListener('visibilitychange', handleVisibility);

    return () => {
      clearInterval(interval);
      clearInterval(syncInterval);
      document.removeEventListener('visibilitychange', handleVisibility);
    };
  }, [loadConfig, loadStatus, loadSyncState]);

  const saveConfig = async (cfg: AppConfig) => {
    await invoke('save_config', { config: cfg });
    setConfig(cfg);
  };

  if (loading) {
    return <div class="container"><div class="empty-state">Loading...</div></div>;
  }

  const tabs: { id: TabType; label: string; icon: string }[] = [
    { id: 'notes', label: 'Notes', icon: '📝' },
    { id: 'p2p', label: 'Devices', icon: '🔗' },
    { id: 'sync', label: 'Sync', icon: '🔄' },
    { id: 'transfer', label: 'Transfer', icon: '📤' },
    { id: 'rules', label: 'Schedule', icon: '⏰' },
    { id: 'quick', label: 'Quick', icon: '⚡' },
    { id: 'settings', label: 'Settings', icon: '⚙️' },
  ];

  if (locked) {
    return (
      <div class="container">
        <ToastContainer />
        <LockScreen onUnlock={() => setLocked(false)} />
      </div>
    );
  }

  return (
    <div class="container">
      <ToastContainer />
      <header>
        <h1>FlyMode</h1>
        <div class="header-status">
          <span style={{ color: 'var(--text-muted)', fontSize: '14px' }}>
            Wireless + Sync
          </span>
          {syncState?.status === 'Syncing' && (
            <span class="sync-indicator">Syncing...</span>
          )}
        </div>
      </header>

      <div class="status-bar">
        <div class="status-item">
          <span class={`status-dot ${status?.wifi_enabled ? 'on' : 'off'}`} />
          <span>WiFi</span>
        </div>
        <div class="status-item">
          <span class={`status-dot ${status?.bluetooth_enabled ? 'on' : 'off'}`} />
          <span>BT</span>
        </div>
        <div class="status-item">
          <span class={`status-dot ${status?.airplane_mode ? 'on' : 'off'}`} />
          <span>Air</span>
        </div>
      </div>

      <div class="tabs">
        {tabs.map(tab => (
          <button
            key={tab.id}
            class={`tab ${activeTab === tab.id ? 'active' : ''}`}
            onClick={() => setActiveTab(tab.id)}
            title={tab.label}
          >
            {tab.icon}
          </button>
        ))}
      </div>

      {activeTab === 'rules' && config && <RulesTab config={config} onSave={saveConfig} />}
      {activeTab === 'quick' && <QuickActionsTab />}
      {activeTab === 'notes' && <NotesTab />}
      {activeTab === 'p2p' && <P2PTab />}
      {activeTab === 'sync' && <SyncTab />}
      {activeTab === 'transfer' && <TransferTab />}
      {activeTab === 'settings' && config && <SettingsTab config={config} onSave={saveConfig} />}
    </div>
  );
}
