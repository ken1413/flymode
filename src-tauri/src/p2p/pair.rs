use chrono::{DateTime, Utc};
use rand::Rng;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::net::IpAddr;
use std::sync::Arc;
use std::time::Instant;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::RwLock;
use tracing::{error, info, warn};

use tauri::Emitter;

use super::{ConnectionType, DeviceStatus, P2PConfig, P2PError, PeerDevice};

// --- Security constants ---
const MAX_PENDING_REQUESTS: usize = 20;
const RATE_LIMIT_MAX: u32 = 5;
const RATE_LIMIT_WINDOW_SECS: u64 = 60;
const REQUEST_EXPIRY_SECS: i64 = 300; // 5 minutes
const CLEANUP_INTERVAL_SECS: u64 = 60;
const MAX_DEVICE_ID_LEN: usize = 64;
const MAX_DEVICE_NAME_LEN: usize = 128;
const MAX_HOSTNAME_LEN: usize = 256;
const MAX_IP_ADDRESS_LEN: usize = 45; // IPv6 max

/// Information about a device participating in the pairing protocol.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PeerInfo {
    pub device_id: String,
    pub device_name: String,
    pub hostname: String,
    pub ip_address: String,
    pub listen_port: u16,
    pub tailscale_hostname: Option<String>,
    pub flymode_version: Option<String>,
}

/// Result of an initiated pair request.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PairResult {
    pub accepted: bool,
    pub pin: Option<String>,
}

/// Messages exchanged over the pairing TCP connection.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PairMessage {
    Request { from: PeerInfo, pin: String },
    Response { accepted: bool, from: Option<PeerInfo> },
}

/// An incoming pair request waiting for user action.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PairRequest {
    pub id: String,
    pub from: PeerInfo,
    pub pin: String,
    pub received_at: DateTime<Utc>,
}

/// Per-IP rate limit tracking.
struct RateLimitEntry {
    count: u32,
    window_start: Instant,
}

/// Manages the TCP pairing protocol: listening for incoming requests,
/// initiating outbound requests, and handling accept/reject.
pub struct PairServer {
    config: Arc<RwLock<P2PConfig>>,
    pending_requests: Arc<RwLock<Vec<PairRequest>>>,
    /// Holds open TCP streams keyed by request ID so we can send responses.
    held_streams: Arc<RwLock<HashMap<String, TcpStream>>>,
    app_handle: tauri::AppHandle,
    rate_limiter: Arc<RwLock<HashMap<IpAddr, RateLimitEntry>>>,
}

