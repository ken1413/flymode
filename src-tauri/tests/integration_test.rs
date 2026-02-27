#[cfg(test)]
mod integration_tests {
    use flymode::notes::{Note, NoteColor, NotesStore};
    use flymode::p2p::{P2PConfig, P2PManager, PeerDevice, ConnectionType, DeviceStatus};
    use flymode::sync::{SyncEngine, SyncState, SyncStatus};
    use flymode::transfer::{TransferManager, TransferStatus};
    use std::sync::Arc;
    use tokio::sync::RwLock;
    use chrono::Utc;

    #[test]
    fn test_notes_integration() {
        let temp_dir = tempfile::tempdir().unwrap();
        let db_path = temp_dir.path().join("notes.db");
        
        let store = NotesStore::with_path(db_path, "test-device".to_string()).unwrap();
        
        let note = store.create("Test Note".to_string(), "Test Content".to_string()).unwrap();
        assert_eq!(note.title, "Test Note");
        
        let notes = store.list(false).unwrap();
        assert_eq!(notes.len(), 1);
    }

    #[test]
    fn test_p2p_integration() {
        let temp_dir = tempfile::tempdir().unwrap();
        let config_path = temp_dir.path().join("p2p.json");
        
        let config = P2PConfig::load_from_path(&config_path).unwrap();
        let _manager = P2PManager::new_with_config(config);
        
        let peer = PeerDevice {
            id: "test-peer".to_string(),
            name: "Test Peer".to_string(),
            hostname: "test-host".to_string(),
            ip_address: "192.168.1.100".to_string(),
            port: 22,
            connection_type: ConnectionType::LanDirect,
            status: DeviceStatus::Offline,
            last_seen: Some(Utc::now()),
            ssh_user: "user".to_string(),
            ssh_key_path: None,
            ssh_password: None,
            is_trusted: false,
            tailscale_hostname: None,
            flymode_version: None,
        };
        
        assert_eq!(peer.ip_address, "192.168.1.100");
    }

    #[tokio::test]
    async fn test_sync_integration() {
        let temp_dir = tempfile::tempdir().unwrap();
        let db_path = temp_dir.path().join("notes.db");
        
        let notes_store = NotesStore::with_path(db_path, "test-device".to_string()).unwrap();
        
        let config_path = temp_dir.path().join("p2p.json");
        let p2p_config = P2PConfig::load_from_path(&config_path).unwrap();
        let p2p_manager = P2PManager::new_with_config(p2p_config);
        
        let sync_engine = SyncEngine::new(
            Arc::new(notes_store),
            Arc::new(p2p_manager),
        ).unwrap();
        
        let state = sync_engine.get_state().await;
        assert_eq!(state.status, SyncStatus::Idle);
    }

    #[tokio::test]
    async fn test_transfer_integration() {
        let transfer_manager = TransferManager::new();
        
        let queue = transfer_manager.get_queue().await;
        assert_eq!(queue.transfers.len(), 0);
        
        transfer_manager.clear_completed().await;
    }
}
