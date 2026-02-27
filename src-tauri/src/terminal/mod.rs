use crate::p2p::{PeerDevice, SSHClient};
use crossbeam_channel::{Receiver, Sender};
use std::collections::HashMap;
use std::io::{Read, Write};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use tauri::ipc::Channel;
use thiserror::Error;
use tokio::sync::RwLock;
use tokio::task::JoinHandle;
use tracing::{error, info, warn};

#[derive(Error, Debug)]
pub enum TerminalError {
    #[error("SSH error: {0}")]
    Ssh(String),
    #[error("Not found: {0}")]
    NotFound(String),
    #[error("Session already closed")]
    AlreadyClosed,
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}

pub enum TerminalInput {
    Data(Vec<u8>),
    Resize { cols: u32, rows: u32 },
    Close,
}

struct TerminalSession {
    _session_id: String,
    _peer_id: String,
    input_tx: Sender<TerminalInput>,
    shutdown: Arc<AtomicBool>,
    reader_handle: Option<JoinHandle<()>>,
}

pub struct TerminalManager {
    sessions: Arc<RwLock<HashMap<String, TerminalSession>>>,
}

impl TerminalManager {
    pub fn new() -> Self {
        Self {
            sessions: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub async fn open_session(
        &self,
        peer: PeerDevice,
        cols: u32,
        rows: u32,
        output_channel: Channel<Vec<u8>>,
    ) -> Result<String, TerminalError> {
        let session_id = uuid::Uuid::new_v4().to_string();
        let peer_id = peer.id.clone();
        let shutdown = Arc::new(AtomicBool::new(false));
        let (input_tx, input_rx) = crossbeam_channel::unbounded::<TerminalInput>();

        let sid = session_id.clone();
        let shutdown_clone = shutdown.clone();

        let handle = tokio::task::spawn_blocking(move || {
            if let Err(e) = run_pty_loop(peer, cols, rows, input_rx, &output_channel, shutdown_clone)
            {
                error!("Terminal session {} ended with error: {}", sid, e);
                // Send error message to frontend terminal
                let msg = format!("\r\nConnection error: {}\r\n", e);
                let _ = output_channel.send(msg.into_bytes());
            } else {
                info!("Terminal session {} ended normally", sid);
            }
        });

        let session = TerminalSession {
            _session_id: session_id.clone(),
            _peer_id: peer_id,
            input_tx,
            shutdown,
            reader_handle: Some(handle),
        };

        let mut sessions = self.sessions.write().await;
        sessions.insert(session_id.clone(), session);

        Ok(session_id)
    }

    pub async fn send_input(
        &self,
        session_id: &str,
        data: Vec<u8>,
    ) -> Result<(), TerminalError> {
        let sessions = self.sessions.read().await;
        let session = sessions
            .get(session_id)
            .ok_or_else(|| TerminalError::NotFound(session_id.to_string()))?;
        session
            .input_tx
            .send(TerminalInput::Data(data))
            .map_err(|_| TerminalError::AlreadyClosed)
    }

    pub async fn resize(
        &self,
        session_id: &str,
        cols: u32,
        rows: u32,
    ) -> Result<(), TerminalError> {
        let sessions = self.sessions.read().await;
        let session = sessions
            .get(session_id)
            .ok_or_else(|| TerminalError::NotFound(session_id.to_string()))?;
        session
            .input_tx
            .send(TerminalInput::Resize { cols, rows })
            .map_err(|_| TerminalError::AlreadyClosed)
    }

    pub async fn close_session(&self, session_id: &str) -> Result<(), TerminalError> {
        let mut sessions = self.sessions.write().await;
        if let Some(mut session) = sessions.remove(session_id) {
            session.shutdown.store(true, Ordering::Relaxed);
            let _ = session.input_tx.send(TerminalInput::Close);
            if let Some(handle) = session.reader_handle.take() {
                drop(sessions); // release lock before awaiting
                let _ = handle.await;
            }
            Ok(())
        } else {
            Err(TerminalError::NotFound(session_id.to_string()))
        }
    }
}

/// Runs on a blocking thread. Handles SSH PTY I/O in a single loop.
fn run_pty_loop(
    peer: PeerDevice,
    cols: u32,
    rows: u32,
    input_rx: Receiver<TerminalInput>,
    output_channel: &Channel<Vec<u8>>,
    shutdown: Arc<AtomicBool>,
) -> Result<(), TerminalError> {
    // Connect via SSH (with 10s timeout)
    let addr = format!("{}:{}", peer.ip_address, peer.port);
    let _ = output_channel.send(format!("Connecting to {}...\r\n", addr).into_bytes());

    let mut ssh = SSHClient::new();
    ssh.connect(&peer).map_err(|e| TerminalError::Ssh(e.to_string()))?;

    let _ = output_channel.send(b"SSH connected. Locating openclaw...\r\n".to_vec());

    let session = ssh
        .session
        .as_mut()
        .ok_or_else(|| TerminalError::Ssh("No SSH session".to_string()))?;

    // Find openclaw binary path on remote machine
    let openclaw_path = {
        let mut find_ch = session
            .channel_session()
            .map_err(|e: ssh2::Error| TerminalError::Ssh(e.to_string()))?;
        find_ch
            .exec("which openclaw 2>/dev/null || command -v openclaw 2>/dev/null || find /usr/local/bin /usr/bin /home -maxdepth 4 -name openclaw -type f 2>/dev/null | head -1")
            .map_err(|e: ssh2::Error| TerminalError::Ssh(e.to_string()))?;
        let mut output = String::new();
        find_ch.read_to_string(&mut output)
            .map_err(|e| TerminalError::Io(e))?;
        find_ch.wait_close()
            .map_err(|e: ssh2::Error| TerminalError::Ssh(e.to_string()))?;
        let path = output.trim().lines().next().unwrap_or("").to_string();
        if path.is_empty() {
            return Err(TerminalError::NotFound("openclaw not found on remote machine".to_string()));
        }
        path
    };

    let _ = output_channel.send(format!("Found: {}. Starting TUI...\r\n", openclaw_path).into_bytes());

    // Setup PTY with full path
    let mut channel = session
        .channel_session()
        .map_err(|e: ssh2::Error| TerminalError::Ssh(e.to_string()))?;

    channel
        .request_pty("xterm-256color", None, Some((cols, rows, 0, 0)))
        .map_err(|e: ssh2::Error| TerminalError::Ssh(e.to_string()))?;

    let exec_cmd = format!("{} tui", openclaw_path);
    channel
        .exec(&exec_cmd)
        .map_err(|e: ssh2::Error| TerminalError::Ssh(e.to_string()))?;

    // Switch to non-blocking AFTER setup is complete
    session.set_blocking(false);

    let mut buf = [0u8; 4096];

    loop {
        if shutdown.load(Ordering::Relaxed) {
            break;
        }

        // Check for user input (non-blocking)
        match input_rx.try_recv() {
            Ok(TerminalInput::Data(data)) => {
                // Temporarily set blocking for write
                session.set_blocking(true);
                if let Err(e) = channel.write_all(&data) {
                    warn!("Failed to write to PTY: {}", e);
                    break;
                }
                let _ = channel.flush();
                session.set_blocking(false);
            }
            Ok(TerminalInput::Resize { cols, rows }) => {
                // libssh2 pty_size change
                session.set_blocking(true);
                if let Err(e) =
                    channel.request_pty_size(cols, rows, None, None)
                {
                    warn!("Failed to resize PTY: {}", e);
                }
                session.set_blocking(false);
            }
            Ok(TerminalInput::Close) => {
                break;
            }
            Err(crossbeam_channel::TryRecvError::Empty) => {}
            Err(crossbeam_channel::TryRecvError::Disconnected) => {
                break;
            }
        }

        // Read SSH output (non-blocking)
        let read_result: std::io::Result<usize> = channel.read(&mut buf);
        match read_result {
            Ok(0) => {
                // EOF
                break;
            }
            Ok(n) => {
                let data = buf[..n].to_vec();
                if output_channel.send(data).is_err() {
                    warn!("Frontend channel closed");
                    break;
                }
            }
            Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                // No data available — sleep briefly to avoid busy-spin
                std::thread::sleep(std::time::Duration::from_millis(10));
            }
            Err(e) => {
                warn!("SSH read error: {}", e);
                break;
            }
        }

        // Check if the remote channel has closed
        if channel.eof() {
            break;
        }
    }

    // Cleanup
    let _ = channel.send_eof();
    let _ = channel.wait_close();
    ssh.disconnect();

    Ok(())
}

/// Check if openclaw-gateway is running on a remote peer.
pub fn check_openclaw_running(peer: &PeerDevice) -> Result<bool, TerminalError> {
    let mut ssh = SSHClient::new();
    ssh.connect(peer)
        .map_err(|e| TerminalError::Ssh(e.to_string()))?;
    let output = ssh
        .execute_command("pgrep -f openclaw-gateway")
        .map_err(|e| TerminalError::Ssh(e.to_string()))?;
    ssh.disconnect();
    Ok(!output.trim().is_empty())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_terminal_error_display() {
        let err = TerminalError::Ssh("connection refused".to_string());
        assert!(err.to_string().contains("SSH error"));

        let err = TerminalError::NotFound("abc123".to_string());
        assert!(err.to_string().contains("Not found"));

        let err = TerminalError::AlreadyClosed;
        assert!(err.to_string().contains("already closed"));
    }

    #[test]
    fn test_terminal_manager_new() {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async {
            let manager = TerminalManager::new();
            let sessions = manager.sessions.read().await;
            assert!(sessions.is_empty());
        });
    }

