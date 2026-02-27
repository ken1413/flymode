import { useState, useEffect, useCallback } from 'preact/hooks';
import { invoke } from '@tauri-apps/api/core';
import { open } from '@tauri-apps/plugin-dialog';
import type { TransferQueue, TransferProgress, PeerDevice, P2PConfig, TransferStatus, RemoteFileInfo } from '../App';
import { toast } from './Toast';

export function TransferTab() {
  const [queue, setQueue] = useState<TransferQueue | null>(null);
  const [p2pConfig, setP2pConfig] = useState<P2PConfig | null>(null);
  const [showUploadModal, setShowUploadModal] = useState(false);
  const [showBrowseModal, setShowBrowseModal] = useState(false);
  const [selectedPeer, setSelectedPeer] = useState<PeerDevice | null>(null);
  const [remotePath, setRemotePath] = useState('');
  const [localPath, setLocalPath] = useState('');
  const [browsePath, setBrowsePath] = useState('/home');
  const [remoteFiles, setRemoteFiles] = useState<RemoteFileInfo[]>([]);
  const [browsing, setBrowsing] = useState(false);

  const loadQueue = useCallback(async () => {
    try {
      const q = await invoke<TransferQueue>('get_transfer_queue');
      setQueue(q);
    } catch (e) {
      toast.error('Failed to load transfer queue');
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
    loadQueue();
    loadP2PConfig();
    const interval = setInterval(loadQueue, 1000);
    return () => clearInterval(interval);
  }, [loadQueue, loadP2PConfig]);

  const selectFile = async () => {
    try {
      const selected = await open({
        multiple: false,
        title: 'Select file to upload',
      });
      if (selected) {
        setLocalPath(selected as string);
      }
    } catch (e) {
      toast.error('Failed to open file picker: ' + e);
    }
  };

  const openUploadModal = (peer: PeerDevice) => {
    setSelectedPeer(peer);
    setRemotePath('/home/' + (peer.ssh_user || '') + '/');
    setLocalPath('');
    setShowUploadModal(true);
  };

  const startUpload = async () => {
    if (!selectedPeer || !localPath || !remotePath) return;

    // Extract filename from local path (handle both / and \ separators)
    const fileName = localPath.replace(/\\/g, '/').split('/').pop() || 'file';

    // If remote path ends with /, append filename; otherwise use as-is
    const fullRemotePath = remotePath.endsWith('/')
      ? remotePath + fileName
      : remotePath;

    try {
      await invoke('upload_file', {
        peer: selectedPeer,
        localPath,
        remotePath: fullRemotePath,
      });
      toast.success('Upload started: ' + fileName);
      setShowUploadModal(false);
      setLocalPath('');
      setRemotePath('');
      await loadQueue();
    } catch (e) {
      toast.error('Failed to start upload: ' + e);
    }
  };

  const startDownload = async (peer: PeerDevice, filePath: string, fileName: string) => {
    let downloadPath: string | null;
    try {
      downloadPath = await open({
        directory: true,
        title: 'Select download folder',
      });
    } catch (e) {
      toast.error('Failed to open folder picker: ' + e);
      return;
    }

    if (!downloadPath) return;

    const localFilePath = `${downloadPath}/${fileName}`;

    try {
      await invoke('download_file', {
        peer,
        remotePath: filePath,
        localPath: localFilePath,
      });
      toast.success('Download started: ' + fileName);
      await loadQueue();
    } catch (e) {
      toast.error('Failed to start download: ' + e);
    }
  };

  const cancelTransfer = async (transferId: string) => {
    try {
      await invoke('cancel_transfer', { transferId });
      await loadQueue();
    } catch (e) {
      toast.error('Failed to cancel transfer');
    }
  };

  const clearCompleted = async () => {
    try {
      await invoke('clear_completed_transfers');
      await loadQueue();
    } catch (e) {
      toast.error('Failed to clear completed');
    }
  };

  const browseRemote = async (peer: PeerDevice, path: string) => {
    setBrowsing(true);
    try {
      const files = await invoke<RemoteFileInfo[]>('browse_remote_files', {
        peer,
        path,
      });
      setRemoteFiles(files);
      setBrowsePath(path);
    } catch (e) {
      toast.error('Failed to browse: ' + e);
    } finally {
      setBrowsing(false);
    }
  };

  const navigateUp = () => {
    if (!selectedPeer) return;
    // Go to parent directory
    const parts = browsePath.replace(/\/+$/, '').split('/');
    parts.pop();
    const parent = parts.join('/') || '/';
    browseRemote(selectedPeer, parent);
  };

  const openBrowseModal = (peer: PeerDevice) => {
    setSelectedPeer(peer);
    const startPath = '/home/' + (peer.ssh_user || '');
    setBrowsePath(startPath);
    browseRemote(peer, startPath);
    setShowBrowseModal(true);
  };

  const getStatusColor = (status: TransferStatus): string => {
    switch (status) {
      case 'Completed': return 'var(--success)';
      case 'Failed': case 'Cancelled': return 'var(--danger)';
      case 'InProgress': return 'var(--primary)';
      default: return 'var(--text-muted)';
    }
  };

  const formatSize = (bytes: number): string => {
    if (bytes < 1024) return `${bytes} B`;
    if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB`;
    if (bytes < 1024 * 1024 * 1024) return `${(bytes / 1024 / 1024).toFixed(1)} MB`;
    return `${(bytes / 1024 / 1024 / 1024).toFixed(2)} GB`;
  };

  const formatProgress = (t: TransferProgress): string => {
    if (t.total_bytes === 0) return formatSize(t.transferred_bytes);
    const percent = ((t.transferred_bytes / t.total_bytes) * 100).toFixed(0);
    return `${formatSize(t.transferred_bytes)} / ${formatSize(t.total_bytes)} (${percent}%)`;
  };

  const peers = p2pConfig?.peers || [];
  const activeTransfers = queue?.transfers.filter(t => t.status === 'InProgress' || t.status === 'Pending') || [];
  const completedTransfers = queue?.transfers.filter(t => t.status !== 'InProgress' && t.status !== 'Pending') || [];

  return (
    <div>
      <div class="card">
        <div class="card-header">
          <span class="card-title">File Transfer</span>
        </div>

        {peers.length === 0 ? (
          <div class="empty-state">
            <p style={{ fontSize: '32px', marginBottom: '12px' }}>📤</p>
            <p>No devices connected</p>
            <p style={{ fontSize: '12px', color: 'var(--text-muted)', marginTop: '8px' }}>
              Add devices in the Devices tab to transfer files
            </p>
          </div>
        ) : (
          <div class="peer-actions-grid">
            {peers.map(peer => (
              <div class="peer-action-card" key={peer.id}>
                <div class="peer-action-info">
                  <span class="peer-name">{peer.name}</span>
                  <span class="peer-details">{peer.ip_address}</span>
                </div>
                <div class="peer-action-buttons">
                  <button
                    class="btn btn-sm btn-icon"
                    onClick={() => openUploadModal(peer)}
                    title="Upload file"
                  >
                    📤
                  </button>
                  <button
                    class="btn btn-sm btn-icon"
                    onClick={() => openBrowseModal(peer)}
                    title="Browse & download"
                  >
                    📂
                  </button>
                </div>
              </div>
            ))}
          </div>
        )}
      </div>

      {activeTransfers.length > 0 && (
        <div class="card">
          <div class="card-header">
            <span class="card-title">Active Transfers ({activeTransfers.length})</span>
          </div>

          {activeTransfers.map(t => (
            <div class="transfer-item" key={t.transfer_id}>
              <div class="transfer-icon">
                {t.direction === 'Upload' ? '📤' : '📥'}
              </div>
              <div class="transfer-info">
                <div class="transfer-name">{t.file_name}</div>
                <div class="transfer-peer">
                  {t.direction === 'Upload' ? 'to' : 'from'} {t.peer_name}
                </div>
                <div class="transfer-progress-bar">
                  <div
                    class="transfer-progress-fill"
                    style={{
                      width: `${t.total_bytes > 0 ? (t.transferred_bytes / t.total_bytes) * 100 : 0}%`,
                    }}
                  />
                </div>
                <div class="transfer-stats">
                  <span>{formatProgress(t)}</span>
                  {t.speed_bps && <span>• {formatSize(t.speed_bps)}/s</span>}
                </div>
              </div>
              <div class="transfer-status" style={{ color: getStatusColor(t.status) }}>
                {t.status}
              </div>
              {(t.status === 'Pending' || t.status === 'InProgress') && (
                <button
                  class="btn btn-danger btn-sm"
                  onClick={() => cancelTransfer(t.transfer_id)}
                >
                  Cancel
                </button>
              )}
            </div>
          ))}
        </div>
      )}

      {completedTransfers.length > 0 && (
        <div class="card">
          <div class="card-header">
            <span class="card-title">Completed ({completedTransfers.length})</span>
            <button class="btn btn-icon btn-sm" onClick={clearCompleted}>
              Clear
            </button>
          </div>

          {completedTransfers.slice(-5).reverse().map(t => (
            <div class="transfer-item completed" key={t.transfer_id}>
              <div class="transfer-icon">
                {t.direction === 'Upload' ? '📤' : '📥'}
              </div>
              <div class="transfer-info">
                <div class="transfer-name">{t.file_name}</div>
                <div class="transfer-peer">
                  {t.direction === 'Upload' ? 'to' : 'from'} {t.peer_name}
                </div>
                {t.total_bytes > 0 && (
                  <div class="transfer-stats">{formatSize(t.total_bytes)}</div>
                )}
              </div>
              <div class="transfer-status" style={{ color: getStatusColor(t.status) }}>
                {t.status}
              </div>
              {t.error_message && (
                <span class="transfer-error" title={t.error_message}>⚠️</span>
              )}
            </div>
          ))}
        </div>
      )}

      {showUploadModal && selectedPeer && (
        <div class="modal-overlay" onClick={() => setShowUploadModal(false)}>
          <div class="modal" onClick={e => e.stopPropagation()}>
            <div class="modal-header">
              <span class="modal-title">Upload File to {selectedPeer.name}</span>
              <button class="modal-close" onClick={() => setShowUploadModal(false)}>×</button>
            </div>

            <div class="form-group">
              <label>Local File</label>
              <div style={{ display: 'flex', gap: '8px' }}>
                <input
                  type="text"
                  class="form-control"
                  value={localPath}
                  readOnly
                  placeholder="Select a file..."
                />
                <button class="btn btn-icon" onClick={selectFile}>Browse</button>
              </div>
            </div>

            <div class="form-group">
              <label>Remote Path (directory ending with / or full file path)</label>
              <input
                type="text"
                class="form-control"
                value={remotePath}
                onInput={e => setRemotePath(e.currentTarget.value)}
                placeholder="/home/user/"
              />
            </div>

            <div class="modal-actions">
              <button class="btn btn-icon" onClick={() => setShowUploadModal(false)}>Cancel</button>
              <button
                class="btn btn-primary"
                onClick={startUpload}
                disabled={!localPath || !remotePath}
              >
                Upload
              </button>
            </div>
          </div>
        </div>
      )}

      {showBrowseModal && selectedPeer && (
        <div class="modal-overlay" onClick={() => setShowBrowseModal(false)}>
          <div class="modal" onClick={e => e.stopPropagation()} style={{ maxWidth: '600px' }}>
            <div class="modal-header">
              <span class="modal-title">Browse {selectedPeer.name}</span>
              <button class="modal-close" onClick={() => setShowBrowseModal(false)}>×</button>
            </div>

            <div class="form-group">
              <div style={{ display: 'flex', gap: '8px' }}>
                <button
                  class="btn btn-icon btn-sm"
                  onClick={navigateUp}
                  disabled={browsePath === '/'}
                  title="Go up"
                >
                  ⬆
                </button>
                <input
                  type="text"
                  class="form-control"
                  value={browsePath}
                  onInput={e => setBrowsePath(e.currentTarget.value)}
                  onKeyDown={e => e.key === 'Enter' && browseRemote(selectedPeer, browsePath)}
                />
                <button
                  class="btn btn-icon"
                  onClick={() => browseRemote(selectedPeer, browsePath)}
                  disabled={browsing}
                >
                  Go
                </button>
              </div>
            </div>

            <div class="file-browser">
              {browsing ? (
                <div class="empty-state">Loading...</div>
              ) : remoteFiles.length === 0 ? (
                <div class="empty-state">Empty directory</div>
              ) : (
                remoteFiles.map(file => (
                  <div
                    class={`file-item ${file.is_dir ? 'directory' : 'file'}`}
                    key={file.path}
                    onClick={() => {
                      if (file.is_dir) {
                        browseRemote(selectedPeer, file.path);
                      }
                    }}
                  >
                    <span class="file-icon">{file.is_dir ? '📁' : '📄'}</span>
                    <span class="file-name">{file.name}</span>
                    <span class="file-size">{file.is_dir ? '' : formatSize(file.size)}</span>
                    {!file.is_dir && (
                      <button
                        class="btn btn-sm btn-icon"
                        onClick={(e) => {
                          e.stopPropagation();
                          startDownload(selectedPeer, file.path, file.name);
                        }}
                      >
                        📥
                      </button>
                    )}
                  </div>
                ))
              )}
            </div>
          </div>
        </div>
      )}
    </div>
  );
}
