use crate::notes::{Note, NotesStore};
use crate::p2p::{P2PManager, PeerDevice, SSHClient, P2PError};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use thiserror::Error;
use tokio::sync::RwLock;
use tracing::{info, warn};

#[derive(Error, Debug)]
pub enum SyncError {
    #[error("P2P error: {0}")]
    P2P(#[from] P2PError),
    #[error("Notes error: {0}")]
    Notes(#[from] crate::notes::NotesError),
    #[error("Sync error: {0}")]
    Sync(String),
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Serialization error: {0}")]
    Serialization(String),
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum SyncStatus {
    Idle,
    Syncing,
    Success,
    Error,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncResult {
    pub peer_id: String,
    pub peer_name: String,
    pub status: SyncStatus,
    pub notes_synced: usize,
    pub files_synced: usize,
    pub timestamp: DateTime<Utc>,
    pub error_message: Option<String>,
    pub duration_ms: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncState {
    pub last_sync: Option<DateTime<Utc>>,
    pub status: SyncStatus,
    pub current_peer: Option<String>,
    pub results: Vec<SyncResult>,
}

impl Default for SyncState {
    fn default() -> Self {
        Self {
            last_sync: None,
            status: SyncStatus::Idle,
            current_peer: None,
            results: Vec::new(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncPayload {
    pub device_id: String,
    pub device_name: String,
    pub timestamp: DateTime<Utc>,
    pub notes: Vec<Note>,
    pub sync_folder_files: Vec<FileSyncInfo>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileSyncInfo {
    pub path: String,
    pub relative_path: String,
    pub size: u64,
    pub modified: DateTime<Utc>,
    pub hash: String,
}

/// Read notes from a SQLite database file (used to read downloaded remote DBs).
fn read_notes_from_db(db_path: &std::path::Path, since: DateTime<Utc>) -> Result<Vec<Note>, SyncError> {
    let conn = rusqlite::Connection::open(db_path)
        .map_err(|e| SyncError::Sync(format!("Failed to open remote DB: {}", e)))?;

    let mut stmt = conn.prepare(
        "SELECT id, title, content, color, category, pinned, archived,
                created_at, updated_at, tags, position_x, position_y,
                width, height, device_id, sync_hash, deleted
         FROM notes WHERE updated_at > ?1
         ORDER BY updated_at ASC",
    ).map_err(|e| SyncError::Sync(format!("Failed to prepare query: {}", e)))?;

    let notes = stmt
        .query_map(rusqlite::params![since.to_rfc3339()], |row| {
            Ok(Note {
                id: row.get(0)?,
                title: row.get(1)?,
                content: row.get(2)?,
                color: serde_json::from_str(&row.get::<_, String>(3)?).unwrap_or_default(),
                category: serde_json::from_str(&row.get::<_, String>(4)?).unwrap_or_default(),
                pinned: row.get::<_, i32>(5)? != 0,
                archived: row.get::<_, i32>(6)? != 0,
                created_at: DateTime::parse_from_rfc3339(&row.get::<_, String>(7)?)
                    .map(|dt| dt.with_timezone(&Utc))
                    .unwrap_or_else(|_| Utc::now()),
                updated_at: DateTime::parse_from_rfc3339(&row.get::<_, String>(8)?)
                    .map(|dt| dt.with_timezone(&Utc))
                    .unwrap_or_else(|_| Utc::now()),
                tags: serde_json::from_str(&row.get::<_, String>(9)?).unwrap_or_default(),
                position_x: row.get(10)?,
                position_y: row.get(11)?,
                width: row.get(12)?,
                height: row.get(13)?,
                device_id: row.get(14)?,
                sync_hash: row.get(15)?,
                deleted: row.get::<_, i32>(16)? != 0,
            })
        })
        .map_err(|e| SyncError::Sync(format!("Failed to query notes: {}", e)))?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| SyncError::Sync(format!("Failed to read notes: {}", e)))?;

    Ok(notes)
}

/// Last-Write-Wins merge of two sets of notes.
///
/// For notes present on both sides, the one with the newer `updated_at` wins.
/// Ties (same timestamp) keep the local version.
/// Notes only on one side are included as-is.
pub fn merge_notes(local: &[Note], remote: &[Note]) -> Vec<Note> {
    let remote_map: HashMap<&str, &Note> = remote.iter().map(|n| (n.id.as_str(), n)).collect();
    let mut seen = HashMap::new();
    let mut result = Vec::with_capacity(local.len() + remote.len());

    for local_note in local {
        seen.insert(local_note.id.as_str(), true);
        if let Some(remote_note) = remote_map.get(local_note.id.as_str()) {
            // Both sides have this note — LWW by updated_at
            if remote_note.updated_at > local_note.updated_at {
                result.push((*remote_note).clone());
            } else {
                // Same or local is newer → keep local
                result.push(local_note.clone());
            }
        } else {
            // Local only
            result.push(local_note.clone());
        }
    }

    // Remote-only notes
    for remote_note in remote {
        if !seen.contains_key(remote_note.id.as_str()) {
            result.push(remote_note.clone());
        }
    }

    result
}

pub struct SyncEngine {
    notes_store: Arc<NotesStore>,
    p2p_manager: Arc<P2PManager>,
    state: Arc<RwLock<SyncState>>,
    sync_folder: PathBuf,
}

impl SyncEngine {
    pub fn new(
        notes_store: Arc<NotesStore>,
        p2p_manager: Arc<P2PManager>,
    ) -> Result<Self, SyncError> {
        let sync_folder = dirs::data_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("flymode")
            .join("sync");
        
        if !sync_folder.exists() {
            std::fs::create_dir_all(&sync_folder)?;
        }

        Ok(Self {
            notes_store,
            p2p_manager,
            state: Arc::new(RwLock::new(SyncState::default())),
            sync_folder,
        })
    }

    pub async fn get_state(&self) -> SyncState {
        self.state.read().await.clone()
    }

    pub async fn sync_with_peer(&self, peer: &PeerDevice) -> Result<SyncResult, SyncError> {
        let start = std::time::Instant::now();
        
        {
            let mut state = self.state.write().await;
            state.status = SyncStatus::Syncing;
            state.current_peer = Some(peer.id.clone());
        }

        let result = self.do_sync_with_peer(peer).await;
        
        let duration_ms = start.elapsed().as_millis() as u64;
        
        let sync_result = match result {
            Ok((notes_synced, files_synced)) => SyncResult {
                peer_id: peer.id.clone(),
                peer_name: peer.name.clone(),
                status: SyncStatus::Success,
                notes_synced,
                files_synced,
                timestamp: Utc::now(),
                error_message: None,
                duration_ms,
            },
            Err(e) => SyncResult {
                peer_id: peer.id.clone(),
                peer_name: peer.name.clone(),
                status: SyncStatus::Error,
                notes_synced: 0,
                files_synced: 0,
                timestamp: Utc::now(),
                error_message: Some(e.to_string()),
                duration_ms,
            },
        };

        {
            let mut state = self.state.write().await;
            state.status = SyncStatus::Idle;
            state.current_peer = None;
            state.last_sync = Some(Utc::now());
            state.results.push(sync_result.clone());
            if state.results.len() > 50 {
                state.results.remove(0);
            }
        }

        Ok(sync_result)
    }

    async fn do_sync_with_peer(&self, peer: &PeerDevice) -> Result<(usize, usize), SyncError> {
        let mut ssh = SSHClient::new();
        ssh.connect(peer)
            .map_err(|e| SyncError::Sync(format!("SSH connection failed: {}", e)))?;

        let since = self.state.read().await.last_sync
            .unwrap_or_else(|| Utc::now() - chrono::Duration::days(30));
        let local_notes = self.notes_store.get_changes_since(since)?;

        // 1. Upload local changes as JSON payload via SFTP
        let config = self.p2p_manager.get_config().await;
        let payload_json = serde_json::to_string(&SyncPayload {
            device_id: config.device_id,
            device_name: config.device_name,
            timestamp: Utc::now(),
            notes: local_notes.clone(),
            sync_folder_files: Vec::new(),
        }).map_err(|e| SyncError::Serialization(e.to_string()))?;

        let remote_flymode_dir = ".flymode";
        let remote_sync_file = format!("{}/sync_in.json", remote_flymode_dir);
        ssh.execute_command(&format!("mkdir -p {}", remote_flymode_dir))?;

        let temp_dir = std::env::temp_dir();
        let temp_payload = temp_dir.join("flymode_sync_payload.json");
        std::fs::write(&temp_payload, &payload_json)?;
        ssh.upload_file(&temp_payload, &remote_sync_file)?;

        // 2. Download remote's notes DB via SFTP and read remote notes
        let remote_db_path = ".local/share/flymode/notes.db";
        let temp_remote_db = temp_dir.join("flymode_remote_notes.db");

        let remote_notes = match ssh.download_file(remote_db_path, &temp_remote_db.clone().into()) {
            Ok(_) => {
                let notes = read_notes_from_db(&temp_remote_db, since);
                let _ = std::fs::remove_file(&temp_remote_db);
                notes.unwrap_or_default()
            }
            Err(_) => {
                info!("Remote notes DB not found, treating as empty");
                Vec::new()
            }
        };

        // 3. Merge using LWW and apply
        let to_apply: Vec<Note> = merge_notes(&local_notes, &remote_notes)
            .into_iter()
            .filter(|n| !local_notes.iter().any(|ln| ln.id == n.id && ln.updated_at == n.updated_at))
            .collect();

        let applied = self.notes_store.apply_remote_changes(to_apply)?;

        // 4. Cleanup remote sync file
        let _ = ssh.execute_command(&format!("rm -f {}", remote_sync_file));
        let _ = std::fs::remove_file(&temp_payload);

        ssh.disconnect();

        Ok((applied, 0))
    }

    fn parse_sync_response(&self, response: &str) -> Result<Vec<Note>, SyncError> {
        let start_marker = "SYNC_RESPONSE_START";
        let end_marker = "SYNC_RESPONSE_END";
        
        if let Some(start) = response.find(start_marker) {
            if let Some(end) = response.find(end_marker) {
                let json_str = &response[start + start_marker.len()..end].trim();
                if json_str.is_empty() || *json_str == "{}" {
                    return Ok(Vec::new());
                }
                
                let payload: SyncPayload = serde_json::from_str(json_str)
                    .map_err(|e| SyncError::Serialization(e.to_string()))?;
                return Ok(payload.notes);
            }
        }

        Ok(Vec::new())
    }

    pub async fn sync_all_peers(&self) -> Vec<SyncResult> {
        let config = self.p2p_manager.get_config().await;
        let mut results = Vec::new();

        for peer in &config.peers {
            if peer.is_trusted {
                match self.sync_with_peer(peer).await {
                    Ok(result) => results.push(result),
                    Err(e) => {
                        warn!("Failed to sync with peer {}: {}", peer.name, e);
                        results.push(SyncResult {
                            peer_id: peer.id.clone(),
                            peer_name: peer.name.clone(),
                            status: SyncStatus::Error,
                            notes_synced: 0,
                            files_synced: 0,
                            timestamp: Utc::now(),
                            error_message: Some(e.to_string()),
                            duration_ms: 0,
                        });
                    }
                }
            }
        }

        results
    }

    pub async fn start_auto_sync(&self) {
        let interval_secs = self.p2p_manager.get_config().await.sync_interval_seconds;
        let state = self.state.clone();
        let p2p = self.p2p_manager.clone();
        let notes = self.notes_store.clone();
        let sync_folder = self.sync_folder.clone();

        tokio::spawn(async move {
            let mut interval = tokio::time::interval(
                std::time::Duration::from_secs(interval_secs)
            );

            loop {
                interval.tick().await;

                let config = p2p.get_config().await;
                if !config.sync_enabled {
                    continue;
                }

                info!("Starting auto-sync...");

                let engine = SyncEngine {
                    notes_store: notes.clone(),
                    p2p_manager: p2p.clone(),
                    state: state.clone(),
                    sync_folder: sync_folder.clone(),
                };

                let results = engine.sync_all_peers().await;
                info!("Auto-sync completed: {} peers processed", results.len());
            }
        });
    }

    pub fn get_sync_folder(&self) -> &PathBuf {
        &self.sync_folder
    }

    pub async fn export_notes(&self) -> Result<String, SyncError> {
        let notes = self.notes_store.list(true)?;
        let json = serde_json::to_string_pretty(&notes)
            .map_err(|e| SyncError::Serialization(e.to_string()))?;
        Ok(json)
    }

    pub async fn import_notes(&self, json: &str) -> Result<usize, SyncError> {
        let notes: Vec<Note> = serde_json::from_str(json)
            .map_err(|e| SyncError::Serialization(e.to_string()))?;
        
        let count = self.notes_store.apply_remote_changes(notes)?;
        Ok(count)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::notes::{Note, NoteCategory, NoteColor, NotesStore};
    use crate::p2p::{P2PConfig, PeerDevice};
    use chrono::Duration;
    use pretty_assertions::assert_eq;
    use tempfile::TempDir;

    /// Helper: create a Note with specific id, title, device, and timestamp
    fn make_note(id: &str, title: &str, device_id: &str, updated_at: DateTime<Utc>) -> Note {
        Note {
            id: id.to_string(),
            title: title.to_string(),
            content: "content".to_string(),
            color: NoteColor::default(),
            category: NoteCategory::default(),
            pinned: false,
            archived: false,
            created_at: updated_at,
            updated_at,
            tags: Vec::new(),
            position_x: 0,
            position_y: 0,
            width: 280,
            height: 200,
            device_id: device_id.to_string(),
            sync_hash: None,
            deleted: false,
        }
    }

    struct TestFixture {
        _temp_dir: TempDir,
        notes_store: NotesStore,
        sync_engine: SyncEngine,
    }

    impl TestFixture {
        fn new() -> Self {
            let temp_dir = tempfile::tempdir().expect("Failed to create temp dir");
            let db_path = temp_dir.path().join("notes.db");
            let notes_store = NotesStore::with_path(db_path, "test-device".to_string())
                .expect("Failed to create notes store");
            
            let config_path = temp_dir.path().join("p2p.json");
            let p2p_config = P2PConfig::load_from_path(&config_path)
                .expect("Failed to create P2P config");
            
            let p2p_manager = P2PManager::new_with_config(p2p_config);
            let sync_engine = SyncEngine::new(Arc::new(notes_store.clone()), Arc::new(p2p_manager))
                .expect("Failed to create sync engine");
            
            Self {
                _temp_dir: temp_dir,
                notes_store,
                sync_engine,
            }
        }

        fn create_test_note(&self, title: &str, content: &str) -> Note {
            self.notes_store.create(title.to_string(), content.to_string())
                .expect("Failed to create note")
        }
    }

    #[test]
    fn test_sync_state_default() {
        let state = SyncState::default();
        
        assert!(state.last_sync.is_none());
        assert_eq!(state.status, SyncStatus::Idle);
        assert!(state.current_peer.is_none());
        assert!(state.results.is_empty());
    }

    #[test]
    fn test_sync_status_serialization() {
        let state = SyncState {
            last_sync: Some(Utc::now()),
            status: SyncStatus::Success,
            current_peer: Some("peer-123".to_string()),
            results: vec![SyncResult {
                peer_id: "peer-123".to_string(),
                peer_name: "Test Peer".to_string(),
                status: SyncStatus::Success,
                notes_synced: 5,
                files_synced: 2,
                timestamp: Utc::now(),
                error_message: None,
                duration_ms: 1000,
            }],
        };
        
        let json = serde_json::to_string(&state).expect("Failed to serialize");
        let deserialized: SyncState = serde_json::from_str(&json).expect("Failed to deserialize");
        
        assert_eq!(state.status, deserialized.status);
        assert_eq!(state.results.len(), deserialized.results.len());
    }

    #[test]
    fn test_sync_result_serialization() {
        let result = SyncResult {
            peer_id: "peer-456".to_string(),
            peer_name: "Another Peer".to_string(),
            status: SyncStatus::Error,
            notes_synced: 0,
            files_synced: 0,
            timestamp: Utc::now(),
            error_message: Some("Connection failed".to_string()),
            duration_ms: 500,
        };
        
        let json = serde_json::to_string(&result).expect("Failed to serialize");
        let deserialized: SyncResult = serde_json::from_str(&json).expect("Failed to deserialize");
        
        assert_eq!(result.peer_id, deserialized.peer_id);
        assert_eq!(result.status, deserialized.status);
        assert_eq!(result.error_message, deserialized.error_message);
    }

    #[test]
    fn test_sync_payload_serialization() {
        let fixture = TestFixture::new();
        let note = fixture.create_test_note("Test", "Content");
        
        let payload = SyncPayload {
            device_id: "device-123".to_string(),
            device_name: "Test Device".to_string(),
            timestamp: Utc::now(),
            notes: vec![note],
            sync_folder_files: vec![FileSyncInfo {
                path: "/test/file.txt".to_string(),
                relative_path: "file.txt".to_string(),
                size: 1024,
                modified: Utc::now(),
                hash: "abc123".to_string(),
            }],
        };
        
        let json = serde_json::to_string(&payload).expect("Failed to serialize");
        let deserialized: SyncPayload = serde_json::from_str(&json).expect("Failed to deserialize");
        
        assert_eq!(payload.device_id, deserialized.device_id);
        assert_eq!(payload.notes.len(), deserialized.notes.len());
        assert_eq!(payload.sync_folder_files.len(), deserialized.sync_folder_files.len());
    }

    #[test]
    fn test_file_sync_info_serialization() {
        let info = FileSyncInfo {
            path: "/home/user/file.txt".to_string(),
            relative_path: "file.txt".to_string(),
            size: 2048,
            modified: Utc::now(),
            hash: "def456".to_string(),
        };
        
        let json = serde_json::to_string(&info).expect("Failed to serialize");
        let deserialized: FileSyncInfo = serde_json::from_str(&json).expect("Failed to deserialize");
        
        assert_eq!(info.path, deserialized.path);
        assert_eq!(info.size, deserialized.size);
        assert_eq!(info.hash, deserialized.hash);
    }

    #[test]
    fn test_sync_status_variants() {
        let statuses = vec![
            SyncStatus::Idle,
            SyncStatus::Syncing,
            SyncStatus::Success,
            SyncStatus::Error,
        ];
        
        for status in statuses {
            let json = serde_json::to_string(&status).expect("Failed to serialize");
            let deserialized: SyncStatus = serde_json::from_str(&json).expect("Failed to deserialize");
            assert_eq!(status, deserialized);
        }
    }

    #[test]
    fn test_sync_error_display() {
        let err = SyncError::P2P(P2PError::Connection("test".to_string()));
        assert!(err.to_string().contains("P2P error"));
        
        let err = SyncError::Sync("sync failed".to_string());
        assert!(err.to_string().contains("Sync error"));
        
        let err = SyncError::Serialization("bad json".to_string());
        assert!(err.to_string().contains("Serialization error"));
    }

    #[tokio::test]
    async fn test_sync_engine_get_state() {
        let fixture = TestFixture::new();
        
        let state = fixture.sync_engine.get_state().await;
        
        assert!(state.last_sync.is_none());
        assert_eq!(state.status, SyncStatus::Idle);
    }

    #[tokio::test]
    async fn test_sync_engine_export_notes() {
        let fixture = TestFixture::new();
        fixture.create_test_note("Note 1", "Content 1");
        fixture.create_test_note("Note 2", "Content 2");
        
        let json = fixture.sync_engine.export_notes().await.expect("Failed to export");
        
        assert!(json.contains("Note 1"));
        assert!(json.contains("Note 2"));
    }

    #[tokio::test]
    async fn test_sync_engine_import_notes() {
        let fixture = TestFixture::new();
        
        let note = Note::new("Imported".to_string(), "Content".to_string(), "remote-device".to_string());
        let json = serde_json::to_string(&vec![note]).expect("Failed to serialize");
        
        let count = fixture.sync_engine.import_notes(&json).await.expect("Failed to import");
        
        assert_eq!(count, 1);
        
        let notes = fixture.notes_store.list(false).expect("Failed to list");
        assert_eq!(notes.len(), 1);
        assert_eq!(notes[0].title, "Imported");
    }

    #[tokio::test]
    async fn test_sync_engine_import_empty() {
        let fixture = TestFixture::new();
        
        let count = fixture.sync_engine.import_notes("[]").await.expect("Failed to import");
        
        assert_eq!(count, 0);
    }

    #[test]
    fn test_sync_result_duration_formatting() {
        let result = SyncResult {
            peer_id: "p1".to_string(),
            peer_name: "Peer".to_string(),
            status: SyncStatus::Success,
            notes_synced: 1,
            files_synced: 0,
            timestamp: Utc::now(),
            error_message: None,
            duration_ms: 1500,
        };

        assert_eq!(result.duration_ms, 1500);
    }

    // ── merge_notes tests ───────────────────────────────────────────

    #[test]
    fn test_merge_both_empty() {
        let result = merge_notes(&[], &[]);
        assert!(result.is_empty());
    }

    #[test]
    fn test_merge_local_only() {
        let now = Utc::now();
        let local = vec![
            make_note("n1", "Local 1", "device-a", now),
            make_note("n2", "Local 2", "device-a", now),
        ];
        let result = merge_notes(&local, &[]);
        assert_eq!(result.len(), 2);
        assert_eq!(result[0].title, "Local 1");
        assert_eq!(result[1].title, "Local 2");
    }

    #[test]
    fn test_merge_remote_only() {
        let now = Utc::now();
        let remote = vec![
            make_note("n1", "Remote 1", "device-b", now),
        ];
        let result = merge_notes(&[], &remote);
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].title, "Remote 1");
    }

    #[test]
    fn test_merge_conflict_remote_newer_wins() {
        let now = Utc::now();
        let earlier = now - Duration::minutes(5);
        let local = vec![make_note("n1", "Local Version", "device-a", earlier)];
        let remote = vec![make_note("n1", "Remote Version", "device-b", now)];

        let result = merge_notes(&local, &remote);
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].title, "Remote Version");
        assert_eq!(result[0].device_id, "device-b");
    }

    #[test]
    fn test_merge_conflict_local_newer_wins() {
        let now = Utc::now();
        let earlier = now - Duration::minutes(5);
        let local = vec![make_note("n1", "Local Version", "device-a", now)];
        let remote = vec![make_note("n1", "Remote Version", "device-b", earlier)];

        let result = merge_notes(&local, &remote);
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].title, "Local Version");
        assert_eq!(result[0].device_id, "device-a");
    }

    #[test]
    fn test_merge_conflict_same_timestamp_keeps_local() {
        let now = Utc::now();
        let local = vec![make_note("n1", "Local Version", "device-a", now)];
        let remote = vec![make_note("n1", "Remote Version", "device-b", now)];

        let result = merge_notes(&local, &remote);
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].title, "Local Version");
    }

    #[test]
    fn test_merge_mixed_notes() {
        let now = Utc::now();
        let earlier = now - Duration::minutes(5);
        let local = vec![
            make_note("n1", "Local Only", "device-a", now),
            make_note("n2", "Conflict Local", "device-a", earlier),
            make_note("n3", "Conflict Local Newer", "device-a", now),
        ];
        let remote = vec![
            make_note("n2", "Conflict Remote", "device-b", now),
            make_note("n3", "Conflict Remote Older", "device-b", earlier),
            make_note("n4", "Remote Only", "device-b", now),
        ];

        let result = merge_notes(&local, &remote);
        assert_eq!(result.len(), 4);

        let by_id: HashMap<&str, &Note> = result.iter().map(|n| (n.id.as_str(), n)).collect();
        assert_eq!(by_id["n1"].title, "Local Only");
        assert_eq!(by_id["n2"].title, "Conflict Remote");    // remote newer
        assert_eq!(by_id["n3"].title, "Conflict Local Newer"); // local newer
        assert_eq!(by_id["n4"].title, "Remote Only");
    }

    #[test]
    fn test_merge_soft_delete_remote_newer() {
        let now = Utc::now();
        let earlier = now - Duration::minutes(5);
        let local = vec![make_note("n1", "Updated", "device-a", earlier)];
        let mut remote_note = make_note("n1", "Deleted", "device-b", now);
        remote_note.deleted = true;

        let result = merge_notes(&local, &[remote_note]);
        assert_eq!(result.len(), 1);
        assert!(result[0].deleted, "Remote delete should win (newer timestamp)");
    }

    #[test]
    fn test_merge_soft_delete_local_newer() {
        let now = Utc::now();
        let earlier = now - Duration::minutes(5);
        let mut local_note = make_note("n1", "Deleted", "device-a", now);
        local_note.deleted = true;
        let remote = vec![make_note("n1", "Updated", "device-b", earlier)];

        let result = merge_notes(&[local_note], &remote);
        assert_eq!(result.len(), 1);
        assert!(result[0].deleted, "Local delete should win (newer timestamp)");
    }

    #[test]
    fn test_merge_preserves_note_fields() {
        let now = Utc::now();
        let earlier = now - Duration::minutes(5);
        let mut local_note = make_note("n1", "Old", "device-a", earlier);
        local_note.color = NoteColor::Pink;
        local_note.pinned = true;
        local_note.tags = vec!["tag1".to_string()];

        let mut remote_note = make_note("n1", "New", "device-b", now);
        remote_note.color = NoteColor::Blue;
        remote_note.category = NoteCategory::Work;

        let result = merge_notes(&[local_note], &[remote_note]);
        assert_eq!(result[0].title, "New");
        assert_eq!(result[0].color, NoteColor::Blue);
        assert_eq!(result[0].category, NoteCategory::Work);
    }

    // ── parse_sync_response tests ───────────────────────────────────

    #[test]
    fn test_parse_response_valid_empty_json() {
        let fixture = TestFixture::new();
        let response = "some output\nSYNC_RESPONSE_START\n{}\nSYNC_RESPONSE_END\nmore output";
        let result = fixture.sync_engine.parse_sync_response(response).unwrap();
        assert!(result.is_empty());
    }

    #[test]
    fn test_parse_response_empty_string() {
        let fixture = TestFixture::new();
        let result = fixture.sync_engine.parse_sync_response("").unwrap();
        assert!(result.is_empty());
    }

    #[test]
    fn test_parse_response_missing_markers() {
        let fixture = TestFixture::new();
        let result = fixture.sync_engine.parse_sync_response("no markers here").unwrap();
        assert!(result.is_empty());
    }

    #[test]
    fn test_parse_response_missing_end_marker() {
        let fixture = TestFixture::new();
        let result = fixture.sync_engine.parse_sync_response("SYNC_RESPONSE_START\n{}\n").unwrap();
        assert!(result.is_empty());
    }

    #[test]
    fn test_parse_response_with_valid_payload() {
        let fixture = TestFixture::new();
        let note = Note::new("Synced".to_string(), "Content".to_string(), "remote".to_string());
        let payload = SyncPayload {
            device_id: "remote".to_string(),
            device_name: "Remote".to_string(),
            timestamp: Utc::now(),
            notes: vec![note],
            sync_folder_files: Vec::new(),
        };
        let json = serde_json::to_string(&payload).unwrap();
        let response = format!("output\nSYNC_RESPONSE_START\n{}\nSYNC_RESPONSE_END\n", json);

        let result = fixture.sync_engine.parse_sync_response(&response).unwrap();
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].title, "Synced");
    }

