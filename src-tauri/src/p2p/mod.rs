pub mod pair;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::io::Read;
use std::net::TcpStream;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;
use ssh2::Session;
use thiserror::Error;
use tokio::sync::RwLock;


#[derive(Error, Debug)]
pub enum P2PError {
    #[error("Connection error: {0}")]
    Connection(String),
    #[error("Authentication error: {0}")]
    Auth(String),
    #[error("Transfer error: {0}")]
    Transfer(String),
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("SSH error: {0}")]
    Ssh(#[from] ssh2::Error),
    #[error("Config error: {0}")]
    Config(String),
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ConnectionType {
    Tailscale,
    LanDirect,
    WanDirect,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum DeviceStatus {
    Online,
    Offline,
    Unknown,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PeerDevice {
    pub id: String,
    pub name: String,
    pub hostname: String,
    pub ip_address: String,
    pub port: u16,
    pub connection_type: ConnectionType,
    pub status: DeviceStatus,
    pub last_seen: Option<DateTime<Utc>>,
    pub ssh_user: String,
    pub ssh_key_path: Option<String>,
    pub ssh_password: Option<String>,
    pub is_trusted: bool,
    pub tailscale_hostname: Option<String>,
    pub flymode_version: Option<String>,
}

impl Default for PeerDevice {
    fn default() -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            name: String::new(),
            hostname: String::new(),
            ip_address: String::new(),
            port: 22,
            connection_type: ConnectionType::LanDirect,
            status: DeviceStatus::Unknown,
            last_seen: None,
            ssh_user: String::new(),
            ssh_key_path: None,
            ssh_password: None,
            is_trusted: false,
            tailscale_hostname: None,
            flymode_version: None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct P2PConfig {
    #[serde(skip)]
    pub config_path: PathBuf,
    pub device_id: String,
    pub device_name: String,
    pub listen_port: u16,
    pub peers: Vec<PeerDevice>,
    pub auto_discover_tailscale: bool,
    pub sync_enabled: bool,
    pub sync_interval_seconds: u64,
}

impl Default for P2PConfig {
    fn default() -> Self {
        Self {
            config_path: Self::config_path(),
            device_id: uuid::Uuid::new_v4().to_string(),
            device_name: hostname::get()
                .map(|h| h.to_string_lossy().to_string())
                .unwrap_or_else(|_| "Unknown".to_string()),
            listen_port: 4827,
            peers: Vec::new(),
            auto_discover_tailscale: true,
            sync_enabled: true,
            sync_interval_seconds: 300,
        }
    }
}

impl P2PConfig {
    pub fn config_path() -> PathBuf {
        dirs::config_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("flymode")
            .join("p2p.json")
    }

    pub fn with_path(path: PathBuf) -> Self {
        Self {
            config_path: path,
            ..Self::default()
        }
    }

    pub fn load() -> Result<Self, P2PError> {
        let path = Self::config_path();
        Self::load_from_path(&path)
    }

    pub fn load_from_path(path: &PathBuf) -> Result<Self, P2PError> {
        if !path.exists() {
            let config = Self {
                config_path: path.clone(),
                ..Self::default()
            };
            config.save_to_path(path)?;
            return Ok(config);
        }
        let content = std::fs::read_to_string(path)?;
        let mut config: P2PConfig = serde_json::from_str(&content)
            .map_err(|e| P2PError::Config(e.to_string()))?;
        config.config_path = path.clone();

        // Decrypt stored passwords
        let device_id = config.device_id.clone();
        for peer in &mut config.peers {
            if let Some(ref enc) = peer.ssh_password {
                match crate::crypto::decrypt(enc, &device_id) {
                    Ok(plain) => peer.ssh_password = Some(plain),
                    Err(_) => {
                        // Password may be in plaintext (pre-encryption migration) — keep as-is
                    }
                }
            }
        }

        Ok(config)
    }

    pub fn save(&self) -> Result<(), P2PError> {
        self.save_to_path(&self.config_path)
    }

    pub fn save_to_path(&self, path: &PathBuf) -> Result<(), P2PError> {
        if let Some(parent) = path.parent() {
            if !parent.exists() {
                std::fs::create_dir_all(parent)?;
            }
        }

        // Clone config and encrypt passwords before writing to disk
        let mut to_save = self.clone();
        let device_id = to_save.device_id.clone();
        for peer in &mut to_save.peers {
            if let Some(ref plain) = peer.ssh_password {
                if let Ok(encrypted) = crate::crypto::encrypt(plain, &device_id) {
                    peer.ssh_password = Some(encrypted);
                }
            }
        }

        let content = serde_json::to_string_pretty(&to_save)
            .map_err(|e| P2PError::Config(e.to_string()))?;
        std::fs::write(path, content)?;
        Ok(())
    }
}

pub struct P2PManager {
    pub config: Arc<RwLock<P2PConfig>>,
}

impl P2PManager {
    pub fn new() -> Result<Self, P2PError> {
        let config = P2PConfig::load()?;
        Ok(Self {
            config: Arc::new(RwLock::new(config)),
        })
    }

    pub fn new_with_config(config: P2PConfig) -> Self {
        Self {
            config: Arc::new(RwLock::new(config)),
        }
    }

    pub async fn get_config(&self) -> P2PConfig {
        self.config.read().await.clone()
    }

    pub async fn save_config(&self, config: P2PConfig) -> Result<(), P2PError> {
        config.save()?;
        let mut current = self.config.write().await;
        *current = config;
        Ok(())
    }

    pub async fn add_peer(&self, peer: PeerDevice) -> Result<(), P2PError> {
        let mut config = self.config.write().await;
        if config.peers.iter().any(|p| p.id == peer.id || p.ip_address == peer.ip_address) {
            return Err(P2PError::Config("Peer already exists".to_string()));
        }
        config.peers.push(peer);
        config.save()?;
        Ok(())
    }

    pub async fn remove_peer(&self, peer_id: &str) -> Result<(), P2PError> {
        let mut config = self.config.write().await;
        config.peers.retain(|p| p.id != peer_id);
        config.save()?;
        Ok(())
    }

    pub async fn update_peer(&self, peer: PeerDevice) -> Result<(), P2PError> {
        let mut config = self.config.write().await;
        if let Some(existing) = config.peers.iter_mut().find(|p| p.id == peer.id) {
            *existing = peer;
            config.save()?;
        }
        Ok(())
    }

    pub async fn check_peer_status(&self, peer: &PeerDevice) -> DeviceStatus {
        let addr = format!("{}:{}", peer.ip_address, peer.port);
        
        match TcpStream::connect_timeout(
            &addr.parse().unwrap_or_else(|_| "0.0.0.0:22".parse().unwrap()),
            Duration::from_secs(5),
        ) {
            Ok(_) => DeviceStatus::Online,
            Err(_) => DeviceStatus::Offline,
        }
    }

    pub async fn check_all_peers(&self) -> Vec<(String, DeviceStatus)> {
        let config = self.config.read().await;
        let mut results = Vec::new();
        
        for peer in &config.peers {
            let status = self.check_peer_status(peer).await;
            results.push((peer.id.clone(), status));
        }
        
        results
    }

    pub async fn discover_tailscale_peers(&self) -> Result<Vec<PeerDevice>, P2PError> {
        #[cfg(target_os = "linux")]
        {
            self.discover_tailscale_linux().await
        }
        #[cfg(target_os = "windows")]
        {
            self.discover_tailscale_windows().await
        }
        #[cfg(target_os = "macos")]
        {
            self.discover_tailscale_macos().await
        }
        #[cfg(not(any(target_os = "linux", target_os = "windows", target_os = "macos")))]
        {
            Ok(Vec::new())
        }
    }

    #[cfg(target_os = "linux")]
    async fn discover_tailscale_linux(&self) -> Result<Vec<PeerDevice>, P2PError> {
        tracing::info!("Running tailscale status --json ...");
        let output = tokio::process::Command::new("tailscale")
            .args(["status", "--json"])
            .output()
            .await
            .map_err(|e| {
                tracing::error!("Failed to run tailscale: {}", e);
                P2PError::Connection(e.to_string())
            })?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            tracing::warn!("tailscale status failed: {}", stderr);
            return Ok(Vec::new());
        }

        let json_str = String::from_utf8_lossy(&output.stdout);
        let peers = self.parse_tailscale_status(&json_str).await?;
        tracing::info!("Discovered {} Tailscale peers", peers.len());
        Ok(peers)
    }

    #[cfg(target_os = "windows")]
    async fn discover_tailscale_windows(&self) -> Result<Vec<PeerDevice>, P2PError> {
        let output = tokio::process::Command::new("tailscale")
            .args(["status", "--json"])
            .output()
            .await
            .map_err(|e| P2PError::Connection(e.to_string()))?;

        if !output.status.success() {
            return Ok(Vec::new());
        }

        let json_str = String::from_utf8_lossy(&output.stdout);
        self.parse_tailscale_status(&json_str).await
    }

    #[cfg(target_os = "macos")]
    async fn discover_tailscale_macos(&self) -> Result<Vec<PeerDevice>, P2PError> {
        let output = tokio::process::Command::new("/Applications/Tailscale.app/Contents/MacOS/Tailscale")
            .args(["status", "--json"])
            .output()
            .await
            .map_err(|e| P2PError::Connection(e.to_string()))?;

        if !output.status.success() {
            return Ok(Vec::new());
        }

        let json_str = String::from_utf8_lossy(&output.stdout);
        self.parse_tailscale_status(&json_str).await
    }

    async fn parse_tailscale_status(&self, json_str: &str) -> Result<Vec<PeerDevice>, P2PError> {
        #[derive(Deserialize)]
        #[allow(non_snake_case)]
        struct TailscaleStatus {
            #[serde(default)]
            Peer: Option<std::collections::HashMap<String, TailscalePeer>>,
        }

        #[derive(Deserialize)]
        #[allow(non_snake_case, dead_code)]
        struct TailscalePeer {
            #[serde(default)]
            HostName: Option<String>,
            #[serde(default)]
            DNSName: Option<String>,
            #[serde(default)]
            TailscaleIPs: Option<Vec<String>>,
            #[serde(default)]
            Online: Option<bool>,
            #[serde(default)]
            OS: Option<String>,
        }

        let status: TailscaleStatus = serde_json::from_str(json_str)
            .map_err(|e| P2PError::Config(e.to_string()))?;

        let config = self.config.read().await;
        let known_ips: Vec<String> = config.peers.iter().map(|p| p.ip_address.clone()).collect();
        drop(config);

        let peers: Vec<PeerDevice> = status.Peer
            .unwrap_or_default()
            .into_iter()
            .filter(|(_, p)| p.Online.unwrap_or(false))
            .filter_map(|(id, peer)| {
                let ip = peer.TailscaleIPs.as_ref().and_then(|ips| ips.first()).cloned()?;
                if known_ips.contains(&ip) {
                    return None;
                }

                Some(PeerDevice {
                    id,
                    name: peer.HostName.clone().unwrap_or_default(),
                    hostname: peer.DNSName.clone().unwrap_or_default(),
                    ip_address: ip,
                    port: 22,
                    connection_type: ConnectionType::Tailscale,
                    status: DeviceStatus::Online,
                    last_seen: Some(Utc::now()),
                    ssh_user: String::new(),
                    ssh_key_path: None,
                    ssh_password: None,
                    is_trusted: false,
                    tailscale_hostname: peer.DNSName.clone(),
                    flymode_version: None,
                })
            })
            .collect();

        Ok(peers)
    }
}

pub struct SSHClient {
    session: Option<Session>,
}

impl SSHClient {
    pub fn new() -> Self {
        Self { session: None }
    }

    pub fn connect(&mut self, peer: &PeerDevice) -> Result<(), P2PError> {
        let addr = format!("{}:{}", peer.ip_address, peer.port);
        let tcp = TcpStream::connect(&addr)
            .map_err(|e| P2PError::Connection(e.to_string()))?;

        let mut session = Session::new()
            .map_err(P2PError::Ssh)?;
        
        session.set_tcp_stream(tcp);
        session.handshake()
            .map_err(|e| P2PError::Connection(e.to_string()))?;

        if let Some(key_path) = &peer.ssh_key_path {
            let key_path = PathBuf::from(key_path);
            session.userauth_pubkey_file(
                &peer.ssh_user,
                None,
                &key_path,
                None,
            ).map_err(|e| P2PError::Auth(e.to_string()))?;
        } else if let Some(password) = &peer.ssh_password {
            session.userauth_password(&peer.ssh_user, password)
                .map_err(|e| P2PError::Auth(e.to_string()))?;
        } else {
            let key_path = dirs::home_dir()
                .map(|h| h.join(".ssh/id_rsa"))
                .unwrap_or_default();
            
            if key_path.exists() {
                session.userauth_pubkey_file(
                    &peer.ssh_user,
                    None,
                    &key_path,
                    None,
                ).map_err(|e| P2PError::Auth(e.to_string()))?;
            } else {
                return Err(P2PError::Auth("No SSH key or password configured".to_string()));
            }
        }

        self.session = Some(session);
        Ok(())
    }

    pub fn disconnect(&mut self) {
        if let Some(session) = self.session.take() {
            let _ = session.disconnect(None, "Closing", None);
        }
    }

    pub fn execute_command(&mut self, command: &str) -> Result<String, P2PError> {
        let session = self.session.as_mut()
            .ok_or_else(|| P2PError::Connection("Not connected".to_string()))?;

        let mut channel = session.channel_session()
            .map_err(|e| P2PError::Connection(e.to_string()))?;

        channel.exec(command)
            .map_err(|e| P2PError::Connection(e.to_string()))?;

        let mut output = String::new();
        channel.read_to_string(&mut output)
            .map_err(|e| P2PError::Transfer(e.to_string()))?;

        channel.wait_close()
            .map_err(|e| P2PError::Connection(e.to_string()))?;

        Ok(output)
    }

    pub fn upload_file(&mut self, local_path: &PathBuf, remote_path: &str) -> Result<(), P2PError> {
        self.upload_file_with_progress(local_path, remote_path, |_| {})
    }

    /// Upload a file via SFTP in 64 KB chunks, calling `on_progress` after each chunk
    /// with the cumulative bytes transferred.
    pub fn upload_file_with_progress<F>(
        &mut self,
        local_path: &PathBuf,
        remote_path: &str,
        mut on_progress: F,
    ) -> Result<(), P2PError>
    where
        F: FnMut(u64),
    {
        let session = self.session.as_mut()
            .ok_or_else(|| P2PError::Connection("Not connected".to_string()))?;

        let file_content = std::fs::read(local_path)?;

        let sftp = session.sftp()
            .map_err(|e| P2PError::Transfer(e.to_string()))?;

        let mut remote_file = sftp.create(&PathBuf::from(remote_path))
            .map_err(|e| P2PError::Transfer(e.to_string()))?;

        use std::io::Write;
        const CHUNK_SIZE: usize = 64 * 1024;
        let mut transferred: u64 = 0;

        for chunk in file_content.chunks(CHUNK_SIZE) {
            remote_file.write_all(chunk)
                .map_err(|e| P2PError::Transfer(e.to_string()))?;
            transferred += chunk.len() as u64;
            on_progress(transferred);
        }

        Ok(())
    }

    pub fn download_file(&mut self, remote_path: &str, local_path: &PathBuf) -> Result<(), P2PError> {
        self.download_file_with_progress(remote_path, local_path, |_| {})
    }

    /// Download a file via SFTP in 64 KB chunks, calling `on_progress` after each chunk
    /// with the cumulative bytes read.
    pub fn download_file_with_progress<F>(
        &mut self,
        remote_path: &str,
        local_path: &PathBuf,
        mut on_progress: F,
    ) -> Result<(), P2PError>
    where
        F: FnMut(u64),
    {
        let session = self.session.as_mut()
            .ok_or_else(|| P2PError::Connection("Not connected".to_string()))?;

        let sftp = session.sftp()
            .map_err(|e| P2PError::Transfer(e.to_string()))?;

        let mut remote_file = sftp.open(PathBuf::from(remote_path))
            .map_err(|e| P2PError::Transfer(e.to_string()))?;

        use std::io::Read;
        const CHUNK_SIZE: usize = 64 * 1024;
        let mut content = Vec::new();
        let mut buf = [0u8; CHUNK_SIZE];
        let mut transferred: u64 = 0;

        loop {
            let n = remote_file.read(&mut buf)
                .map_err(|e| P2PError::Transfer(e.to_string()))?;
            if n == 0 {
                break;
            }
            content.extend_from_slice(&buf[..n]);
            transferred += n as u64;
            on_progress(transferred);
        }

        if let Some(parent) = local_path.parent() {
            if !parent.exists() {
                std::fs::create_dir_all(parent)?;
            }
        }

        std::fs::write(local_path, content)?;
        Ok(())
    }

    pub fn list_remote_files(&mut self, remote_dir: &str) -> Result<Vec<RemoteFileInfo>, P2PError> {
        let session = self.session.as_mut()
            .ok_or_else(|| P2PError::Connection("Not connected".to_string()))?;

        let sftp = session.sftp()
            .map_err(|e| P2PError::Transfer(e.to_string()))?;

        let files = sftp.readdir(PathBuf::from(remote_dir))
            .map_err(|e| P2PError::Transfer(e.to_string()))?;

        let file_infos: Vec<RemoteFileInfo> = files
            .into_iter()
            .map(|(path, stat)| RemoteFileInfo {
                name: path.file_name()
                    .map(|n| n.to_string_lossy().to_string())
                    .unwrap_or_default(),
                path: path.to_string_lossy().to_string(),
                is_dir: stat.is_dir(),
                size: stat.size.unwrap_or(0),
                modified: stat.mtime.and_then(|t| {
                    DateTime::from_timestamp(t as i64, 0)
                        .map(|d| d.with_timezone(&Utc))
                }),
            })
            .collect();

        Ok(file_infos)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RemoteFileInfo {
    pub name: String,
    pub path: String,
    pub is_dir: bool,
    pub size: u64,
    pub modified: Option<DateTime<Utc>>,
}

impl Default for SSHClient {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;
    use tempfile::TempDir;

    struct TestFixture {
        _temp_dir: TempDir,
        config_path: PathBuf,
    }

    impl TestFixture {
        fn new() -> Self {
            let temp_dir = tempfile::tempdir().expect("Failed to create temp dir");
            let config_path = temp_dir.path().join("p2p.json");
            Self {
                _temp_dir: temp_dir,
                config_path,
            }
        }

        fn create_config(&self) -> P2PConfig {
            P2PConfig::load_from_path(&self.config_path).expect("Failed to create config")
        }
    }

    fn create_test_peer(name: &str, ip: &str) -> PeerDevice {
        PeerDevice {
            id: uuid::Uuid::new_v4().to_string(),
            name: name.to_string(),
            hostname: format!("{}.local", name.to_lowercase()),
            ip_address: ip.to_string(),
            port: 22,
            connection_type: ConnectionType::LanDirect,
            status: DeviceStatus::Online,
            last_seen: Some(Utc::now()),
            ssh_user: "testuser".to_string(),
            ssh_key_path: None,
            ssh_password: None,
            is_trusted: false,
            tailscale_hostname: None,
            flymode_version: None,
        }
    }

    #[test]
    fn test_peer_device_default() {
        let peer = PeerDevice::default();

        assert!(!peer.id.is_empty());
        assert_eq!(peer.port, 22);
        assert_eq!(peer.connection_type, ConnectionType::LanDirect);
        assert_eq!(peer.status, DeviceStatus::Unknown);
        assert!(!peer.is_trusted);
    }

    #[test]
    fn test_peer_device_serialization() {
        let peer = create_test_peer("TestDevice", "192.168.1.100");
        
        let json = serde_json::to_string(&peer).expect("Failed to serialize");
        let deserialized: PeerDevice = serde_json::from_str(&json).expect("Failed to deserialize");

        assert_eq!(peer.id, deserialized.id);
        assert_eq!(peer.name, deserialized.name);
        assert_eq!(peer.ip_address, deserialized.ip_address);
    }

    #[test]
    fn test_connection_type_serialization() {
        let types = vec![
            ConnectionType::Tailscale,
            ConnectionType::LanDirect,
            ConnectionType::WanDirect,
        ];

        for ct in types {
            let json = serde_json::to_string(&ct).expect("Failed to serialize");
            let deserialized: ConnectionType = serde_json::from_str(&json).expect("Failed to deserialize");
            assert_eq!(ct, deserialized);
        }
    }

    #[test]
    fn test_device_status_serialization() {
        let statuses = vec![
            DeviceStatus::Online,
            DeviceStatus::Offline,
            DeviceStatus::Unknown,
        ];

        for status in statuses {
            let json = serde_json::to_string(&status).expect("Failed to serialize");
            let deserialized: DeviceStatus = serde_json::from_str(&json).expect("Failed to deserialize");
            assert_eq!(status, deserialized);
        }
    }

    #[test]
    fn test_p2p_config_default() {
        let config = P2PConfig::default();

        assert!(!config.device_id.is_empty());
        assert!(!config.device_name.is_empty());
        assert_eq!(config.listen_port, 4827);
        assert!(config.peers.is_empty());
        assert!(config.auto_discover_tailscale);
        assert!(config.sync_enabled);
        assert_eq!(config.sync_interval_seconds, 300);
    }

    #[test]
    fn test_p2p_config_save_and_load() {
        let fixture = TestFixture::new();
        
        let mut config = fixture.create_config();
        config.device_name = "TestDevice".to_string();
        config.sync_interval_seconds = 600;
        config.peers.push(create_test_peer("Peer1", "192.168.1.10"));
        
        config.save_to_path(&fixture.config_path).expect("Failed to save");

        let loaded = P2PConfig::load_from_path(&fixture.config_path).expect("Failed to load");

        assert_eq!(loaded.device_name, "TestDevice");
        assert_eq!(loaded.sync_interval_seconds, 600);
        assert_eq!(loaded.peers.len(), 1);
        assert_eq!(loaded.peers[0].name, "Peer1");
    }

    #[test]
    fn test_p2p_config_creates_file_if_not_exists() {
        let fixture = TestFixture::new();
        
        assert!(!fixture.config_path.exists());

        let config = P2PConfig::load_from_path(&fixture.config_path).expect("Failed to load");

        assert!(fixture.config_path.exists());
        assert!(!config.device_id.is_empty());
    }

    #[test]
    fn test_p2p_config_multiple_peers() {
        let fixture = TestFixture::new();
        
        let mut config = fixture.create_config();
        config.peers.push(create_test_peer("Device1", "192.168.1.10"));
        config.peers.push(create_test_peer("Device2", "192.168.1.11"));
        config.peers.push(create_test_peer("Device3", "192.168.1.12"));
        
        config.save_to_path(&fixture.config_path).expect("Failed to save");

        let loaded = P2PConfig::load_from_path(&fixture.config_path).expect("Failed to load");

        assert_eq!(loaded.peers.len(), 3);
        assert!(loaded.peers.iter().any(|p| p.name == "Device1"));
        assert!(loaded.peers.iter().any(|p| p.name == "Device2"));
        assert!(loaded.peers.iter().any(|p| p.name == "Device3"));
    }

    #[test]
    fn test_peer_trust_flag() {
        let mut peer = create_test_peer("Test", "192.168.1.1");
        
        assert!(!peer.is_trusted);

        peer.is_trusted = true;
        
        assert!(peer.is_trusted);
        
        let json = serde_json::to_string(&peer).expect("Failed to serialize");
        let deserialized: PeerDevice = serde_json::from_str(&json).expect("Failed to deserialize");
        
        assert!(deserialized.is_trusted);
    }

    #[test]
    fn test_remote_file_info() {
        let info = RemoteFileInfo {
            name: "test.txt".to_string(),
            path: "/home/user/test.txt".to_string(),
            is_dir: false,
            size: 1024,
            modified: Some(Utc::now()),
        };

        let json = serde_json::to_string(&info).expect("Failed to serialize");
        let deserialized: RemoteFileInfo = serde_json::from_str(&json).expect("Failed to deserialize");

        assert_eq!(info.name, deserialized.name);
        assert_eq!(info.size, deserialized.size);
        assert_eq!(info.is_dir, deserialized.is_dir);
    }

    #[test]
    fn test_p2p_error_display() {
        let err = P2PError::Connection("test error".to_string());
        assert!(err.to_string().contains("Connection error"));

        let err = P2PError::Auth("auth failed".to_string());
        assert!(err.to_string().contains("Authentication error"));

        let err = P2PError::Config("bad config".to_string());
        assert!(err.to_string().contains("Config error"));
    }

    #[test]
    fn test_ssh_client_new() {
        let client = SSHClient::new();
        assert!(client.session.is_none());
    }

    #[test]
    fn test_peer_with_ssh_key() {
        let mut peer = create_test_peer("KeyAuth", "192.168.1.1");
        peer.ssh_key_path = Some("/home/user/.ssh/id_rsa".to_string());
        peer.ssh_password = None;

        let json = serde_json::to_string(&peer).expect("Failed to serialize");
        let deserialized: PeerDevice = serde_json::from_str(&json).expect("Failed to deserialize");

        assert_eq!(deserialized.ssh_key_path, Some("/home/user/.ssh/id_rsa".to_string()));
        assert_eq!(deserialized.ssh_password, None);
    }

    #[test]
    fn test_peer_with_password() {
        let mut peer = create_test_peer("PassAuth", "192.168.1.1");
        peer.ssh_key_path = None;
        peer.ssh_password = Some("secret123".to_string());

        let json = serde_json::to_string(&peer).expect("Failed to serialize");
        let deserialized: PeerDevice = serde_json::from_str(&json).expect("Failed to deserialize");

        assert_eq!(deserialized.ssh_password, Some("secret123".to_string()));
        assert_eq!(deserialized.ssh_key_path, None);
    }

    #[test]
    fn test_peer_tailscale_connection() {
        let mut peer = create_test_peer("TailscalePeer", "100.64.0.1");
        peer.connection_type = ConnectionType::Tailscale;
        peer.tailscale_hostname = Some("ts-node.tailnet.ts.net".to_string());

        let json = serde_json::to_string(&peer).expect("Failed to serialize");
        let deserialized: PeerDevice = serde_json::from_str(&json).expect("Failed to deserialize");

        assert_eq!(deserialized.connection_type, ConnectionType::Tailscale);
        assert_eq!(deserialized.tailscale_hostname, Some("ts-node.tailnet.ts.net".to_string()));
    }

    #[test]
    fn test_config_sync_settings() {
        let fixture = TestFixture::new();
        
        let mut config = fixture.create_config();
        config.sync_enabled = false;
        config.sync_interval_seconds = 60;

        config.save_to_path(&fixture.config_path).expect("Failed to save");

        let loaded = P2PConfig::load_from_path(&fixture.config_path).expect("Failed to load");

        assert!(!loaded.sync_enabled);
        assert_eq!(loaded.sync_interval_seconds, 60);
    }

    #[test]
    fn test_peer_last_seen() {
        let mut peer = create_test_peer("Test", "192.168.1.1");
        let now = Utc::now();
        peer.last_seen = Some(now);

        let json = serde_json::to_string(&peer).expect("Failed to serialize");
        let deserialized: PeerDevice = serde_json::from_str(&json).expect("Failed to deserialize");

        assert!(deserialized.last_seen.is_some());
    }

    #[test]
    fn test_p2p_config_with_path() {
        let temp_dir = tempfile::tempdir().expect("Failed to create temp dir");
        let custom_path = temp_dir.path().join("custom_p2p.json");
        
        let config = P2PConfig::with_path(custom_path.clone());
        
        assert_eq!(config.config_path, custom_path);
    }

    #[test]
    fn test_peer_different_ports() {
        let mut peer = create_test_peer("Test", "192.168.1.1");
        peer.port = 2222;

        let json = serde_json::to_string(&peer).expect("Failed to serialize");
        let deserialized: PeerDevice = serde_json::from_str(&json).expect("Failed to deserialize");

        assert_eq!(deserialized.port, 2222);
    }
}