impl PairServer {
    pub fn new(config: Arc<RwLock<P2PConfig>>, app_handle: tauri::AppHandle) -> Self {
        Self {
            config,
            pending_requests: Arc::new(RwLock::new(Vec::new())),
            held_streams: Arc::new(RwLock::new(HashMap::new())),
            app_handle,
            rate_limiter: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Build a `PeerInfo` representing the local machine from current config.
    pub async fn build_peer_info(&self) -> PeerInfo {
        let config = self.config.read().await;
        let hostname = hostname::get()
            .map(|h| h.to_string_lossy().to_string())
            .unwrap_or_default();

        // Try to get our Tailscale IP
        let local_ip = tokio::process::Command::new("tailscale")
            .args(["ip", "-4"])
            .output()
            .await
            .ok()
            .and_then(|o| {
                if o.status.success() {
                    String::from_utf8(o.stdout).ok()
                } else {
                    None
                }
            })
            .map(|s| s.trim().to_string())
            .unwrap_or_default();

        PeerInfo {
            device_id: config.device_id.clone(),
            device_name: config.device_name.clone(),
            hostname,
            ip_address: local_ip,
            listen_port: config.listen_port,
            tailscale_hostname: None,
            flymode_version: Some(env!("CARGO_PKG_VERSION").to_string()),
        }
    }

    /// Start the TCP listener on `0.0.0.0:{listen_port}`.
    pub async fn start_listener(self: Arc<Self>) -> Result<(), P2PError> {
        let port = self.config.read().await.listen_port;
        let addr = format!("0.0.0.0:{}", port);

        let listener = TcpListener::bind(&addr).await.map_err(|e| {
            error!("Failed to bind pair listener on {}: {}", addr, e);
            P2PError::Connection(format!("Failed to bind on {}: {}", addr, e))
        })?;

        info!("Pair listener started on {}", addr);

        // Spawn background cleanup task for expired requests
        let cleanup_self = Arc::clone(&self);
        tokio::spawn(async move {
            cleanup_self.cleanup_loop().await;
        });

        loop {
            match listener.accept().await {
                Ok((stream, peer_addr)) => {
                    info!("Incoming pair connection from {}", peer_addr);
                    let server = Arc::clone(&self);
                    let remote_ip = peer_addr.ip();
                    tokio::spawn(async move {
                        if let Err(e) = server.handle_connection(stream, remote_ip).await {
                            warn!("Error handling pair connection from {}: {}", peer_addr, e);
                        }
                    });
                }
                Err(e) => {
                    warn!("Failed to accept pair connection: {}", e);
                }
            }
        }
    }

    /// Periodically clean up expired pending requests and stale rate-limit entries.
    async fn cleanup_loop(&self) {
        loop {
            tokio::time::sleep(std::time::Duration::from_secs(CLEANUP_INTERVAL_SECS)).await;

            // Clean expired requests
            let mut expired_ids = Vec::new();
            {
                let mut pending = self.pending_requests.write().await;
                let now = Utc::now();
                pending.retain(|r| {
                    let age = (now - r.received_at).num_seconds();
                    if age > REQUEST_EXPIRY_SECS {
                        expired_ids.push(r.id.clone());
                        false
                    } else {
                        true
                    }
                });
            }

            // Drop corresponding held streams
            if !expired_ids.is_empty() {
                let mut streams = self.held_streams.write().await;
                for id in &expired_ids {
                    streams.remove(id);
                }
                info!("Cleaned up {} expired pair requests", expired_ids.len());
            }

            // Clean stale rate-limit entries
            {
                let mut limiter = self.rate_limiter.write().await;
                limiter.retain(|_, entry| entry.window_start.elapsed().as_secs() < RATE_LIMIT_WINDOW_SECS * 2);
            }
        }
    }

    /// Check and update rate limit for an IP. Returns true if allowed.
    async fn check_rate_limit(&self, ip: IpAddr) -> bool {
        let mut limiter = self.rate_limiter.write().await;
        let now = Instant::now();

        let entry = limiter.entry(ip).or_insert(RateLimitEntry {
            count: 0,
            window_start: now,
        });

        if now.duration_since(entry.window_start).as_secs() >= RATE_LIMIT_WINDOW_SECS {
            // Reset window
            entry.count = 1;
            entry.window_start = now;
            true
        } else if entry.count < RATE_LIMIT_MAX {
            entry.count += 1;
            true
        } else {
            false
        }
    }

    /// Validate that peer info fields don't exceed safe lengths.
    fn validate_peer_info(info: &PeerInfo) -> bool {
        info.device_id.len() <= MAX_DEVICE_ID_LEN
            && info.device_name.len() <= MAX_DEVICE_NAME_LEN
            && info.hostname.len() <= MAX_HOSTNAME_LEN
            && info.ip_address.len() <= MAX_IP_ADDRESS_LEN
    }

    /// Handle an incoming TCP connection: read a PairRequest, store it, emit event.
    async fn handle_connection(
        &self,
        mut stream: TcpStream,
        remote_ip: IpAddr,
    ) -> Result<(), P2PError> {
        // Rate limit check
        if !self.check_rate_limit(remote_ip).await {
            warn!("Rate limit exceeded for {}", remote_ip);
            return Ok(());
        }

        // Queue cap check
        {
            let pending = self.pending_requests.read().await;
            if pending.len() >= MAX_PENDING_REQUESTS {
                warn!("Pending requests queue full ({}), dropping connection from {}", MAX_PENDING_REQUESTS, remote_ip);
                return Ok(());
            }
        }

        let msg = read_message(&mut stream).await?;

        match msg {
            PairMessage::Request { mut from, pin } => {
                // Validate field lengths
                if !Self::validate_peer_info(&from) {
                    warn!("Invalid peer info from {} — field too long", remote_ip);
                    return Ok(());
                }

                // Always override IP with actual remote IP (prevent spoofing)
                from.ip_address = remote_ip.to_string();

                let request_id = uuid::Uuid::new_v4().to_string();
                let request = PairRequest {
                    id: request_id.clone(),
                    from: from.clone(),
                    pin,
                    received_at: Utc::now(),
                };

                // Store the request and hold the stream
                {
                    let mut pending = self.pending_requests.write().await;
                    pending.push(request.clone());
                }
                {
                    let mut streams = self.held_streams.write().await;
                    streams.insert(request_id.clone(), stream);
                }

                info!(
                    "Pair request received from {} ({}), request_id={}",
                    from.device_name, from.ip_address, request_id
                );

                // Emit Tauri event so the frontend can show the dialog
                let _ = self
                    .app_handle
                    .emit("pair-request-received", &request);

                Ok(())
            }
            PairMessage::Response { .. } => {
                warn!("Received unexpected Response on listener");
                Ok(())
            }
        }
    }

    /// Initiate a pair request to a remote peer. Returns a `PairResult`
    /// with the generated PIN and whether the remote accepted.
    pub async fn initiate_pair(
        &self,
        target_ip: &str,
        target_port: u16,
    ) -> Result<PairResult, P2PError> {
        let local_info = self.build_peer_info().await;
        let addr = format!("{}:{}", target_ip, target_port);

        // Generate 6-digit PIN
        let pin: u32 = rand::thread_rng().gen_range(100_000..999_999);
        let pin_str = pin.to_string();

        info!("Initiating pair request to {} with PIN", addr);

        let mut stream = TcpStream::connect(&addr).await.map_err(|e| {
            P2PError::Connection(format!("Failed to connect to {}: {}", addr, e))
        })?;

        let request = PairMessage::Request {
            from: local_info,
            pin: pin_str.clone(),
        };
        write_message(&mut stream, &request).await?;

        // Emit PIN to frontend so user can see it while waiting
        let _ = self.app_handle.emit("pair-pin-generated", &pin_str);

        // Wait for response (with 120s timeout — user may take time to decide)
        let response = tokio::time::timeout(
            std::time::Duration::from_secs(120),
            read_message(&mut stream),
        )
        .await
        .map_err(|_| P2PError::Connection("Pair request timed out (120s)".to_string()))??;

        match response {
            PairMessage::Response { accepted, from } => {
                if accepted {
                    if let Some(ref peer_info) = from {
                        info!("Pair request accepted by {}", peer_info.device_name);
                        self.add_peer_from_info(peer_info).await?;
                    }
                } else {
                    info!("Pair request rejected");
                }
                Ok(PairResult {
                    accepted,
                    pin: Some(pin_str),
                })
            }
            PairMessage::Request { .. } => {
                warn!("Received unexpected Request as response");
                Err(P2PError::Connection(
                    "Protocol error: expected Response, got Request".to_string(),
                ))
            }
        }
    }

    /// Accept a pending pair request: verify PIN, add the remote peer,
    /// and send acceptance back.
    pub async fn accept_request(&self, request_id: &str, user_pin: &str) -> Result<(), P2PError> {
        // Peek at the request to validate PIN before consuming it
        {
            let pending = self.pending_requests.read().await;
            let request = pending.iter().find(|r| r.id == request_id).ok_or_else(|| {
                P2PError::Config(format!("Pair request {} not found", request_id))
            })?;

            if request.pin != user_pin {
                return Err(P2PError::Config("PIN does not match".to_string()));
            }
        }

        let request = self.take_request(request_id).await.ok_or_else(|| {
            P2PError::Config(format!("Pair request {} not found", request_id))
        })?;

        let mut stream = self.take_stream(request_id).await.ok_or_else(|| {
            P2PError::Connection(format!(
                "TCP stream for request {} no longer available",
                request_id
            ))
        })?;

        // Add the remote peer (untrusted by default)
        self.add_peer_from_info(&request.from).await?;

        // Send acceptance with our info
        let local_info = self.build_peer_info().await;
        let response = PairMessage::Response {
            accepted: true,
            from: Some(local_info),
        };
        write_message(&mut stream, &response).await?;

        info!(
            "Accepted pair request from {} ({})",
            request.from.device_name, request.from.ip_address
        );

        Ok(())
    }

    /// Reject a pending pair request. Does NOT send local device info.
    pub async fn reject_request(&self, request_id: &str) -> Result<(), P2PError> {
        let request = self.take_request(request_id).await;

        if let Some(mut stream) = self.take_stream(request_id).await {
            // Send rejection without local info (prevent info leak)
            let response = PairMessage::Response {
                accepted: false,
                from: None,
            };
            // Best-effort send — stream may already be closed
            let _ = write_message(&mut stream, &response).await;
        }

        if let Some(req) = &request {
            info!(
                "Rejected pair request from {} ({})",
                req.from.device_name, req.from.ip_address
            );
        }

        Ok(())
    }

    /// Return a snapshot of all pending pair requests.
    pub async fn get_pending_requests(&self) -> Vec<PairRequest> {
        self.pending_requests.read().await.clone()
    }

    /// Remove and return a request from the pending list.
    async fn take_request(&self, request_id: &str) -> Option<PairRequest> {
        let mut pending = self.pending_requests.write().await;
        if let Some(pos) = pending.iter().position(|r| r.id == request_id) {
            Some(pending.remove(pos))
        } else {
            None
        }
    }

    /// Remove and return a held TCP stream.
    async fn take_stream(&self, request_id: &str) -> Option<TcpStream> {
        let mut streams = self.held_streams.write().await;
        streams.remove(request_id)
    }

    /// Convert a `PeerInfo` into a `PeerDevice` and add it to config.
    /// New peers are added as **untrusted** by default. Existing peers
    /// keep their current trust level.
    async fn add_peer_from_info(&self, info: &PeerInfo) -> Result<(), P2PError> {
        let mut config = self.config.write().await;

        // Don't add duplicate — update existing peer instead
        if config.peers.iter().any(|p| {
            p.id == info.device_id || p.ip_address == info.ip_address
        }) {
            if let Some(existing) = config
                .peers
                .iter_mut()
                .find(|p| p.id == info.device_id || p.ip_address == info.ip_address)
            {
                // Keep existing trust level — don't auto-trust
                existing.status = DeviceStatus::Online;
                existing.last_seen = Some(Utc::now());
            }
            config.save()?;
            return Ok(());
        }

        let peer = PeerDevice {
            id: info.device_id.clone(),
            name: info.device_name.clone(),
            hostname: info.hostname.clone(),
            ip_address: info.ip_address.clone(),
            port: 22, // SSH port for sync/transfer
            connection_type: ConnectionType::Tailscale,
            status: DeviceStatus::Online,
            last_seen: Some(Utc::now()),
            ssh_user: String::new(), // User can fill in later via edit
            ssh_key_path: None,
            ssh_password: None,
            is_trusted: false, // Untrusted by default — user must manually trust
            tailscale_hostname: info.tailscale_hostname.clone(),
            flymode_version: info.flymode_version.clone(),
        };

        config.peers.push(peer);
        config.save()?;

        Ok(())
    }
}

// --- Wire format: u32 big-endian length prefix + JSON bytes ---

/// Write a length-prefixed JSON message to a TCP stream.
pub async fn write_message(stream: &mut TcpStream, msg: &PairMessage) -> Result<(), P2PError> {
    let json = serde_json::to_vec(msg).map_err(|e| P2PError::Config(e.to_string()))?;
    let len = json.len() as u32;
    stream
        .write_all(&len.to_be_bytes())
        .await
        .map_err(|e| P2PError::Connection(format!("Failed to write length: {}", e)))?;
    stream
        .write_all(&json)
        .await
        .map_err(|e| P2PError::Connection(format!("Failed to write message: {}", e)))?;
    stream
        .flush()
        .await
        .map_err(|e| P2PError::Connection(format!("Failed to flush: {}", e)))?;
    Ok(())
}

/// Read a length-prefixed JSON message from a TCP stream.
pub async fn read_message(stream: &mut TcpStream) -> Result<PairMessage, P2PError> {
    let mut len_buf = [0u8; 4];
    stream
        .read_exact(&mut len_buf)
        .await
        .map_err(|e| P2PError::Connection(format!("Failed to read length: {}", e)))?;
    let len = u32::from_be_bytes(len_buf) as usize;

    // Sanity check: reject messages > 1 MB
    if len > 1_048_576 {
        return Err(P2PError::Connection(format!(
            "Message too large: {} bytes",
            len
        )));
    }

    let mut buf = vec![0u8; len];
    stream
        .read_exact(&mut buf)
        .await
        .map_err(|e| P2PError::Connection(format!("Failed to read message: {}", e)))?;

    let msg: PairMessage =
        serde_json::from_slice(&buf).map_err(|e| P2PError::Config(e.to_string()))?;
    Ok(msg)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::p2p::DEFAULT_LISTEN_PORT;

    fn sample_peer_info() -> PeerInfo {
        PeerInfo {
            device_id: "test-device-001".to_string(),
            device_name: "TestDevice".to_string(),
            hostname: "testdevice.local".to_string(),
            ip_address: "100.64.0.1".to_string(),
            listen_port: DEFAULT_LISTEN_PORT,
            tailscale_hostname: Some("testdevice.ts.net".to_string()),
            flymode_version: Some("0.3.0".to_string()),
        }
    }

    #[test]
    fn test_peer_info_serialization() {
        let info = sample_peer_info();
        let json = serde_json::to_string(&info).expect("serialize");
        let deserialized: PeerInfo = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(info, deserialized);
    }

    #[test]
    fn test_pair_message_request_roundtrip() {
        let msg = PairMessage::Request {
            from: sample_peer_info(),
            pin: "123456".to_string(),
        };
        let json = serde_json::to_vec(&msg).expect("serialize");
        let deserialized: PairMessage = serde_json::from_slice(&json).expect("deserialize");
        match deserialized {
            PairMessage::Request { from, pin } => {
                assert_eq!(from.device_id, "test-device-001");
                assert_eq!(from.device_name, "TestDevice");
                assert_eq!(pin, "123456");
            }
            _ => panic!("Expected Request"),
        }
    }

    #[test]
    fn test_pair_message_response_roundtrip() {
        let msg = PairMessage::Response {
            accepted: true,
            from: Some(sample_peer_info()),
        };
        let json = serde_json::to_vec(&msg).expect("serialize");
        let deserialized: PairMessage = serde_json::from_slice(&json).expect("deserialize");
        match deserialized {
            PairMessage::Response { accepted, from } => {
                assert!(accepted);
                assert_eq!(from.unwrap().device_name, "TestDevice");
            }
            _ => panic!("Expected Response"),
        }
    }

    #[test]
    fn test_pair_message_response_rejected_no_info() {
        let msg = PairMessage::Response {
            accepted: false,
            from: None,
        };
        let json = serde_json::to_vec(&msg).expect("serialize");
        let deserialized: PairMessage = serde_json::from_slice(&json).expect("deserialize");
        match deserialized {
            PairMessage::Response { accepted, from } => {
                assert!(!accepted);
                assert!(from.is_none());
            }
            _ => panic!("Expected Response"),
        }
    }

    #[test]
    fn test_pair_request_serialization() {
        let request = PairRequest {
            id: "req-001".to_string(),
            from: sample_peer_info(),
            pin: "654321".to_string(),
            received_at: Utc::now(),
        };
        let json = serde_json::to_string(&request).expect("serialize");
        let deserialized: PairRequest = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(deserialized.id, "req-001");
        assert_eq!(deserialized.from.device_name, "TestDevice");
        assert_eq!(deserialized.pin, "654321");
    }

    #[test]
    fn test_pair_result_serialization() {
        let result = PairResult {
            accepted: true,
            pin: Some("123456".to_string()),
        };
        let json = serde_json::to_string(&result).expect("serialize");
        let deserialized: PairResult = serde_json::from_str(&json).expect("deserialize");
        assert!(deserialized.accepted);
        assert_eq!(deserialized.pin, Some("123456".to_string()));
    }

    #[test]
    fn test_validate_peer_info_valid() {
        let info = sample_peer_info();
        assert!(PairServer::validate_peer_info(&info));
    }

    #[test]
    fn test_validate_peer_info_device_id_too_long() {
        let mut info = sample_peer_info();
        info.device_id = "x".repeat(MAX_DEVICE_ID_LEN + 1);
        assert!(!PairServer::validate_peer_info(&info));
    }

    #[test]
    fn test_validate_peer_info_device_name_too_long() {
        let mut info = sample_peer_info();
        info.device_name = "x".repeat(MAX_DEVICE_NAME_LEN + 1);
        assert!(!PairServer::validate_peer_info(&info));
    }

    #[test]
    fn test_validate_peer_info_hostname_too_long() {
        let mut info = sample_peer_info();
        info.hostname = "x".repeat(MAX_HOSTNAME_LEN + 1);
        assert!(!PairServer::validate_peer_info(&info));
    }

    #[test]
    fn test_validate_peer_info_ip_too_long() {
        let mut info = sample_peer_info();
        info.ip_address = "x".repeat(MAX_IP_ADDRESS_LEN + 1);
        assert!(!PairServer::validate_peer_info(&info));
    }

    #[test]
    fn test_pin_generation_range() {
        // Verify PIN is always 6 digits
        for _ in 0..100 {
            let pin: u32 = rand::thread_rng().gen_range(100_000..999_999);
            assert!(pin >= 100_000 && pin < 999_999);
            assert_eq!(pin.to_string().len(), 6);
        }
    }

    #[tokio::test]
    async fn test_wire_format_roundtrip() {
        let listener = TcpListener::bind("127.0.0.1:0").await.expect("bind");
        let addr = listener.local_addr().expect("local_addr");

        let msg = PairMessage::Request {
            from: sample_peer_info(),
            pin: "999888".to_string(),
        };
        let msg_clone = msg.clone();

        let writer = tokio::spawn(async move {
            let mut stream = TcpStream::connect(addr).await.expect("connect");
            write_message(&mut stream, &msg_clone).await.expect("write");
        });

        let (mut stream, _) = listener.accept().await.expect("accept");
        let received = read_message(&mut stream).await.expect("read");

        writer.await.expect("writer task");

        match received {
            PairMessage::Request { from, pin } => {
                assert_eq!(from.device_id, "test-device-001");
                assert_eq!(pin, "999888");
            }
            _ => panic!("Expected Request"),
        }
    }

    #[tokio::test]
    async fn test_wire_format_response_roundtrip() {
        let listener = TcpListener::bind("127.0.0.1:0").await.expect("bind");
        let addr = listener.local_addr().expect("local_addr");

        let msg = PairMessage::Response {
            accepted: true,
            from: Some(sample_peer_info()),
        };
        let msg_clone = msg.clone();

        let writer = tokio::spawn(async move {
            let mut stream = TcpStream::connect(addr).await.expect("connect");
            write_message(&mut stream, &msg_clone).await.expect("write");
        });

        let (mut stream, _) = listener.accept().await.expect("accept");
        let received = read_message(&mut stream).await.expect("read");

        writer.await.expect("writer task");

        match received {
            PairMessage::Response { accepted, from } => {
                assert!(accepted);
                assert_eq!(from.unwrap().device_name, "TestDevice");
            }
            _ => panic!("Expected Response"),
        }
    }

    #[test]
    fn test_peer_info_with_empty_optionals() {
        let info = PeerInfo {
            device_id: "id".to_string(),
            device_name: "name".to_string(),
            hostname: "host".to_string(),
            ip_address: "1.2.3.4".to_string(),
            listen_port: DEFAULT_LISTEN_PORT,
            tailscale_hostname: None,
            flymode_version: None,
        };
        let json = serde_json::to_string(&info).expect("serialize");
        let deserialized: PeerInfo = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(deserialized.tailscale_hostname, None);
        assert_eq!(deserialized.flymode_version, None);
    }

    #[test]
    fn test_wire_format_length_prefix() {
        let msg = PairMessage::Request {
            from: sample_peer_info(),
            pin: "112233".to_string(),
        };
        let json = serde_json::to_vec(&msg).expect("serialize");
        let len = json.len() as u32;

        let mut wire = Vec::new();
        wire.extend_from_slice(&len.to_be_bytes());
        wire.extend_from_slice(&json);

        // Verify: first 4 bytes are the length
        let read_len = u32::from_be_bytes([wire[0], wire[1], wire[2], wire[3]]);
        assert_eq!(read_len as usize, json.len());

        // Verify: remaining bytes are valid JSON
        let parsed: PairMessage =
            serde_json::from_slice(&wire[4..]).expect("parse from wire");
        match parsed {
            PairMessage::Request { from, pin } => {
                assert_eq!(from.device_id, "test-device-001");
                assert_eq!(pin, "112233");
            }
            _ => panic!("Expected Request"),
        }
    }
}
