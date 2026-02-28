use std::path::PathBuf;
use tempfile::TempDir;

pub struct TestContext {
    pub temp_dir: TempDir,
    pub data_dir: PathBuf,
    pub config_dir: PathBuf,
}

impl TestContext {
    pub fn new() -> Self {
        let temp_dir = tempfile::tempdir().expect("Failed to create temp dir");
        let data_dir = temp_dir.path().join("data");
        let config_dir = temp_dir.path().join("config");
        
        std::fs::create_dir_all(&data_dir).expect("Failed to create data dir");
        std::fs::create_dir_all(&config_dir).expect("Failed to create config dir");
        
        Self {
            temp_dir,
            data_dir,
            config_dir,
        }
    }

    pub fn notes_db_path(&self) -> PathBuf {
        self.data_dir.join("notes.db")
    }

    pub fn config_path(&self) -> PathBuf {
        self.config_dir.join("config.json")
    }

    pub fn p2p_config_path(&self) -> PathBuf {
        self.config_dir.join("p2p.json")
    }
}

impl Default for TestContext {
    fn default() -> Self {
        Self::new()
    }
}

pub fn create_test_note(title: &str, content: &str) -> crate::notes::Note {
    crate::notes::Note::new(
        title.to_string(),
        content.to_string(),
        "test-device-id".to_string(),
    )
}

pub fn create_test_peer(name: &str, ip: &str) -> crate::p2p::PeerDevice {
    crate::p2p::PeerDevice {
        id: uuid::Uuid::new_v4().to_string(),
        name: name.to_string(),
        hostname: format!("{}.local", name.to_lowercase()),
        ip_address: ip.to_string(),
        port: 22,
        connection_type: crate::p2p::ConnectionType::LanDirect,
        status: crate::p2p::DeviceStatus::Online,
        last_seen: Some(chrono::Utc::now()),
        ssh_user: "testuser".to_string(),
        ssh_key_path: None,
        ssh_password: Some("testpass".to_string()),
        is_trusted: true,
        tailscale_hostname: None,
        flymode_version: Some("0.3.0".to_string()),
    }
}

pub fn assert_notes_equal(left: &crate::notes::Note, right: &crate::notes::Note) {
    use pretty_assertions::assert_eq;
    assert_eq!(left.id, right.id);
    assert_eq!(left.title, right.title);
    assert_eq!(left.content, right.content);
    assert_eq!(left.color, right.color);
    assert_eq!(left.category, right.category);
}
