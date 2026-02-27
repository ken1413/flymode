import { useState, useEffect, useCallback } from 'preact/hooks';
import { invoke } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event';
import type { PeerDevice, P2PConfig, DeviceStatus, ConnectionType, PairRequest } from '../App';
import { toast } from './Toast';

interface PeerFormData {
  name: string;
  hostname: string;
  ip_address: string;
  port: number;
  ssh_user: string;
  ssh_key_path: string;
  ssh_password: string;
  is_trusted: boolean;
}

export function P2PTab() {
  const [config, setConfig] = useState<P2PConfig | null>(null);
  const [showModal, setShowModal] = useState(false);
  const [editingPeer, setEditingPeer] = useState<PeerDevice | null>(null);
  const [discoveredPeers, setDiscoveredPeers] = useState<PeerDevice[]>([]);
  const [discovering, setDiscovering] = useState(false);
  const [peerStatuses, setPeerStatuses] = useState<Map<string, DeviceStatus>>(new Map());
  const [pairRequests, setPairRequests] = useState<PairRequest[]>([]);
  const [pairingIp, setPairingIp] = useState<string | null>(null);
  const [form, setForm] = useState<PeerFormData>({
    name: '',
    hostname: '',
    ip_address: '',
    port: 22,
    ssh_user: '',
    ssh_key_path: '',
    ssh_password: '',
    is_trusted: false,
  });

  const loadConfig = useCallback(async () => {
    try {
      const cfg = await invoke<P2PConfig>('get_p2p_config');
      setConfig(cfg);
    } catch (e) {
      toast.error('Failed to load P2P config');
    }
  }, []);

  const checkStatuses = useCallback(async () => {
    try {
      const statuses = await invoke<[string, DeviceStatus][]>('check_all_peers');
      const map = new Map<string, DeviceStatus>();
      statuses.forEach(([id, status]) => map.set(id, status));
      setPeerStatuses(map);
    } catch (e) {
      toast.error('Failed to check peer statuses');
    }
  }, []);

  const loadPairRequests = useCallback(async () => {
    try {
      const requests = await invoke<PairRequest[]>('get_pending_pair_requests');
      setPairRequests(requests);
    } catch {
      // Silent — pair requests polling failure is expected during startup
    }
  }, []);

  useEffect(() => {
    loadConfig();
    checkStatuses();
    loadPairRequests();
    const statusInterval = setInterval(checkStatuses, 30000);
    const pairInterval = setInterval(loadPairRequests, 3000);

    // Listen for real-time pair request events
    const unlisten = listen<PairRequest>('pair-request-received', () => {
      loadPairRequests();
    });

    return () => {
      clearInterval(statusInterval);
      clearInterval(pairInterval);
      unlisten.then(fn => fn());
    };
  }, [loadConfig, checkStatuses, loadPairRequests]);

  const discoverPeers = async () => {
    setDiscovering(true);
    try {
      const peers = await invoke<PeerDevice[]>('discover_tailscale');
      setDiscoveredPeers(peers);
      if (peers.length === 0) {
        toast.info('No new Tailscale peers found');
      } else {
        toast.success(`Found ${peers.length} Tailscale peer(s)`);
      }
    } catch (e) {
      toast.error('Failed to discover peers: ' + e);
    } finally {
      setDiscovering(false);
    }
  };

  const pairWithPeer = async (peer: PeerDevice) => {
    setPairingIp(peer.ip_address);
    try {
      const accepted = await invoke<boolean>('pair_with_peer', {
        ip: peer.ip_address,
        port: config?.listen_port || 4827,
      });
      if (accepted) {
        toast.success(`Paired with ${peer.name}!`);
        await loadConfig();
        // Remove from discovered list
        setDiscoveredPeers(prev => prev.filter(p => p.ip_address !== peer.ip_address));
      } else {
        toast.info(`${peer.name} declined the pair request`);
      }
    } catch (e) {
      toast.error('Pair failed: ' + e);
    } finally {
      setPairingIp(null);
    }
  };

  const acceptPairRequest = async (requestId: string) => {
    try {
      await invoke('accept_pair_request', { requestId });
      toast.success('Pair request accepted!');
      await loadConfig();
      await loadPairRequests();
    } catch (e) {
      toast.error('Failed to accept: ' + e);
    }
  };

  const rejectPairRequest = async (requestId: string) => {
    try {
      await invoke('reject_pair_request', { requestId });
      toast.info('Pair request rejected');
      await loadPairRequests();
    } catch (e) {
      toast.error('Failed to reject: ' + e);
    }
  };

  const openAddModal = () => {
    setEditingPeer(null);
    setForm({
      name: '',
      hostname: '',
      ip_address: '',
      port: 22,
      ssh_user: '',
      ssh_key_path: '',
      ssh_password: '',
      is_trusted: false,
    });
    setShowModal(true);
  };

  const openEditModal = (peer: PeerDevice) => {
    setEditingPeer(peer);
    setForm({
      name: peer.name,
      hostname: peer.hostname,
      ip_address: peer.ip_address,
      port: peer.port,
      ssh_user: peer.ssh_user,
      ssh_key_path: peer.ssh_key_path || '',
      ssh_password: peer.ssh_password || '',
      is_trusted: peer.is_trusted,
    });
    setShowModal(true);
  };

  const handleSubmit = async () => {
    if (!config) return;

    if (!form.name.trim()) {
      toast.error('Device name is required');
      return;
    }
    if (!form.ip_address.trim()) {
      toast.error('IP address is required');
      return;
    }
    if (!/^(\d{1,3}\.){3}\d{1,3}$/.test(form.ip_address) &&
        !/^[a-fA-F0-9:]+$/.test(form.ip_address)) {
      toast.error('Invalid IP address format');
      return;
    }
    if (form.port < 1 || form.port > 65535) {
      toast.error('Port must be between 1 and 65535');
      return;
    }
    if (!form.ssh_user.trim()) {
      toast.error('SSH username is required');
      return;
    }
    if (!form.ssh_key_path && !form.ssh_password) {
      toast.error('SSH key path or password is required');
      return;
    }

    const peer: PeerDevice = {
      id: editingPeer?.id || '',
      name: form.name,
      hostname: form.hostname,
      ip_address: form.ip_address,
      port: form.port,
      connection_type: 'LanDirect',
      status: 'Unknown',
      last_seen: null,
      ssh_user: form.ssh_user,
      ssh_key_path: form.ssh_key_path || null,
      ssh_password: form.ssh_password || null,
      is_trusted: form.is_trusted,
      tailscale_hostname: null,
      flymode_version: null,
    };

    try {
      if (editingPeer) {
        await invoke('update_peer', { peer });
      } else {
        await invoke('add_peer', { peer });
      }
      await loadConfig();
      setShowModal(false);
    } catch (e) {
      toast.error('Failed to save peer: ' + e);
    }
  };

  const handleDelete = async (peerId: string) => {
    if (!confirm('Remove this device?')) return;
    try {
      await invoke('remove_peer', { peerId });
      await loadConfig();
    } catch (e) {
      toast.error('Failed to remove peer');
    }
  };

  const toggleTrust = async (peer: PeerDevice) => {
    const updated = { ...peer, is_trusted: !peer.is_trusted };
    try {
      await invoke('update_peer', { peer: updated });
      await loadConfig();
    } catch (e) {
      toast.error('Failed to toggle trust');
    }
  };

  const getConnectionIcon = (type: ConnectionType): string => {
    switch (type) {
      case 'Tailscale': return '🦎';
      case 'LanDirect': return '🏠';
      case 'WanDirect': return '🌐';
    }
  };

  const getStatusColor = (status: DeviceStatus): string => {
    switch (status) {
      case 'Online': return 'var(--success)';
      case 'Offline': return 'var(--danger)';
      case 'Unknown': return 'var(--text-muted)';
    }
  };

  const formatTime = (isoString: string): string => {
    const date = new Date(isoString);
    return date.toLocaleTimeString();
  };

  if (!config) {
    return <div class="empty-state">Loading...</div>;
  }

  return (
    <div>
      {/* Incoming Pair Requests */}
      {pairRequests.length > 0 && (
        <div class="card" style={{ borderColor: 'var(--primary)', borderWidth: '2px' }}>
          <div class="card-header">
            <span class="card-title">Incoming Pair Requests ({pairRequests.length})</span>
          </div>
          {pairRequests.map(req => (
            <div class="peer-item" key={req.id}>
              <div class="peer-status" style={{ backgroundColor: 'var(--primary)' }} />
              <div class="peer-info">
                <div class="peer-name">{req.from.device_name}</div>
                <div class="peer-details">
                  {req.from.hostname} • {req.from.ip_address}
                  {req.from.flymode_version && ` • v${req.from.flymode_version}`}
                  {' • '}{formatTime(req.received_at)}
                </div>
              </div>
              <div class="peer-actions">
                <button class="btn btn-primary btn-sm" onClick={() => acceptPairRequest(req.id)}>
                  Accept
                </button>
                <button class="btn btn-danger btn-sm" onClick={() => rejectPairRequest(req.id)}>
                  Reject
                </button>
              </div>
            </div>
          ))}
        </div>
      )}

      <div class="card">
        <div class="card-header">
          <span class="card-title">This Device</span>
        </div>
        <div class="device-info">
          <div class="info-row">
            <span class="info-label">Device ID</span>
            <span class="info-value">{config.device_id.slice(0, 8)}...</span>
          </div>
          <div class="info-row">
            <span class="info-label">Device Name</span>
            <span class="info-value">{config.device_name}</span>
          </div>
          <div class="info-row">
            <span class="info-label">Listen Port</span>
            <span class="info-value">{config.listen_port}</span>
          </div>
        </div>
      </div>

      <div class="card">
        <div class="card-header">
          <span class="card-title">Connected Devices ({config.peers.length})</span>
          <div style={{ display: 'flex', gap: '8px' }}>
            <button
              class="btn btn-icon btn-sm"
              onClick={discoverPeers}
              disabled={discovering}
              title="Discover via Tailscale"
            >
              {discovering ? '🔍...' : '🔍'}
            </button>
            <button class="btn btn-primary btn-sm" onClick={openAddModal}>
              + Add Device
            </button>
          </div>
        </div>

        {config.peers.length === 0 ? (
          <div class="empty-state">
            <p style={{ fontSize: '32px', marginBottom: '12px' }}>🔗</p>
            <p>No devices connected yet.</p>
            <p style={{ fontSize: '12px', color: 'var(--text-muted)', marginTop: '8px' }}>
              Add devices manually or discover via Tailscale
            </p>
          </div>
        ) : (
          config.peers.map(peer => (
            <div class="peer-item" key={peer.id}>
              <div class="peer-status" style={{ backgroundColor: getStatusColor(peerStatuses.get(peer.id) || 'Unknown') }} />
              <div class="peer-info">
                <div class="peer-name">
                  {peer.is_trusted && <span title="Trusted">🔒</span>}
                  {peer.name || peer.hostname}
                </div>
                <div class="peer-details">
                  {getConnectionIcon(peer.connection_type)} {peer.ip_address}:{peer.port}
                  {' • '}
                  {peerStatuses.get(peer.id) || 'Unknown'}
                </div>
              </div>
              <div class="peer-actions">
                <button
                  class={`btn btn-sm ${peer.is_trusted ? 'btn-primary' : 'btn-icon'}`}
                  onClick={() => toggleTrust(peer)}
                  title={peer.is_trusted ? 'Remove trust' : 'Mark as trusted'}
                >
                  {peer.is_trusted ? '🔒' : '🔓'}
                </button>
                <button class="btn btn-icon btn-sm" onClick={() => openEditModal(peer)}>Edit</button>
                <button class="btn btn-danger btn-sm" onClick={() => handleDelete(peer.id)}>Remove</button>
              </div>
            </div>
          ))
        )}
      </div>

      {discoveredPeers.length > 0 && (
        <div class="card">
          <div class="card-header">
            <span class="card-title">Discovered via Tailscale ({discoveredPeers.length})</span>
            <button class="btn btn-icon btn-sm" onClick={() => setDiscoveredPeers([])}>×</button>
          </div>
          {discoveredPeers.map(peer => (
            <div class="peer-item discovered" key={peer.id}>
              <div class="peer-status" style={{ backgroundColor: 'var(--success)' }} />
              <div class="peer-info">
                <div class="peer-name">{peer.name}</div>
                <div class="peer-details">
                  🦎 {peer.ip_address}
                </div>
              </div>
              <button
                class="btn btn-primary btn-sm"
                onClick={() => pairWithPeer(peer)}
                disabled={pairingIp === peer.ip_address}
              >
                {pairingIp === peer.ip_address ? 'Pairing...' : 'Pair'}
              </button>
            </div>
          ))}
        </div>
      )}

      {showModal && (
        <div class="modal-overlay" onClick={() => setShowModal(false)}>
          <div class="modal" onClick={e => e.stopPropagation()}>
            <div class="modal-header">
              <span class="modal-title">{editingPeer ? 'Edit Device' : 'Add Device'}</span>
              <button class="modal-close" onClick={() => setShowModal(false)}>×</button>
            </div>

            <div class="form-row">
              <div class="form-group">
                <label>Device Name</label>
                <input
                  type="text"
                  class="form-control"
                  value={form.name}
                  onInput={e => setForm({ ...form, name: e.currentTarget.value })}
                  placeholder="My Laptop"
                />
              </div>
              <div class="form-group">
                <label>Hostname</label>
                <input
                  type="text"
                  class="form-control"
                  value={form.hostname}
                  onInput={e => setForm({ ...form, hostname: e.currentTarget.value })}
                  placeholder="laptop.local"
                />
              </div>
            </div>

            <div class="form-row">
              <div class="form-group">
                <label>IP Address</label>
                <input
                  type="text"
                  class="form-control"
                  value={form.ip_address}
                  onInput={e => setForm({ ...form, ip_address: e.currentTarget.value })}
                  placeholder="192.168.1.100"
                />
              </div>
              <div class="form-group">
                <label>SSH Port</label>
                <input
                  type="number"
                  class="form-control"
                  value={form.port}
                  onInput={e => setForm({ ...form, port: parseInt(e.currentTarget.value) || 22 })}
                />
              </div>
            </div>

            <div class="form-group">
              <label>SSH Username</label>
              <input
                type="text"
                class="form-control"
                value={form.ssh_user}
                onInput={e => setForm({ ...form, ssh_user: e.currentTarget.value })}
                placeholder="username"
              />
            </div>

            <div class="form-row">
              <div class="form-group">
                <label>SSH Key Path (optional)</label>
                <input
                  type="text"
                  class="form-control"
                  value={form.ssh_key_path}
                  onInput={e => setForm({ ...form, ssh_key_path: e.currentTarget.value })}
                  placeholder="~/.ssh/id_rsa"
                />
              </div>
              <div class="form-group">
                <label>SSH Password (optional)</label>
                <input
                  type="password"
                  class="form-control"
                  value={form.ssh_password}
                  onInput={e => setForm({ ...form, ssh_password: e.currentTarget.value })}
                  placeholder="••••••••"
                />
              </div>
            </div>

            <div class="form-group">
              <label>
                <input
                  type="checkbox"
                  checked={form.is_trusted}
                  onChange={e => setForm({ ...form, is_trusted: e.currentTarget.checked })}
                />
                {' '}Mark as trusted (allows auto-sync)
              </label>
            </div>

            <div class="modal-actions">
              <button class="btn btn-icon" onClick={() => setShowModal(false)}>Cancel</button>
              <button class="btn btn-primary" onClick={handleSubmit}>
                {editingPeer ? 'Save Changes' : 'Add Device'}
              </button>
            </div>
          </div>
        </div>
      )}
    </div>
  );
}
