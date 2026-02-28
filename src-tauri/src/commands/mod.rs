use crate::config::{ActionType, AppConfig, ScheduleRule, TargetType};
use crate::notes::{Note, NoteColor, NotesStore};
use crate::p2p::pair::{PairRequest, PairResult, PairServer};
use crate::p2p::{DeviceStatus, P2PConfig, P2PManager, PeerDevice};
use crate::scheduler::{execute_custom_command, execute_command, get_airplane_command, get_bluetooth_command, get_wifi_command};
use crate::sync::{SyncEngine, SyncResult, SyncState};
use crate::terminal::TerminalManager;
use crate::transfer::{TransferManager, TransferProgress, TransferQueue};
use crate::wireless::{get_wireless_status, WirelessStatus};
use std::path::PathBuf;
use std::sync::Arc;
use tauri::ipc::Channel;
use tauri::State;
use tokio::sync::RwLock;

pub type ConfigState = Arc<RwLock<AppConfig>>;
pub type NotesState = Arc<NotesStore>;
pub type P2PState = Arc<P2PManager>;
pub type PairState = Arc<PairServer>;
pub type SyncStateType = Arc<SyncEngine>;
pub type TransferState = Arc<TransferManager>;
pub type TerminalState = Arc<TerminalManager>;

#[tauri::command]
pub async fn get_config(state: State<'_, ConfigState>) -> Result<AppConfig, String> {
    let config = state.read().await.clone();
    Ok(config)
}

