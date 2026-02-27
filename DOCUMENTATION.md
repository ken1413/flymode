# FlyMode - Wireless Scheduler + P2P Sync + Notes

跨平台桌面應用程式，整合無線控制、P2P 同步、便利貼筆記三大功能。

---

## 目錄

1. [專案概述](#專案概述)
2. [功能總覽](#功能總覽)
3. [系統架構](#系統架構)
4. [技術棧](#技術棧)
5. [目錄結構](#目錄結構)
6. [核心模組設計](#核心模組設計)
7. [P2P 同步架構](#p2p-同步架構)
8. [便利貼模組](#便利貼模組)
9. [檔案傳輸](#檔案傳輸)
10. [API 參考](#api-參考)
11. [建置與部署](#建置與部署)
12. [安全性考量](#安全性考量)

---

## 專案概述

### 背景

FlyMode 是一款整合多種實用功能的桌面應用程式：

1. **無線控制**：定時管理 WiFi、藍牙、飛航模式
2. **P2P 同步**：透過 Tailscale/SSH 在裝置間同步資料
3. **便利貼筆記**：多顏色多類別的便利貼系統
4. **檔案傳輸**：裝置間直接傳送檔案

### 設計理念

- **P2P 架構**：無需中央伺服器，裝置間直接通訊
- **跨平台**：支援 Linux、Windows、macOS
- **輕量高效**：使用 Rust + Tauri，資源佔用低
- **安全可靠**：SSH 加密傳輸，本地資料庫存儲

---

## 功能總覽

### 1. 無線排程 (Wireless Scheduler)

| 功能 | 說明 |
|------|------|
| 定時規則 | 設定時間範圍和星期幾自動開關 |
| 快速操作 | 即時切換 WiFi/藍牙/飛航模式 |
| 自定義命令 | 執行任意 CLI 命令 |
| 系統匣 | 背景執行，最小化到系統匣 |

### 2. P2P 同步 (Peer-to-Peer Sync)

| 功能 | 說明 |
|------|------|
| 裝置管理 | 新增、編輯、移除遠端裝置 |
| Tailscale 發現 | 自動發現 Tailscale 網路中的裝置 |
| 信任機制 | 標記信任裝置以啟用自動同步 |
| SSH 連線 | 使用 SSH 金鑰或密碼認證 |

### 3. 便利貼筆記 (Sticky Notes)

| 功能 | 說明 |
|------|------|
| 多顏色 | 8 種顏色選擇（黃、粉、藍、綠、紫、橙、白、灰）|
| 分類 | 7 種類別（一般、工作、個人、點子、任務、重要、封存）|
| 標籤 | 自定義標籤系統 |
| 釘選 | 置頂重要筆記 |
| 同步 | 自動同步到信任裝置 |

### 4. 檔案傳輸 (File Transfer)

| 功能 | 說明 |
|------|------|
| 上傳 | 傳送檔案到遠端裝置 |
| 下載 | 從遠端裝置下載檔案 |
| 瀏覽 | 瀏覽遠端檔案系統 |
| 佇列 | 管理傳輸佇列 |

---

## 系統架構

```
┌─────────────────────────────────────────────────────────────────────┐
│                          FlyMode App v0.2                           │
├─────────────────────────────────────────────────────────────────────┤
│                                                                     │
│  ┌──────────────────────────────────────────────────────────────┐  │
│  │                     Frontend (Preact + TypeScript)           │  │
│  │  ┌────────┐ ┌────────┐ ┌────────┐ ┌────────┐ ┌────────┐     │  │
│  │  │ Notes  │ │  P2P   │ │  Sync  │ │Transfer│ │Schedule│     │  │
│  │  │  Tab   │ │  Tab   │ │  Tab   │ │  Tab   │ │  Tab   │     │  │
│  │  └────────┘ └────────┘ └────────┘ └────────┘ └────────┘     │  │
│  └────────────────────────────┬─────────────────────────────────┘  │
│                               │ Tauri IPC                          │
│  ┌────────────────────────────▼─────────────────────────────────┐  │
│  │                     Backend (Rust + Tauri)                    │  │
│  │  ┌─────────────┐ ┌─────────────┐ ┌─────────────┐              │  │
│  │  │   Notes     │ │     P2P     │ │    Sync     │              │  │
│  │  │   Store     │ │   Manager   │ │   Engine    │              │  │
│  │  │  (SQLite)   │ │   (SSH2)    │ │             │              │  │
│  │  └─────────────┘ └─────────────┘ └─────────────┘              │  │
│  │  ┌─────────────┐ ┌─────────────┐ ┌─────────────┐              │  │
│  │  │  Transfer   │ │  Scheduler  │ │  Wireless   │              │  │
│  │  │   Manager   │ │             │ │  Controller │              │  │
│  │  └─────────────┘ └─────────────┘ └─────────────┘              │  │
│  └────────────────────────────┬─────────────────────────────────┘  │
│                               │                                    │
│  ┌────────────────────────────▼─────────────────────────────────┐  │
│  │  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐        │  │
│  │  │   Tailscale  │  │     SSH      │  │   SQLite     │        │  │
│  │  │   Network    │  │   Protocol   │  │   Database   │        │  │
│  │  └──────────────┘  └──────────────┘  └──────────────┘        │  │
│  └──────────────────────────────────────────────────────────────┘  │
│                                                                     │
│  ┌──────────────────────────────────────────────────────────────┐  │
│  │                    System Tray + Auto Start                   │  │
│  └──────────────────────────────────────────────────────────────┘  │
│                                                                     │
└─────────────────────────────────────────────────────────────────────┘
```

---

## 技術棧

### 後端 (Rust)

| 套件 | 版本 | 用途 |
|------|------|------|
| `tauri` | 2.x | 桌面應用框架 |
| `tauri-plugin-shell` | 2.x | 執行外部命令 |
| `tauri-plugin-dialog` | 2.x | 檔案對話框 |
| `tauri-plugin-fs` | 2.x | 檔案系統操作 |
| `tokio` | 1.x | 非同步運行時 |
| `rusqlite` | 0.31 | SQLite 資料庫 |
| `ssh2` | 0.9 | SSH 通訊協定 |
| `serde` | 1.x | 序列化 |
| `chrono` | 0.4 | 時間處理 |
| `sha2` | 0.10 | 雜湊計算 |

### 前端 (TypeScript/Preact)

| 套件 | 版本 | 用途 |
|------|------|------|
| `preact` | 10.x | UI 框架 |
| `@tauri-apps/api` | 2.x | Tauri API |
| `@tauri-apps/plugin-dialog` | 2.x | 檔案選擇 |
| `vite` | 5.x | 建置工具 |

---

## 目錄結構

```
flymode/
├── Cargo.toml                    # Workspace 根配置
├── README.md                     # 專案說明
├── DOCUMENTATION.md              # 技術文件
│
├── src-tauri/                    # Rust 後端
│   ├── Cargo.toml                # Rust 依賴配置
│   ├── build.rs                  # 建置腳本
│   ├── tauri.conf.json           # Tauri 配置
│   ├── icons/                    # 應用圖示
│   └── src/
│       ├── main.rs               # 應用入口
│       ├── config/               # 無線排程配置
│       ├── notes/                # 便利貼模組
│       │   └── mod.rs            # SQLite 存儲、Note 結構
│       ├── p2p/                  # P2P 連接管理
│       │   └── mod.rs            # SSH 客戶端、裝置管理
│       ├── sync/                 # 資料同步引擎
│       │   └── mod.rs            # 同步邏輯、衝突解決
│       ├── transfer/             # 檔案傳輸
│       │   └── mod.rs            # 上傳/下載管理
│       ├── scheduler/            # 定時任務
│       ├── wireless/             # 無線控制
│       └── commands/             # Tauri IPC 命令
│
└── src-ui/                       # 前端
    ├── package.json              # npm 配置
    ├── tsconfig.json             # TypeScript 配置
    ├── vite.config.ts            # Vite 配置
    ├── index.html                # HTML 入口
    └── src/
        ├── main.tsx              # 應用入口
        ├── App.tsx               # 主應用組件
        ├── style.css             # 全域樣式
        └── components/
            ├── NotesTab.tsx      # 便利貼頁面
            ├── P2PTab.tsx        # 裝置管理頁面
            ├── SyncTab.tsx       # 同步頁面
            ├── TransferTab.tsx   # 檔案傳輸頁面
            ├── RulesTab.tsx      # 排程規則頁面
            ├── QuickActionsTab.tsx # 快速操作頁面
            └── SettingsTab.tsx   # 設定頁面
```

---

## 核心模組設計

### 1. Notes Store (`src-tauri/src/notes/mod.rs`)

#### 資料結構

```rust
pub struct Note {
    pub id: String,              // UUID
    pub title: String,           // 標題
    pub content: String,         // 內容
    pub color: NoteColor,        // 顏色
    pub category: NoteCategory,  // 類別
    pub pinned: bool,            // 是否釘選
    pub archived: bool,          // 是否封存
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub tags: Vec<String>,       // 標籤
    pub position_x: i32,         // X 座標（預留）
    pub position_y: i32,         // Y 座標（預留）
    pub width: i32,              // 寬度
    pub height: i32,             // 高度
    pub device_id: String,       // 建立裝置 ID
    pub sync_hash: Option<String>,// 同步雜湊
    pub deleted: bool,           // 軟刪除標記
}

pub enum NoteColor {
    Yellow, Pink, Blue, Green,
    Purple, Orange, White, Gray,
}

pub enum NoteCategory {
    General, Work, Personal, Ideas,
    Tasks, Important, Archive,
}
```

#### SQLite Schema

```sql
CREATE TABLE notes (
    id TEXT PRIMARY KEY,
    title TEXT NOT NULL,
    content TEXT NOT NULL,
    color TEXT NOT NULL,
    category TEXT NOT NULL,
    pinned INTEGER NOT NULL DEFAULT 0,
    archived INTEGER NOT NULL DEFAULT 0,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL,
    tags TEXT NOT NULL DEFAULT '[]',
    position_x INTEGER NOT NULL DEFAULT 0,
    position_y INTEGER NOT NULL DEFAULT 0,
    width INTEGER NOT NULL DEFAULT 280,
    height INTEGER NOT NULL DEFAULT 200,
    device_id TEXT NOT NULL,
    sync_hash TEXT,
    deleted INTEGER NOT NULL DEFAULT 0
);

CREATE INDEX idx_notes_updated ON notes(updated_at);
CREATE INDEX idx_notes_category ON notes(category);
CREATE INDEX idx_notes_deleted ON notes(deleted);
```

### 2. P2P Manager (`src-tauri/src/p2p/mod.rs`)

#### 資料結構

```rust
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

pub enum ConnectionType {
    Tailscale,    // Tailscale VPN
    LanDirect,    // 區域網路直連
    WanDirect,    // 廣域網路直連
}
```

#### Tailscale 發現

```rust
// 執行 tailscale status --json 解析
let output = Command::new("tailscale")
    .args(["status", "--json"])
    .output()
    .await;

// 解析 JSON 取得 Peer 列表
let status: TailscaleStatus = serde_json::from_str(&json_str)?;
```

### 3. SSH Client

```rust
pub struct SSHClient {
    session: Option<Session>,
}

impl SSHClient {
    // 連線
    pub fn connect(&mut self, peer: &PeerDevice) -> Result<()>;
    
    // 執行命令
    pub fn execute_command(&mut self, command: &str) -> Result<String>;
    
    // 上傳檔案 (SFTP)
    pub fn upload_file(&mut self, local: &PathBuf, remote: &str) -> Result<()>;
    
    // 下載檔案 (SFTP)
    pub fn download_file(&mut self, remote: &str, local: &PathBuf) -> Result<()>;
    
    // 列出遠端檔案
    pub fn list_remote_files(&mut self, dir: &str) -> Result<Vec<RemoteFileInfo>>;
}
```

---

## P2P 同步架構

### 同步流程

```
Device A                          Device B
   │                                 │
   │  1. SSH Connect                 │
   │────────────────────────────────►│
   │                                 │
   │  2. Send Sync Payload           │
   │    (local changes as JSON)      │
   │────────────────────────────────►│
   │                                 │
   │  3. Receive Remote Payload      │
   │◄────────────────────────────────│
   │    (remote changes as JSON)     │
   │                                 │
   │  4. Apply Remote Changes        │
   │    (merge to local SQLite)      │
   │                                 │
   │  5. Disconnect                  │
   │◄────────────────────────────────►│
   │                                 │
```

### Sync Payload

```rust
pub struct SyncPayload {
    pub device_id: String,
    pub device_name: String,
    pub timestamp: DateTime<Utc>,
    pub notes: Vec<Note>,
    pub sync_folder_files: Vec<FileSyncInfo>,
}
```

### 衝突解決

使用 **Last-Write-Wins** 策略：
- 比較 `updated_at` 時間戳
- 較新的變更覆蓋較舊的
- 使用 `sync_hash` 檢測實際變更

```rust
fn should_update(local: &Note, remote: &Note) -> bool {
    // 如果 hash 相同，無需更新
    if local.sync_hash == remote.sync_hash {
        return false;
    }
    // 較新的更新時間優先
    remote.updated_at > local.updated_at
}
```

---

## 便利貼模組

### 顏色配置

| 顏色 | Hex | 用途建議 |
|------|-----|----------|
| Yellow | `#fef08a` | 一般筆記 |
| Pink | `#fbcfe8` | 個人/心情 |
| Blue | `#bfdbfe` | 工作/專案 |
| Green | `#bbf7d0` | 完成事項 |
| Purple | `#e9d5ff` | 創意點子 |
| Orange | `#fed7aa` | 待辦事項 |
| White | `#ffffff` | 簡潔筆記 |
| Gray | `#e5e7eb` | 封存/參考 |

### 分類系統

| 類別 | 說明 |
|------|------|
| General | 一般筆記 |
| Work | 工作相關 |
| Personal | 個人事物 |
| Ideas | 創意點子 |
| Tasks | 待辦任務 |
| Important | 重要事項 |
| Archive | 封存筆記 |

---

## 檔案傳輸

### 傳輸佇列

```rust
pub struct TransferQueue {
    pub transfers: Vec<TransferProgress>,
    pub max_concurrent: usize,  // 預設 3
}

pub struct TransferProgress {
    pub transfer_id: String,
    pub peer_id: String,
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
```

### 傳輸狀態

| 狀態 | 說明 |
|------|------|
| Pending | 等待中 |
| InProgress | 傳輸中 |
| Completed | 已完成 |
| Failed | 失敗 |
| Cancelled | 已取消 |

---

## API 參考

### Notes API

| 命令 | 參數 | 返回值 |
|------|------|--------|
| `create_note` | title, content | Note |
| `update_note` | note | - |
| `delete_note` | id | - |
| `get_note` | id | Option<Note> |
| `list_notes` | include_archived | Vec<Note> |
| `search_notes` | query | Vec<Note> |

### P2P API

| 命令 | 參數 | 返回值 |
|------|------|--------|
| `get_p2p_config` | - | P2PConfig |
| `save_p2p_config` | config | - |
| `add_peer` | peer | - |
| `remove_peer` | peer_id | - |
| `update_peer` | peer | - |
| `check_peer_status` | peer | DeviceStatus |
| `discover_tailscale` | - | Vec<PeerDevice> |

### Sync API

| 命令 | 參數 | 返回值 |
|------|------|--------|
| `get_sync_state` | - | SyncState |
| `sync_with_peer` | peer | SyncResult |
| `sync_all_peers` | - | Vec<SyncResult> |
| `export_notes` | - | String (JSON) |
| `import_notes` | json | usize |

### Transfer API

| 命令 | 參數 | 返回值 |
|------|------|--------|
| `get_transfer_queue` | - | TransferQueue |
| `upload_file` | peer, local_path, remote_path | transfer_id |
| `download_file` | peer, remote_path, local_path | transfer_id |
| `cancel_transfer` | transfer_id | - |
| `browse_remote_files` | peer, path | Vec<RemoteFileInfo> |

---

## 建置與部署

### 環境需求

| 工具 | 版本 | 說明 |
|------|------|------|
| Rust | 1.70+ | 編譯後端 |
| Node.js | 18+ | 前端建置 |
| SQLite | 3.x | 資料庫（bundled） |

### 安裝依賴

```bash
# Linux 系統依賴
sudo apt install libgtk-3-dev libwebkit2gtk-4.0-dev \
                 libappindicator3-dev librsvg2-dev patchelf \
                 libssl-dev pkg-config

# 前端依賴
cd src-ui && npm install
```

### 開發模式

```bash
cargo tauri dev
```

### 生產建置

```bash
cargo tauri build
```

---

## 安全性考量

### SSH 認證

1. **金鑰認證（推薦）**：使用 SSH 金鑰進行認證
2. **密碼認證**：密碼僅存儲於本地配置檔案

### 資料存儲

| 資料類型 | 存儲位置 | 加密 |
|----------|----------|------|
| 配置 | `~/.config/flymode/` | 否 |
| 筆記 | `~/.local/share/flymode/notes.db` | 否 |
| SSH 密碼 | `~/.config/flymode/p2p.json` | 否（未來可加密）|

### 網路安全

- 所有通訊透過 SSH 加密
- 無需暴露端口（作為 SSH 客戶端）
- 支援 Tailscale 私有網路

---

## 設定檔案

### 無線排程配置 (`~/.config/flymode/config.json`)

```json
{
  "rules": [...],
  "check_interval_seconds": 60,
  "show_notifications": true,
  "minimize_to_tray": true,
  "auto_start": false
}
```

### P2P 配置 (`~/.config/flymode/p2p.json`)

```json
{
  "device_id": "uuid-v4",
  "device_name": "My Laptop",
  "listen_port": 4827,
  "peers": [...],
  "auto_discover_tailscale": true,
  "sync_enabled": true,
  "sync_interval_seconds": 300
}
```

---

*文件最後更新：2026-02-27*
*版本：v0.2.0*