    #[test]
    fn test_terminal_input_variants() {
        let _data = TerminalInput::Data(vec![0x1b, 0x5b, 0x41]); // ESC [ A (arrow up)
        let _resize = TerminalInput::Resize {
            cols: 80,
            rows: 24,
        };
        let _close = TerminalInput::Close;
    }

    #[tokio::test]
    async fn test_send_input_not_found() {
        let manager = TerminalManager::new();
        let result = manager.send_input("nonexistent", vec![0x41]).await;
        assert!(result.is_err());
        match result.unwrap_err() {
            TerminalError::NotFound(id) => assert_eq!(id, "nonexistent"),
            other => panic!("Expected NotFound, got: {}", other),
        }
    }

    #[tokio::test]
    async fn test_resize_not_found() {
        let manager = TerminalManager::new();
        let result = manager.resize("nonexistent", 120, 40).await;
        assert!(result.is_err());
        match result.unwrap_err() {
            TerminalError::NotFound(id) => assert_eq!(id, "nonexistent"),
            other => panic!("Expected NotFound, got: {}", other),
        }
    }

    #[tokio::test]
    async fn test_close_session_not_found() {
        let manager = TerminalManager::new();
        let result = manager.close_session("nonexistent").await;
        assert!(result.is_err());
        match result.unwrap_err() {
            TerminalError::NotFound(id) => assert_eq!(id, "nonexistent"),
            other => panic!("Expected NotFound, got: {}", other),
        }
    }
}
