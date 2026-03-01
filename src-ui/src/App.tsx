import { useState, useEffect, useCallback, useRef } from 'preact/hooks';
import { invoke } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event';
import { RulesTab } from './components/RulesTab';
import { QuickActionsTab } from './components/QuickActionsTab';
import { NotesTab } from './components/NotesTab';
import { P2PTab } from './components/P2PTab';
import { SyncTab } from './components/SyncTab';
import { TransferTab } from './components/TransferTab';
import { SettingsTab } from './components/SettingsTab';
import { LockScreen } from './components/LockScreen';
import { TerminalModal } from './components/TerminalModal';
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
  pin: string;
  received_at: string;
}

export interface PairResult {
  accepted: boolean;
  pin: string | null;
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

  // OpenClaw state — lifted to App level for header display
  const [openclawPeers, setOpenclawPeers] = useState<Set<string>>(new Set());
  const [openclawLocalPeer, setOpenclawLocalPeer] = useState<PeerDevice | null>(null);
  const [p2pConfig, setP2pConfig] = useState<P2PConfig | null>(null);
  const [showTerminalFromHeader, setShowTerminalFromHeader] = useState(false);

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

  // Load P2P config for OpenClaw detection
  const loadP2pConfig = useCallback(async () => {
    try {
      const cfg = await invoke<P2PConfig>('get_p2p_config');
      setP2pConfig(cfg);
      return cfg;
    } catch {
      return null;
    }
  }, []);

  // Check peer statuses for OpenClaw detection
  const checkPeerStatuses = useCallback(async (peers: PeerDevice[]) => {
    const statuses = new Map<string, DeviceStatus>();
    for (const peer of peers) {
      if (peer.is_trusted) {
        try {
          const s = await invoke<DeviceStatus>('check_peer_status', { peer });
          statuses.set(peer.id, s);
        } catch {
          statuses.set(peer.id, 'Unknown');
        }
      }
    }
    return statuses;
  }, []);

  // OpenClaw detection — runs at App level so header can show node count
  const checkingOpenclawRef = useRef(false);
  const checkOpenclawStatus = useCallback(async () => {
    if (checkingOpenclawRef.current) return;
    checkingOpenclawRef.current = true;
    try {
      const cfg = await loadP2pConfig();
      if (!cfg) return;

      // Check local OpenClaw
      try {
        const localRunning = await invoke<boolean>('check_local_openclaw');
        if (localRunning) {
          const [username, keyPath] = await invoke<[string, string | null]>('get_local_ssh_info');
          setOpenclawLocalPeer({
            id: '__local__',
            name: `${cfg.device_name} (localhost)`,
            hostname: 'localhost',
            ip_address: '127.0.0.1',
            port: 22,
            connection_type: 'LanDirect',
            status: 'Online',
            last_seen: null,
            ssh_user: username,
            ssh_key_path: keyPath,
            ssh_password: null,
            is_trusted: true,
            tailscale_hostname: null,
            flymode_version: null,
          });
        } else {
          setOpenclawLocalPeer(null);
        }
      } catch {
        setOpenclawLocalPeer(null);
      }

      // Check remote peers
      const results = new Set<string>();
      const statuses = await checkPeerStatuses(cfg.peers);
      for (const peer of cfg.peers) {
        if (peer.is_trusted && statuses.get(peer.id) === 'Online') {
          try {
            const running = await invoke<boolean>('check_openclaw_status', { peer });
            if (running) results.add(peer.id);
          } catch {
            // Silently skip
          }
        }
      }
      setOpenclawPeers(results);
    } finally {
      checkingOpenclawRef.current = false;
    }
  }, [loadP2pConfig, checkPeerStatuses]);

  useEffect(() => {
    loadConfig().then((cfg) => {
      setLoading(false);
      if (cfg?.require_password) {
        setLocked(true);
      }
    });
    loadStatus();
    loadSyncState();
    checkOpenclawStatus();
    const interval = setInterval(loadStatus, 5000);
    const syncInterval = setInterval(loadSyncState, 3000);
    const openclawInterval = setInterval(checkOpenclawStatus, 120000);

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

    // Notify user when a pair request arrives from another device
    const unlistenPair = listen<PairRequest>('pair-request-received', (event) => {
      const name = event.payload.from.device_name || 'Unknown device';
      toast.info(
        `"${name}" wants to pair`,
        { label: 'Go to Devices', onClick: () => setActiveTab('p2p') },
      );
    });

    return () => {
      clearInterval(interval);
      clearInterval(syncInterval);
      clearInterval(openclawInterval);
      document.removeEventListener('visibilitychange', handleVisibility);
      unlistenPair.then(fn => fn());
    };
  }, [loadConfig, loadStatus, loadSyncState, checkOpenclawStatus]);

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

  const openclawNodeCount = (openclawLocalPeer ? 1 : 0) + openclawPeers.size;
  const openclawAllPeers = [
    ...(openclawLocalPeer ? [openclawLocalPeer] : []),
    ...(p2pConfig?.peers.filter(p => openclawPeers.has(p.id)) || []),
  ];

  return (
    <div class="container">
      <ToastContainer />
      <header>
        <div class="header-left">
          <h1>FlyMode</h1>
        </div>
        <div class="header-right">
          {openclawNodeCount > 0 && (
            <button
              class="openclaw-btn"
              onClick={() => setShowTerminalFromHeader(true)}
              title={`${openclawNodeCount} OpenClaw node${openclawNodeCount > 1 ? 's' : ''} available`}
            >
              <span class="openclaw-dot" />
              {'>_'} OpenClaw
              <span class="openclaw-count">{openclawNodeCount}</span>
            </button>
          )}
          <div class="wireless-chips">
            <span class={`chip ${status?.wifi_enabled ? 'chip-on' : 'chip-off'}`} title={status?.wifi_enabled ? 'WiFi On' : 'WiFi Off'}>
              {status?.wifi_enabled ? '📶' : '📵'}
            </span>
            <span class={`chip ${status?.bluetooth_enabled ? 'chip-on' : 'chip-off'}`} title={status?.bluetooth_enabled ? 'Bluetooth On' : 'Bluetooth Off'}>
              {status?.bluetooth_enabled ? '🟦' : '⬛'}
            </span>
            {status?.airplane_mode && (
              <span class="chip chip-warn" title="Airplane Mode On">✈️</span>
            )}
          </div>
          {syncState?.status === 'Syncing' && (
            <span class="sync-indicator">Syncing...</span>
          )}
        </div>
      </header>

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
      {activeTab === 'p2p' && (
        <P2PTab
          openclawPeers={openclawPeers}
          openclawLocalPeer={openclawLocalPeer}
          onOpenclawRefresh={checkOpenclawStatus}
        />
      )}
      {activeTab === 'sync' && <SyncTab />}
      {activeTab === 'transfer' && <TransferTab />}
      {activeTab === 'settings' && config && <SettingsTab config={config} onSave={saveConfig} />}

      {showTerminalFromHeader && openclawAllPeers.length > 0 && (
        <TerminalModal
          openclawPeers={openclawAllPeers}
          initialPeer={openclawAllPeers[0]}
          onClose={() => setShowTerminalFromHeader(false)}
        />
      )}
    </div>
  );
}