    #[test]
    fn test_parse_response_invalid_json() {
        let fixture = TestFixture::new();
        let response = "SYNC_RESPONSE_START\n{invalid json}\nSYNC_RESPONSE_END\n";
        let result = fixture.sync_engine.parse_sync_response(response);
        assert!(result.is_err());
    }

    // ── apply_remote_changes LWW tests ──────────────────────────────

    #[test]
    fn test_apply_remote_new_note() {
        let fixture = TestFixture::new();
        let remote_note = Note::new("Remote".to_string(), "Content".to_string(), "device-b".to_string());

        let applied = fixture.notes_store.apply_remote_changes(vec![remote_note]).unwrap();
        assert_eq!(applied, 1);

        let notes = fixture.notes_store.list(false).unwrap();
        assert_eq!(notes.len(), 1);
        assert_eq!(notes[0].title, "Remote");
    }

    #[test]
    fn test_apply_remote_newer_overwrites_local() {
        let fixture = TestFixture::new();
        let local_note = fixture.create_test_note("Old Local", "Content");

        // Create a newer remote version with same ID
        let mut remote_note = local_note.clone();
        remote_note.title = "Updated Remote".to_string();
        remote_note.updated_at = Utc::now() + Duration::minutes(5);
        remote_note.sync_hash = Some(remote_note.compute_hash());

        let applied = fixture.notes_store.apply_remote_changes(vec![remote_note]).unwrap();
        assert_eq!(applied, 1);

        let notes = fixture.notes_store.list(false).unwrap();
        assert_eq!(notes[0].title, "Updated Remote");
    }

    #[test]
    fn test_apply_remote_older_does_not_overwrite() {
        let fixture = TestFixture::new();
        let local_note = fixture.create_test_note("Current Local", "Content");

        // Create an older remote version with same ID
        let mut remote_note = local_note.clone();
        remote_note.title = "Old Remote".to_string();
        remote_note.updated_at = Utc::now() - Duration::hours(1);
        remote_note.sync_hash = Some(remote_note.compute_hash());

        let applied = fixture.notes_store.apply_remote_changes(vec![remote_note]).unwrap();
        assert_eq!(applied, 0, "Older remote note should NOT overwrite local");

        let notes = fixture.notes_store.list(false).unwrap();
        assert_eq!(notes[0].title, "Current Local");
    }
}
