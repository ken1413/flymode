use crate::p2p::{PeerDevice, P2PError, SSHClient};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use thiserror::Error;
use tokio::sync::{RwLock, Semaphore};
use tracing::error;

#[derive(Error, Debug)]
pub enum TransferError {
    #[error("P2P error: {0}")]
    P2P(#[from] P2PError),
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum TransferDirection {
    Upload,
    Download,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum TransferStatus {
    Pending,
    InProgress,
    Completed,
    Failed,
    Cancelled,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransferProgress {
    pub transfer_id: String,
    pub peer_id: String,
    pub peer_name: String,
    pub direction: TransferDirection,
    pub local_path: String,
    pub remote_path: String,
    pub file_name: String,
    pub total_bytes: u64,
    pub transferred_bytes: u64,
    pub status: TransferStatus,
    pub started_at: Option<DateTime<Utc>>,
    pub completed_at: Option<DateTime<Utc>>,
    pub error_message: Option<String>,
    pub speed_bps: Option<u64>,
}

impl TransferProgress {
    pub fn progress_percent(&self) -> f64 {
        if self.total_bytes == 0 {
            return 0.0;
        }
        (self.transferred_bytes as f64 / self.total_bytes as f64) * 100.0
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransferQueue {
    pub transfers: Vec<TransferProgress>,
    pub max_concurrent: usize,
}

impl Default for TransferQueue {
    fn default() -> Self {
        Self {
            transfers: Vec::new(),
            max_concurrent: 3,
        }
    }
}

const DEFAULT_MAX_CONCURRENT: usize = 3;

pub struct TransferManager {
    queue: Arc<RwLock<TransferQueue>>,
    semaphore: Arc<Semaphore>,
}

impl TransferManager {
    pub fn new() -> Self {
        Self {
            queue: Arc::new(RwLock::new(TransferQueue::default())),
            semaphore: Arc::new(Semaphore::new(DEFAULT_MAX_CONCURRENT)),
        }
    }

    pub fn max_concurrent(&self) -> usize {
        DEFAULT_MAX_CONCURRENT
    }

    pub async fn active_count(&self) -> usize {
        let q = self.queue.read().await;
        q.transfers.iter().filter(|t| t.status == TransferStatus::InProgress).count()
    }

    pub async fn get_queue(&self) -> TransferQueue {
        self.queue.read().await.clone()
    }

    pub async fn upload_file(
        &self,
        peer: &PeerDevice,
        local_path: PathBuf,
        remote_path: String,
    ) -> Result<String, TransferError> {
        let file_name = local_path
            .file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_else(|| "unknown".to_string());

        let file_size = std::fs::metadata(&local_path)
            .map(|m| m.len())
            .unwrap_or(0);

        let transfer_id = uuid::Uuid::new_v4().to_string();
        
        let progress = TransferProgress {
            transfer_id: transfer_id.clone(),
            peer_id: peer.id.clone(),
            peer_name: peer.name.clone(),
            direction: TransferDirection::Upload,
            local_path: local_path.to_string_lossy().to_string(),
            remote_path: remote_path.clone(),
            file_name,
            total_bytes: file_size,
            transferred_bytes: 0,
            status: TransferStatus::Pending,
            started_at: Some(Utc::now()),
            completed_at: None,
            error_message: None,
            speed_bps: None,
        };

        {
            let mut queue = self.queue.write().await;
            queue.transfers.push(progress);
        }

        let queue_clone = self.queue.clone();
        let peer_clone = peer.clone();
        let transfer_id_clone = transfer_id.clone();
        let semaphore = self.semaphore.clone();

        tokio::spawn(async move {
            let _permit = semaphore.acquire().await.expect("semaphore closed");
            if let Err(e) = Self::do_upload(queue_clone, peer_clone, local_path, remote_path, transfer_id_clone).await {
                error!("Upload failed: {}", e);
            }
        });

        Ok(transfer_id)
    }

    async fn do_upload(
        queue: Arc<RwLock<TransferQueue>>,
        peer: PeerDevice,
        local_path: PathBuf,
        remote_path: String,
        transfer_id: String,
    ) -> Result<(), TransferError> {
        {
            let mut q = queue.write().await;
            if let Some(t) = q.transfers.iter_mut().find(|t| t.transfer_id == transfer_id) {
                t.status = TransferStatus::InProgress;
            }
        }

        let mut ssh = SSHClient::new();

        if let Err(e) = ssh.connect(&peer) {
            let mut q = queue.write().await;
            if let Some(t) = q.transfers.iter_mut().find(|t| t.transfer_id == transfer_id) {
                t.status = TransferStatus::Failed;
                t.error_message = Some(e.to_string());
                t.completed_at = Some(Utc::now());
            }
            return Err(TransferError::P2P(e));
        }

        // Chunked upload with progress tracking via atomic counter
        let progress_bytes = Arc::new(AtomicU64::new(0));
        let progress_clone = progress_bytes.clone();
        let queue_clone = queue.clone();
        let tid = transfer_id.clone();

        // Background task polls progress and updates the queue
        let reporter = tokio::spawn(async move {
            let mut interval = tokio::time::interval(std::time::Duration::from_millis(200));
            loop {
                interval.tick().await;
                let bytes = progress_clone.load(Ordering::Relaxed);
                let mut q = queue_clone.write().await;
                if let Some(t) = q.transfers.iter_mut().find(|t| t.transfer_id == tid) {
                    t.transferred_bytes = bytes;
                    if t.status != TransferStatus::InProgress {
                        break;
                    }
                } else {
                    break;
                }
            }
        });

        let result: Result<(), TransferError> = ssh
            .upload_file_with_progress(&local_path, &remote_path, |bytes| {
                progress_bytes.store(bytes, Ordering::Relaxed);
            })
            .map_err(TransferError::P2P);
        ssh.disconnect();
        reporter.abort();

        let mut q = queue.write().await;
        if let Some(t) = q.transfers.iter_mut().find(|t| t.transfer_id == transfer_id) {
            match &result {
                Ok(_) => {
                    t.status = TransferStatus::Completed;
                    t.transferred_bytes = t.total_bytes;
                    t.completed_at = Some(Utc::now());
                }
                Err(e) => {
                    t.status = TransferStatus::Failed;
                    t.error_message = Some(e.to_string());
                    t.completed_at = Some(Utc::now());
                }
            }
        }

        result
    }

    pub async fn download_file(
        &self,
        peer: &PeerDevice,
        remote_path: String,
        local_path: PathBuf,
    ) -> Result<String, TransferError> {
        let file_name = PathBuf::from(&remote_path)
            .file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_else(|| "unknown".to_string());

        let transfer_id = uuid::Uuid::new_v4().to_string();
        
        let progress = TransferProgress {
            transfer_id: transfer_id.clone(),
            peer_id: peer.id.clone(),
            peer_name: peer.name.clone(),
            direction: TransferDirection::Download,
            local_path: local_path.to_string_lossy().to_string(),
            remote_path: remote_path.clone(),
            file_name,
            total_bytes: 0,
            transferred_bytes: 0,
            status: TransferStatus::Pending,
            started_at: Some(Utc::now()),
            completed_at: None,
            error_message: None,
            speed_bps: None,
        };

        {
            let mut queue = self.queue.write().await;
            queue.transfers.push(progress);
        }

        let queue_clone = self.queue.clone();
        let peer_clone = peer.clone();
        let transfer_id_clone = transfer_id.clone();
        let semaphore = self.semaphore.clone();

        tokio::spawn(async move {
            let _permit = semaphore.acquire().await.expect("semaphore closed");
            if let Err(e) = Self::do_download(queue_clone, peer_clone, remote_path, local_path, transfer_id_clone).await {
                error!("Download failed: {}", e);
            }
        });

        Ok(transfer_id)
    }

    async fn do_download(
        queue: Arc<RwLock<TransferQueue>>,
        peer: PeerDevice,
        remote_path: String,
        local_path: PathBuf,
        transfer_id: String,
    ) -> Result<(), TransferError> {
        {
            let mut q = queue.write().await;
            if let Some(t) = q.transfers.iter_mut().find(|t| t.transfer_id == transfer_id) {
                t.status = TransferStatus::InProgress;
            }
        }

        let mut ssh = SSHClient::new();

        if let Err(e) = ssh.connect(&peer) {
            let mut q = queue.write().await;
            if let Some(t) = q.transfers.iter_mut().find(|t| t.transfer_id == transfer_id) {
                t.status = TransferStatus::Failed;
                t.error_message = Some(e.to_string());
                t.completed_at = Some(Utc::now());
            }
            return Err(TransferError::P2P(e));
        }

        // Chunked download with progress tracking via atomic counter
        let progress_bytes = Arc::new(AtomicU64::new(0));
        let progress_clone = progress_bytes.clone();
        let queue_clone = queue.clone();
        let tid = transfer_id.clone();

        let reporter = tokio::spawn(async move {
            let mut interval = tokio::time::interval(std::time::Duration::from_millis(200));
            loop {
                interval.tick().await;
                let bytes = progress_clone.load(Ordering::Relaxed);
                let mut q = queue_clone.write().await;
                if let Some(t) = q.transfers.iter_mut().find(|t| t.transfer_id == tid) {
                    t.transferred_bytes = bytes;
                    if t.status != TransferStatus::InProgress {
                        break;
                    }
                } else {
                    break;
                }
            }
        });

        let result: Result<(), TransferError> = ssh
            .download_file_with_progress(&remote_path, &local_path, |bytes| {
                progress_bytes.store(bytes, Ordering::Relaxed);
            })
            .map_err(TransferError::P2P);
        ssh.disconnect();
        reporter.abort();

        let file_size = local_path.metadata().map(|m| m.len()).unwrap_or(0);

        let mut q = queue.write().await;
        if let Some(t) = q.transfers.iter_mut().find(|t| t.transfer_id == transfer_id) {
            match &result {
                Ok(_) => {
                    t.status = TransferStatus::Completed;
                    t.total_bytes = file_size;
                    t.transferred_bytes = file_size;
                    t.completed_at = Some(Utc::now());
                }
                Err(e) => {
                    t.status = TransferStatus::Failed;
                    t.error_message = Some(e.to_string());
                    t.completed_at = Some(Utc::now());
                }
            }
        }

        result
    }

    pub async fn cancel_transfer(&self, transfer_id: &str) -> Result<(), TransferError> {
        let mut queue = self.queue.write().await;
        if let Some(t) = queue.transfers.iter_mut().find(|t| t.transfer_id == transfer_id) {
            if t.status == TransferStatus::Pending || t.status == TransferStatus::InProgress {
                t.status = TransferStatus::Cancelled;
                t.completed_at = Some(Utc::now());
            }
        }
        Ok(())
    }

    pub async fn clear_completed(&self) {
        let mut queue = self.queue.write().await;
        queue.transfers.retain(|t| {
            t.status != TransferStatus::Completed 
                && t.status != TransferStatus::Failed 
                && t.status != TransferStatus::Cancelled
        });
    }

    pub async fn get_transfer(&self, transfer_id: &str) -> Option<TransferProgress> {
        let queue = self.queue.read().await;
        queue.transfers.iter().find(|t| t.transfer_id == transfer_id).cloned()
    }

    pub async fn browse_remote(&self, peer: &PeerDevice, path: &str) -> Result<Vec<crate::p2p::RemoteFileInfo>, TransferError> {
        let mut ssh = SSHClient::new();
        ssh.connect(peer)?;
        let files = ssh.list_remote_files(path)?;
        ssh.disconnect();
        Ok(files)
    }
}

impl Default for TransferManager {
    fn default() -> Self {
        Self::new()
    }
}

impl Clone for TransferManager {
    fn clone(&self) -> Self {
        Self {
            queue: self.queue.clone(),
            semaphore: self.semaphore.clone(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::p2p::{ConnectionType, DeviceStatus, PeerDevice};
    use pretty_assertions::assert_eq;

    fn create_test_peer() -> PeerDevice {
        PeerDevice {
            id: "test-peer-id".to_string(),
            name: "Test Peer".to_string(),
            hostname: "test.local".to_string(),
            ip_address: "192.168.1.100".to_string(),
            port: 22,
            connection_type: ConnectionType::LanDirect,
            status: DeviceStatus::Online,
            last_seen: None,
            ssh_user: "testuser".to_string(),
            ssh_key_path: None,
            ssh_password: Some("testpass".to_string()),
            is_trusted: true,
            tailscale_hostname: None,
            flymode_version: None,
        }
    }

    #[test]
    fn test_transfer_direction_serialization() {
        let directions = vec![TransferDirection::Upload, TransferDirection::Download];
        
        for dir in directions {
            let json = serde_json::to_string(&dir).expect("Failed to serialize");
            let deserialized: TransferDirection = serde_json::from_str(&json).expect("Failed to deserialize");
            assert_eq!(dir, deserialized);
        }
    }

    #[test]
    fn test_transfer_status_serialization() {
        let statuses = vec![
            TransferStatus::Pending,
            TransferStatus::InProgress,
            TransferStatus::Completed,
            TransferStatus::Failed,
            TransferStatus::Cancelled,
        ];
        
        for status in statuses {
            let json = serde_json::to_string(&status).expect("Failed to serialize");
            let deserialized: TransferStatus = serde_json::from_str(&json).expect("Failed to deserialize");
            assert_eq!(status, deserialized);
        }
    }

    #[test]
    fn test_transfer_progress_percent() {
        let progress = TransferProgress {
            transfer_id: "test".to_string(),
            peer_id: "peer".to_string(),
            peer_name: "Peer".to_string(),
            direction: TransferDirection::Upload,
            local_path: "/local/file".to_string(),
            remote_path: "/remote/file".to_string(),
            file_name: "file.txt".to_string(),
            total_bytes: 1000,
            transferred_bytes: 500,
            status: TransferStatus::InProgress,
            started_at: Some(Utc::now()),
            completed_at: None,
            error_message: None,
            speed_bps: Some(100),
        };
        
        assert_eq!(progress.progress_percent(), 50.0);
    }

    #[test]
    fn test_transfer_progress_percent_zero_total() {
        let progress = TransferProgress {
            transfer_id: "test".to_string(),
            peer_id: "peer".to_string(),
            peer_name: "Peer".to_string(),
            direction: TransferDirection::Upload,
            local_path: "/local/file".to_string(),
            remote_path: "/remote/file".to_string(),
            file_name: "file.txt".to_string(),
            total_bytes: 0,
            transferred_bytes: 0,
            status: TransferStatus::Pending,
            started_at: Some(Utc::now()),
            completed_at: None,
            error_message: None,
            speed_bps: None,
        };
        
        assert_eq!(progress.progress_percent(), 0.0);
    }

    #[test]
    fn test_transfer_progress_serialization() {
        let progress = TransferProgress {
            transfer_id: "id-123".to_string(),
            peer_id: "peer-456".to_string(),
            peer_name: "Test Peer".to_string(),
            direction: TransferDirection::Download,
            local_path: "/local/path/file.txt".to_string(),
            remote_path: "/remote/path/file.txt".to_string(),
            file_name: "file.txt".to_string(),
            total_bytes: 2048,
            transferred_bytes: 1024,
            status: TransferStatus::InProgress,
            started_at: Some(Utc::now()),
            completed_at: None,
            error_message: None,
            speed_bps: Some(512),
        };
        
        let json = serde_json::to_string(&progress).expect("Failed to serialize");
        let deserialized: TransferProgress = serde_json::from_str(&json).expect("Failed to deserialize");
        
        assert_eq!(progress.transfer_id, deserialized.transfer_id);
        assert_eq!(progress.direction, deserialized.direction);
        assert_eq!(progress.status, deserialized.status);
    }

    #[test]
    fn test_transfer_queue_default() {
        let queue = TransferQueue::default();
        
        assert!(queue.transfers.is_empty());
        assert_eq!(queue.max_concurrent, 3);
    }

    #[test]
    fn test_transfer_queue_serialization() {
        let queue = TransferQueue {
            transfers: vec![TransferProgress {
                transfer_id: "t1".to_string(),
                peer_id: "p1".to_string(),
                peer_name: "Peer".to_string(),
                direction: TransferDirection::Upload,
                local_path: "/local".to_string(),
                remote_path: "/remote".to_string(),
                file_name: "file.txt".to_string(),
                total_bytes: 100,
                transferred_bytes: 50,
                status: TransferStatus::InProgress,
                started_at: Some(Utc::now()),
                completed_at: None,
                error_message: None,
                speed_bps: None,
            }],
            max_concurrent: 5,
        };
        
        let json = serde_json::to_string(&queue).expect("Failed to serialize");
        let deserialized: TransferQueue = serde_json::from_str(&json).expect("Failed to deserialize");
        
        assert_eq!(queue.transfers.len(), deserialized.transfers.len());
        assert_eq!(queue.max_concurrent, deserialized.max_concurrent);
    }

    #[test]
    fn test_transfer_manager_new() {
        let manager = TransferManager::new();
        let queue = tokio_test::block_on(manager.get_queue());
        
        assert!(queue.transfers.is_empty());
    }

    #[test]
    fn test_transfer_manager_default() {
        let manager = TransferManager::default();
        let queue = tokio_test::block_on(manager.get_queue());
        
        assert!(queue.transfers.is_empty());
        assert_eq!(queue.max_concurrent, 3);
    }

    #[test]
    fn test_transfer_error_display() {
        let err = TransferError::Io(std::io::Error::new(std::io::ErrorKind::NotFound, "file not found"));
        assert!(err.to_string().contains("IO error"));
    }

    #[tokio::test]
    async fn test_transfer_manager_get_queue_empty() {
        let manager = TransferManager::new();
        let queue = manager.get_queue().await;
        
        assert!(queue.transfers.is_empty());
    }

    #[tokio::test]
    async fn test_transfer_manager_get_transfer_not_found() {
        let manager = TransferManager::new();
        let result = manager.get_transfer("nonexistent").await;
        
        assert!(result.is_none());
    }

    #[tokio::test]
    async fn test_transfer_manager_clear_completed_empty() {
        let manager = TransferManager::new();
        manager.clear_completed().await;

        let queue = manager.get_queue().await;
        assert!(queue.transfers.is_empty());
    }

    #[tokio::test]
    async fn test_cancel_pending_transfer() {
        let manager = TransferManager::new();
        let transfer_id = "test-cancel-pending";

        // Manually push a pending transfer
        {
            let mut q = manager.queue.write().await;
            q.transfers.push(TransferProgress {
                transfer_id: transfer_id.to_string(),
                peer_id: "p1".to_string(),
                peer_name: "Peer".to_string(),
                direction: TransferDirection::Upload,
                local_path: "/local".to_string(),
                remote_path: "/remote".to_string(),
                file_name: "file.txt".to_string(),
                total_bytes: 1000,
                transferred_bytes: 0,
                status: TransferStatus::Pending,
                started_at: Some(Utc::now()),
                completed_at: None,
                error_message: None,
                speed_bps: None,
            });
        }

        manager.cancel_transfer(transfer_id).await.unwrap();

        let t = manager.get_transfer(transfer_id).await.unwrap();
        assert_eq!(t.status, TransferStatus::Cancelled);
        assert!(t.completed_at.is_some());
    }

    #[tokio::test]
    async fn test_cancel_completed_does_nothing() {
        let manager = TransferManager::new();
        let transfer_id = "test-cancel-complete";

        {
            let mut q = manager.queue.write().await;
            q.transfers.push(TransferProgress {
                transfer_id: transfer_id.to_string(),
                peer_id: "p1".to_string(),
                peer_name: "Peer".to_string(),
                direction: TransferDirection::Upload,
                local_path: "/local".to_string(),
                remote_path: "/remote".to_string(),
                file_name: "file.txt".to_string(),
                total_bytes: 1000,
                transferred_bytes: 1000,
                status: TransferStatus::Completed,
                started_at: Some(Utc::now()),
                completed_at: Some(Utc::now()),
                error_message: None,
                speed_bps: None,
            });
        }

        manager.cancel_transfer(transfer_id).await.unwrap();

        let t = manager.get_transfer(transfer_id).await.unwrap();
        assert_eq!(t.status, TransferStatus::Completed, "Completed transfer should not be cancelled");
    }

    #[tokio::test]
    async fn test_clear_completed_retains_active() {
        let manager = TransferManager::new();

        {
            let mut q = manager.queue.write().await;
            // Active transfer
            q.transfers.push(TransferProgress {
                transfer_id: "active".to_string(),
                peer_id: "p1".to_string(),
                peer_name: "Peer".to_string(),
                direction: TransferDirection::Upload,
                local_path: "/local".to_string(),
                remote_path: "/remote".to_string(),
                file_name: "a.txt".to_string(),
                total_bytes: 1000,
                transferred_bytes: 500,
                status: TransferStatus::InProgress,
                started_at: Some(Utc::now()),
                completed_at: None,
                error_message: None,
                speed_bps: None,
            });
            // Completed transfer
            q.transfers.push(TransferProgress {
                transfer_id: "done".to_string(),
                peer_id: "p1".to_string(),
                peer_name: "Peer".to_string(),
                direction: TransferDirection::Download,
                local_path: "/local".to_string(),
                remote_path: "/remote".to_string(),
                file_name: "b.txt".to_string(),
                total_bytes: 2000,
                transferred_bytes: 2000,
                status: TransferStatus::Completed,
                started_at: Some(Utc::now()),
                completed_at: Some(Utc::now()),
                error_message: None,
                speed_bps: None,
            });
            // Failed transfer
            q.transfers.push(TransferProgress {
                transfer_id: "fail".to_string(),
                peer_id: "p1".to_string(),
                peer_name: "Peer".to_string(),
                direction: TransferDirection::Upload,
                local_path: "/local".to_string(),
                remote_path: "/remote".to_string(),
                file_name: "c.txt".to_string(),
                total_bytes: 3000,
                transferred_bytes: 100,
                status: TransferStatus::Failed,
                started_at: Some(Utc::now()),
                completed_at: Some(Utc::now()),
                error_message: Some("err".to_string()),
                speed_bps: None,
            });
        }

        manager.clear_completed().await;

        let q = manager.get_queue().await;
        assert_eq!(q.transfers.len(), 1, "Only active transfer should remain");
        assert_eq!(q.transfers[0].transfer_id, "active");
    }

    #[tokio::test]
    async fn test_concurrent_limit_respected() {
        let manager = TransferManager::new();
        assert_eq!(manager.active_count().await, 0);
        assert_eq!(manager.max_concurrent(), 3);

        // Simulate 3 in-progress transfers
        {
            let mut q = manager.queue.write().await;
            for i in 0..3 {
                q.transfers.push(TransferProgress {
                    transfer_id: format!("t{}", i),
                    peer_id: "p1".to_string(),
                    peer_name: "Peer".to_string(),
                    direction: TransferDirection::Upload,
                    local_path: "/local".to_string(),
                    remote_path: "/remote".to_string(),
                    file_name: format!("{}.txt", i),
                    total_bytes: 1000,
                    transferred_bytes: 500,
                    status: TransferStatus::InProgress,
                    started_at: Some(Utc::now()),
                    completed_at: None,
                    error_message: None,
                    speed_bps: None,
                });
            }
        }

        assert_eq!(manager.active_count().await, 3);
    }

    #[test]
    fn test_progress_percent_boundaries() {
        let make = |transferred, total| {
            TransferProgress {
                transfer_id: "t".to_string(),
                peer_id: "p".to_string(),
                peer_name: "P".to_string(),
                direction: TransferDirection::Upload,
                local_path: "/l".to_string(),
                remote_path: "/r".to_string(),
                file_name: "f".to_string(),
                total_bytes: total,
                transferred_bytes: transferred,
                status: TransferStatus::InProgress,
                started_at: None,
                completed_at: None,
                error_message: None,
                speed_bps: None,
            }
        };

        assert_eq!(make(0, 1000).progress_percent(), 0.0);
        assert_eq!(make(250, 1000).progress_percent(), 25.0);
        assert_eq!(make(1000, 1000).progress_percent(), 100.0);
        assert_eq!(make(0, 0).progress_percent(), 0.0);
    }
}