#[tauri::command]
pub async fn save_config(state: State<'_, ConfigState>, config: AppConfig) -> Result<(), String> {
    let mut current = state.write().await;
    *current = config.clone();
    current.save().map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub async fn add_rule(state: State<'_, ConfigState>, mut rule: ScheduleRule) -> Result<(), String> {
    let mut config = state.write().await;
    rule.id = uuid::Uuid::new_v4().to_string();
    config.rules.push(rule);
    config.save().map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub async fn update_rule(state: State<'_, ConfigState>, rule: ScheduleRule) -> Result<(), String> {
    let mut config = state.write().await;
    if let Some(pos) = config.rules.iter().position(|r| r.id == rule.id) {
        config.rules[pos] = rule;
        config.save().map_err(|e| e.to_string())?;
        Ok(())
    } else {
        Err("Rule not found".to_string())
    }
}

#[tauri::command]
pub async fn delete_rule(state: State<'_, ConfigState>, rule_id: String) -> Result<(), String> {
    let mut config = state.write().await;
    config.rules.retain(|r| r.id != rule_id);
    config.save().map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub async fn toggle_rule(state: State<'_, ConfigState>, rule_id: String) -> Result<(), String> {
    let mut config = state.write().await;
    if let Some(rule) = config.rules.iter_mut().find(|r| r.id == rule_id) {
        rule.enabled = !rule.enabled;
        config.save().map_err(|e| e.to_string())?;
        Ok(())
    } else {
        Err("Rule not found".to_string())
    }
}

#[tauri::command]
pub async fn execute_rule_now(rule: ScheduleRule) -> Result<String, String> {
    match &rule.target {
        TargetType::CustomCommand => {
            if let Some(cmd) = &rule.command {
                execute_custom_command(cmd).await
            } else {
                Err("No command specified".to_string())
            }
        }
        TargetType::Wifi => {
            let cmd = get_wifi_command(&rule.action).map_err(|e| e.to_string())?;
            execute_command(&cmd).await
        }
        TargetType::Bluetooth => {
            let cmd = get_bluetooth_command(&rule.action).map_err(|e| e.to_string())?;
            execute_command(&cmd).await
        }
        TargetType::AirplaneMode => {
            let cmd = get_airplane_command(&rule.action).map_err(|e| e.to_string())?;
            execute_command(&cmd).await
        }
    }
}

#[tauri::command]
pub fn get_status() -> WirelessStatus {
    get_wireless_status()
}

#[tauri::command]
pub async fn toggle_wifi(enable: bool) -> Result<String, String> {
    let action = if enable { ActionType::Enable } else { ActionType::Disable };
    let cmd = get_wifi_command(&action).map_err(|e| e.to_string())?;
    execute_command(&cmd).await
}

#[tauri::command]
pub async fn toggle_bluetooth(enable: bool) -> Result<String, String> {
    let action = if enable { ActionType::Enable } else { ActionType::Disable };
    let cmd = get_bluetooth_command(&action).map_err(|e| e.to_string())?;
    execute_command(&cmd).await
}

#[tauri::command]
pub async fn toggle_airplane_mode(enable: bool) -> Result<String, String> {
    let action = if enable { ActionType::Enable } else { ActionType::Disable };
    let cmd = get_airplane_command(&action).map_err(|e| e.to_string())?;
    execute_command(&cmd).await
}

#[tauri::command]
pub async fn run_custom_command(command: String) -> Result<String, String> {
    execute_custom_command(&command).await
}

// Notes commands
#[tauri::command]
pub fn create_note(state: State<'_, NotesState>, title: String, content: String) -> Result<Note, String> {
    state.create(title, content).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn update_note(state: State<'_, NotesState>, note: Note) -> Result<(), String> {
    state.update(&note).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn delete_note(state: State<'_, NotesState>, id: String) -> Result<(), String> {
    state.delete(&id).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn get_note(state: State<'_, NotesState>, id: String) -> Result<Option<Note>, String> {
    state.get(&id).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn list_notes(state: State<'_, NotesState>, include_archived: bool) -> Result<Vec<Note>, String> {
    state.list(include_archived).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn search_notes(state: State<'_, NotesState>, query: String) -> Result<Vec<Note>, String> {
    state.search(&query).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn get_note_colors() -> Vec<(String, String)> {
    vec![
        ("Yellow".to_string(), NoteColor::Yellow.hex().to_string()),
        ("Pink".to_string(), NoteColor::Pink.hex().to_string()),
        ("Blue".to_string(), NoteColor::Blue.hex().to_string()),
        ("Green".to_string(), NoteColor::Green.hex().to_string()),
        ("Purple".to_string(), NoteColor::Purple.hex().to_string()),
        ("Orange".to_string(), NoteColor::Orange.hex().to_string()),
        ("White".to_string(), NoteColor::White.hex().to_string()),
        ("Gray".to_string(), NoteColor::Gray.hex().to_string()),
    ]
}

#[tauri::command]
pub fn get_note_categories() -> Vec<String> {
    vec![
        "General".to_string(),
        "Work".to_string(),
        "Personal".to_string(),
        "Ideas".to_string(),
        "Tasks".to_string(),
        "Important".to_string(),
        "Archive".to_string(),
    ]
}

// P2P commands
#[tauri::command]
pub async fn get_p2p_config(state: State<'_, P2PState>) -> Result<P2PConfig, String> {
    Ok(state.get_config().await)
}

#[tauri::command]
pub async fn save_p2p_config(state: State<'_, P2PState>, config: P2PConfig) -> Result<(), String> {
    state.save_config(config).await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn add_peer(state: State<'_, P2PState>, peer: PeerDevice) -> Result<(), String> {
    state.add_peer(peer).await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn remove_peer(state: State<'_, P2PState>, peer_id: String) -> Result<(), String> {
    state.remove_peer(&peer_id).await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn update_peer(state: State<'_, P2PState>, peer: PeerDevice) -> Result<(), String> {
    state.update_peer(peer).await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn check_peer_status(state: State<'_, P2PState>, peer: PeerDevice) -> Result<DeviceStatus, String> {
    Ok(state.check_peer_status(&peer).await)
}

#[tauri::command]
pub async fn check_all_peers(state: State<'_, P2PState>) -> Result<Vec<(String, DeviceStatus)>, String> {
    Ok(state.check_all_peers().await)
}

#[tauri::command]
pub async fn discover_tailscale(state: State<'_, P2PState>) -> Result<Vec<PeerDevice>, String> {
    state.discover_tailscale_peers().await.map_err(|e| e.to_string())
}

// Sync commands
#[tauri::command]
pub async fn get_sync_state(state: State<'_, SyncStateType>) -> Result<SyncState, String> {
    Ok(state.get_state().await)
}

#[tauri::command]
pub async fn sync_with_peer(state: State<'_, SyncStateType>, peer: PeerDevice) -> Result<SyncResult, String> {
    state.sync_with_peer(&peer).await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn sync_all_peers(state: State<'_, SyncStateType>) -> Result<Vec<SyncResult>, String> {
    Ok(state.sync_all_peers().await)
}

#[tauri::command]
pub async fn export_notes(state: State<'_, SyncStateType>) -> Result<String, String> {
    state.export_notes().await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn import_notes(state: State<'_, SyncStateType>, json: String) -> Result<usize, String> {
    state.import_notes(&json).await.map_err(|e| e.to_string())
}

#[tauri::command]
pub fn get_sync_folder(state: State<'_, SyncStateType>) -> String {
    state.get_sync_folder().to_string_lossy().to_string()
}

// Transfer commands
#[tauri::command]
pub async fn get_transfer_queue(state: State<'_, TransferState>) -> Result<TransferQueue, String> {
    Ok(state.get_queue().await)
}

#[tauri::command]
pub async fn upload_file(
    state: State<'_, TransferState>,
    peer: PeerDevice,
    local_path: String,
    remote_path: String,
) -> Result<String, String> {
    state
        .upload_file(&peer, PathBuf::from(local_path), remote_path)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn download_file(
    state: State<'_, TransferState>,
    peer: PeerDevice,
    remote_path: String,
    local_path: String,
) -> Result<String, String> {
    state
        .download_file(&peer, remote_path, PathBuf::from(local_path))
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn cancel_transfer(state: State<'_, TransferState>, transfer_id: String) -> Result<(), String> {
    state.cancel_transfer(&transfer_id).await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn clear_completed_transfers(state: State<'_, TransferState>) -> Result<(), String> {
    state.clear_completed().await;
    Ok(())
}

#[tauri::command]
pub async fn get_transfer_progress(state: State<'_, TransferState>, transfer_id: String) -> Result<Option<TransferProgress>, String> {
    Ok(state.get_transfer(&transfer_id).await)
}

#[tauri::command]
pub async fn browse_remote_files(
    state: State<'_, TransferState>,
    peer: PeerDevice,
    path: String,
) -> Result<Vec<crate::p2p::RemoteFileInfo>, String> {
    state.browse_remote(&peer, &path).await.map_err(|e| e.to_string())
}

// Pair commands
#[tauri::command]
pub async fn pair_with_peer(state: State<'_, PairState>, ip: String, port: u16) -> Result<PairResult, String> {
    state.initiate_pair(&ip, port).await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_pending_pair_requests(state: State<'_, PairState>) -> Result<Vec<PairRequest>, String> {
    Ok(state.get_pending_requests().await)
}

#[tauri::command]
pub async fn accept_pair_request(state: State<'_, PairState>, request_id: String, pin: String) -> Result<(), String> {
    state.accept_request(&request_id, &pin).await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn reject_pair_request(state: State<'_, PairState>, request_id: String) -> Result<(), String> {
    state.reject_request(&request_id).await.map_err(|e| e.to_string())
}

// Build info
#[tauri::command]
pub fn get_build_info() -> std::collections::HashMap<String, String> {
    let mut info = std::collections::HashMap::new();
    info.insert("version".to_string(), env!("CARGO_PKG_VERSION").to_string());
    info.insert("git_hash".to_string(), env!("GIT_HASH").to_string());
    info
}

// Authentication
#[tauri::command]
pub fn verify_system_password(password: String) -> Result<bool, String> {
    use std::io::Write;
    use std::process::{Command, Stdio};

    let username = std::env::var("USER")
        .or_else(|_| std::env::var("LOGNAME"))
        .map_err(|_| "Cannot determine username".to_string())?;

    // Use unix_chkpwd (PAM helper, setuid root, available on all PAM-enabled Linux)
    let chkpwd = ["/usr/sbin/unix_chkpwd", "/sbin/unix_chkpwd"]
        .iter()
        .find(|p| std::path::Path::new(p).exists())
        .ok_or_else(|| "unix_chkpwd not found".to_string())?;

    let mut child = Command::new(chkpwd)
        .arg(&username)
        .arg("nullok")
        .stdin(Stdio::piped())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .map_err(|e| format!("Failed to spawn unix_chkpwd: {}", e))?;

    if let Some(ref mut stdin) = child.stdin {
        let _ = stdin.write_all(format!("{}\0", password).as_bytes());
    }
    drop(child.stdin.take());

    let status = child.wait().map_err(|e| format!("unix_chkpwd wait failed: {}", e))?;
    Ok(status.success())
}

// Local OpenClaw detection (no SSH needed)
#[tauri::command]
pub async fn check_local_openclaw() -> Result<bool, String> {
    tokio::task::spawn_blocking(|| {
        std::process::Command::new("pgrep")
            .args(["-f", "openclaw"])
            .stdout(std::process::Stdio::null())
            .status()
            .map(|s| s.success())
            .unwrap_or(false)
    })
    .await
    .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn get_local_ssh_info() -> Result<(String, Option<String>), String> {
    let username = std::env::var("USER")
        .or_else(|_| std::env::var("USERNAME"))
        .map_err(|_| "Cannot determine username".to_string())?;

    let home = std::env::var("HOME").unwrap_or_default();
    let key_names = ["id_ed25519", "id_rsa", "id_ecdsa"];
    let ssh_key_path = key_names
        .iter()
        .map(|k| format!("{}/.ssh/{}", home, k))
        .find(|p| std::path::Path::new(p).exists());

    Ok((username, ssh_key_path))
}

// Terminal commands
#[tauri::command]
pub async fn check_openclaw_status(peer: PeerDevice) -> Result<bool, String> {
    let peer_clone = peer;
    tokio::task::spawn_blocking(move || {
        crate::terminal::check_openclaw_running(&peer_clone)
    })
    .await
    .map_err(|e| e.to_string())?
    .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn open_terminal(
    state: State<'_, TerminalState>,
    peer: PeerDevice,
    cols: u32,
    rows: u32,
    on_data: Channel<Vec<u8>>,
) -> Result<String, String> {
    state
        .open_session(peer, cols, rows, on_data)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn send_terminal_input(
    state: State<'_, TerminalState>,
    session_id: String,
    data: Vec<u8>,
) -> Result<(), String> {
    state
        .send_input(&session_id, data)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn resize_terminal(
    state: State<'_, TerminalState>,
    session_id: String,
    cols: u32,
    rows: u32,
) -> Result<(), String> {
    state
        .resize(&session_id, cols, rows)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn close_terminal(
    state: State<'_, TerminalState>,
    session_id: String,
) -> Result<(), String> {
    state
        .close_session(&session_id)
        .await
        .map_err(|e| e.to_string())
}

// Utility commands
#[tauri::command]
pub fn get_device_id() -> String {
    uuid::Uuid::new_v4().to_string()
}

#[tauri::command]
pub fn get_device_name() -> String {
    hostname::get()
        .map(|h| h.to_string_lossy().to_string())
        .unwrap_or_else(|_| "Unknown".to_string())
}
