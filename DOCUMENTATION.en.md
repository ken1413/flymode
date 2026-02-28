# FlyMode User Guide

Cross-platform desktop app — Wireless Scheduling + P2P Device Sync + Sticky Notes + File Transfer + Remote Terminal

Version: v0.2.0 | Last updated: 2026-03-01

---

## Table of Contents

1. [Installation](#1-installation)
2. [Post-Install Setup](#2-post-install-setup)
3. [Pairing Two Machines](#3-pairing-two-machines)
4. [Features](#4-features)
   - [Sticky Notes](#41-sticky-notes)
   - [Device Management](#42-device-management)
   - [Data Sync](#43-data-sync)
   - [File Transfer](#44-file-transfer)
   - [OpenClaw Remote Management](#45-openclaw-remote-management)
   - [Wireless Scheduling](#46-wireless-scheduling)
   - [Quick Actions](#47-quick-actions)
   - [Settings](#48-settings)
5. [Architecture](#5-architecture)
6. [Data Storage](#6-data-storage)
7. [Security](#7-security)
8. [Troubleshooting](#8-troubleshooting)
9. [Technical Reference](#9-technical-reference)

---

## 1. Installation

### 1.1 One-Line Install (Recommended)

On Linux (Ubuntu/Fedora/Arch) or macOS:

```bash
curl -fsSL https://raw.githubusercontent.com/ken1413/flymode/main/setup.sh | bash
```

The installer automatically handles:

| Step | Details |
|------|---------|
| System dependencies | GTK3, WebKit2GTK, OpenSSL, pkg-config, etc. |
| Rust toolchain | Installs `rustup` + stable toolchain |
| Node.js 22 LTS | Required for frontend build |
| GitHub CLI (`gh`) | Authenticates to clone the repo |
| Tauri CLI | `cargo install tauri-cli` |
| SSH Server | `openssh-server`, auto-started |
| Compilation | `cargo tauri build` (release mode) |
| Installation | Binary placed at `~/.local/bin/flymode` |
| Desktop shortcut | `.desktop` entry created (Linux) |

> **Note:** First-time install requires GitHub authentication. The installer will guide you through `gh auth login`.

After installation:

```bash
flymode
```

### 1.2 Manual Install

#### Requirements

| Tool | Minimum Version | Purpose |
|------|----------------|---------|
| Rust | 1.70+ | Backend compilation |
| Node.js | 18+ | Frontend build |
| npm | 9+ | Package management |

#### Linux (Ubuntu/Debian)

```bash
sudo apt install build-essential curl wget git pkg-config \
    libgtk-3-dev libwebkit2gtk-4.1-dev \
    libayatana-appindicator3-dev librsvg2-dev patchelf \
    libssl-dev libsoup-3.0-dev libjavascriptcoregtk-4.1-dev
```

#### Linux (Fedora)

```bash
sudo dnf install gcc gcc-c++ make curl wget git pkg-config \
    gtk3-devel webkit2gtk4.1-devel \
    libappindicator-gtk3-devel librsvg2-devel \
    openssl-devel libsoup3-devel javascriptcoregtk4.1-devel
```

#### Linux (Arch)

```bash
sudo pacman -Syu --needed base-devel curl wget git pkg-config \
    gtk3 webkit2gtk-4.1 libappindicator-gtk3 librsvg patchelf \
    openssl libsoup3
```

#### macOS

```bash
# Install Xcode command-line tools
xcode-select --install

# Install Homebrew (if not already installed)
/bin/bash -c "$(curl -fsSL https://raw.githubusercontent.com/Homebrew/install/HEAD/install.sh)"
```

#### Build & Install

```bash
# Clone
git clone https://github.com/ken1413/flymode.git
cd flymode

# Install frontend dependencies
cd src-ui && npm install && cd ..

# Build
cargo tauri build

# Install to PATH
mkdir -p ~/.local/bin
cp target/release/flymode ~/.local/bin/
```

### 1.3 Updating

Re-run the one-line installer. It will `git pull` and rebuild:

```bash
curl -fsSL https://raw.githubusercontent.com/ken1413/flymode/main/setup.sh | bash
```

Or manually:

```bash
cd ~/app/flymode
git pull
cd src-ui && npm install && cd ..
cargo tauri build
cp target/release/flymode ~/.local/bin/
```

---

## 2. Post-Install Setup

The installer only handles compilation and installation. For P2P features, additional setup is required.

### 2.1 SSH Server (Required)

Both machines that need to communicate **must** have an SSH server. The installer handles this automatically, but for manual installs:

```bash
# Ubuntu/Debian
sudo apt install openssh-server
sudo systemctl enable --now ssh

# Fedora
sudo dnf install openssh-server
sudo systemctl enable --now sshd

# Arch
sudo pacman -S openssh
sudo systemctl enable --now sshd
```

macOS: System Settings > General > Sharing > Remote Login > On

#### SSH Key Authentication (Recommended)

Key-based auth avoids entering passwords:

```bash
# Generate a key (if you don't have one)
ssh-keygen -t ed25519

# Copy public key to remote machine
ssh-copy-id user@remote-ip
```

FlyMode will automatically use `~/.ssh/id_ed25519` or `~/.ssh/id_rsa` for connections.

### 2.2 Tailscale (Recommended)

Tailscale lets two machines connect even across different networks (e.g., home and office).

```bash
# Install Tailscale
curl -fsSL https://tailscale.com/install.sh | sh

# Start and log in
sudo tailscale up
```

Once both machines are logged into the same Tailscale account, FlyMode will **auto-discover** each other.

### 2.3 Firewall (If Needed)

FlyMode requires these ports:

| Port | Protocol | Purpose |
|------|----------|---------|
| 4827 | TCP | Device pairing requests |
| 22 | TCP | SSH connections (sync, transfer, terminal) |

```bash
# Ubuntu (ufw)
sudo ufw allow 4827/tcp
sudo ufw allow 22/tcp

# Fedora (firewalld)
sudo firewall-cmd --permanent --add-port=4827/tcp
sudo firewall-cmd --permanent --add-service=ssh
sudo firewall-cmd --reload
```

> If both machines are on the same LAN with no firewall restrictions, no extra setup is needed.
> Tailscale traffic bypasses firewalls automatically.

---

## 3. Pairing Two Machines

Assuming:
- **Machine A**: FlyMode installed
- **Machine B**: FlyMode installed
- Both have SSH Server running
- Both are on the same Tailscale network (or LAN)

### Step 1: Find Each Other's IP

**Option A — Tailscale Auto-Discovery (Recommended):**

In Machine A's FlyMode > Devices tab > click "Discover Tailscale Peers".
Machine B will automatically appear in the device list.

**Option B — Manual Add:**

On Machine B, find its IP:

```bash
# Tailscale IP
tailscale ip

# Or LAN IP
hostname -I
```

In Machine A's FlyMode > Devices tab > click "Add Peer", fill in:
- **Name**: Custom name (e.g., "Office PC")
- **IP Address**: Machine B's IP
- **SSH User**: Machine B's username
- **SSH Port**: 22 (default)
- **Auth**: SSH key path or password

### Step 2: Pair

Find Machine B in the device list > click "Pair".

Machine B's FlyMode will receive a pairing request. On Machine B's Devices tab > "Incoming Pair Requests" section > click "Accept".

After pairing, both machines appear in each other's device list.

### Step 3: Configure SSH Credentials (Required)

Pairing only exchanges device information (IP, name) — **it does not automatically configure SSH credentials**. You must set them manually:

1. In the device list, click the "Edit" button on the peer's device card
2. Enter **SSH User** (the remote machine's username)
3. Enter authentication (choose one):
   - **SSH Key Path**: e.g., `~/.ssh/id_ed25519` (recommended — requires `ssh-copy-id` first)
   - **SSH Password**: the remote machine's login password
4. Save

> **Important:** Both machines must configure each other's SSH credentials. Without this, sync, file transfer, and terminal features will fail due to SSH connection errors.

Once configured, the device status should show **Online** (green dot). If it still shows Offline, see [Troubleshooting](#8-troubleshooting).

### Step 4: Establish Trust

Click the "Trust" button on the other machine's device card. After trusting:
- Notes auto-sync to that device
- File transfer enabled
- Remote terminal enabled

> **Both machines must Trust each other for bidirectional sync.**

### Step 5: Verify Connection

- Sync tab: Click "Sync Now" to verify notes sync
- Transfer tab: Try uploading a small file
- Status bar should show the peer as Online (green dot)

---

## 4. Features

### 4.1 Sticky Notes

#### Basic Operations

| Action | Description |
|--------|-------------|
| Create | Click "+ New Note", enter title and content |
| Edit | Click the "Edit" button on a note card |
| Delete | Click "Delete" (soft delete — can be recovered via sync) |
| Pin | Click "Pin" — pinned notes appear at the top |
| Search | Type keywords in the search bar to search title and content |

#### Colors

| Color | Hex | Suggested Use |
|-------|-----|---------------|
| Yellow | `#fef08a` | General notes |
| Pink | `#fbcfe8` | Personal / mood |
| Blue | `#bfdbfe` | Work / projects |
| Green | `#bbf7d0` | Completed items |
| Purple | `#e9d5ff` | Creative ideas |
| Orange | `#fed7aa` | To-do items |
| White | `#ffffff` | Clean notes |
| Gray | `#e5e7eb` | Archived / reference |

#### Categories

| Category | Description |
|----------|-------------|
| General | General notes |
| Work | Work-related |
| Personal | Personal matters |
| Ideas | Creative ideas |
| Tasks | To-do tasks |
| Important | Important items |
| Archive | Archived notes |

#### Tags

Add tags (e.g., `#projectA`, `#todo`) when editing notes. Tags are displayed on the note card.

#### View Modes

Toggle between **Grid** and **List** view using the button in the top-right corner.

#### Sync Behavior

- Notes auto-refresh every 5 seconds
- With trusted devices and auto-sync enabled, notes sync in the background
- Deleted notes are soft-deleted (not removed from the database) and the deletion propagates via sync

---

### 4.2 Device Management

#### Device Info

The top of the page shows local machine info:

- **Device ID**: Unique identifier (UUID)
- **Device Name**: Machine hostname
- **Listen Port**: TCP pairing service port (default 4827)

#### Adding a Device

| Field | Description | Example |
|-------|-------------|---------|
| Name | Custom name | Office PC |
| Hostname | Remote hostname | my-desktop |
| IP Address | Remote IP | 100.64.0.2 |
| SSH Port | SSH port | 22 |
| SSH User | SSH username | alice |
| Connection Type | Connection type | Tailscale / LAN Direct / WAN Direct |
| SSH Key Path | Key path (optional) | ~/.ssh/id_ed25519 |
| SSH Password | Password (optional) | If not using key auth |

#### Connection Type Icons

| Icon | Type | Description |
|------|------|-------------|
| 🦎 | Tailscale | Via Tailscale VPN |
| 🏠 | LAN Direct | Local area network |
| 🌐 | WAN Direct | Wide area network |

#### Device Status

| Status | Description |
|--------|-------------|
| 🟢 Online | Connection OK (SSH reachable) |
| 🔴 Offline | Unreachable |
| ⚪ Unknown | Not yet checked |

Status auto-updates every 30 seconds.

#### Tailscale Auto-Discovery

Click "Discover Tailscale Peers" — FlyMode runs `tailscale status --json` to find all devices on the same Tailscale network and adds them to the device list.

#### TCP Pairing Protocol

FlyMode has a built-in TCP pairing service (port 4827):

1. **Machine A** clicks "Pair" > sends pairing request to Machine B
2. **Machine B** receives it > displayed in "Incoming Pair Requests"
3. **Machine B** clicks "Accept" > both machines add each other
4. Pairing complete — both connected and visible

#### Trust Model

- **Untrusted devices**: Can only see online status, no sync or transfer
- **Trusted devices**: Auto-sync notes, transfer files, open remote terminal
- Toggle via the "Trust / Untrust" button on the device card

#### OpenClaw Detection

FlyMode detects OpenClaw on both the local machine and remote devices:

- **Local**: Uses `pgrep` (no SSH needed) — if detected, a ">_" button appears on the "This Device" card
- **Remote**: Runs `pgrep -f openclaw` via SSH — if detected, a ">_" button appears on the device card

Detection runs every 120 seconds.

---

### 4.3 Data Sync

#### Sync Strategy

FlyMode uses **Last-Write-Wins (LWW)** conflict resolution:

- When two devices modify the same note, the one with the newer `updated_at` timestamp wins
- `sync_hash` (SHA-256) detects actual changes to avoid unnecessary overwrites
- Notes only on one side are synced to the other
- Deletion status is also synced (soft delete)

#### Auto-Sync

| Setting | Description |
|---------|-------------|
| Enable/Disable | Auto-Sync toggle on the Sync tab |
| Interval | Choose from 1 min, 5 min, 15 min, 30 min, 1 hour |
| Targets | All trusted and online devices |

#### Manual Operations

| Action | Description |
|--------|-------------|
| Sync Now | Immediately sync all trusted devices |
| Sync with Peer | Sync only with a specific device |

#### Sync History

The bottom of the page shows the last 10 sync results:

- Peer device name
- Sync status (success/failure)
- Number of notes synced
- Duration
- Error message (if any)

#### Export / Import

| Action | Description |
|--------|-------------|
| Export Notes | Export all notes as a JSON file |
| Import Notes | Import notes from a JSON file |

Useful for backup/restore, or manually exchanging notes when SSH isn't convenient.

---

### 4.4 File Transfer

#### Uploading Files

1. Select target device (must be trusted and online)
2. Click "Upload" > native file picker opens
3. Select the file to upload
4. Enter remote destination path (defaults to the peer's home directory)
5. Transfer starts with a progress bar

#### Downloading Files

1. Select source device
2. Browse the remote file system (click directories to navigate)
3. Click "Download" on the target file
4. Choose local save path
5. Transfer starts with a progress bar

#### Remote File Browser

- Displays file name, size, and last modified date
- Directories are clickable to enter
- `..` navigates to the parent directory

#### Transfer Management

| Feature | Description |
|---------|-------------|
| Progress bar | Real-time percentage and transfer speed |
| Cancel | Cancel in-progress transfers |
| Clear completed | Remove completed transfers from the list |
| Concurrency limit | Up to 3 simultaneous transfers |

#### Transfer States

| Status | Description |
|--------|-------------|
| Pending | Queued, waiting |
| InProgress | Currently transferring |
| Completed | Done |
| Failed | Error occurred (message displayed) |
| Cancelled | User cancelled |

---

### 4.5 OpenClaw Remote Management

FlyMode provides deep integration with [OpenClaw](https://github.com/openclaw), offering a seamless detect-connect-operate experience for remote management. You can manage multiple OpenClaw instances across different machines directly from FlyMode's Devices tab — no manual SSH required.

#### Automatic OpenClaw Gateway Detection

FlyMode automatically detects OpenClaw on remote devices:

| Item | Details |
|------|---------|
| Detection method | Runs `pgrep -f openclaw-gateway` via SSH |
| Detection interval | Automatic scan every 120 seconds |
| Detection targets | All **trusted** and **online** devices |
| UI indicator | ">_" terminal button appears on the device card |

No extra configuration needed — as long as OpenClaw Gateway is running on the remote device, FlyMode will discover it automatically.

#### One-Click OpenClaw TUI Launch

Click the ">_" button on any device card, and FlyMode will automatically:

1. **Establish an SSH PTY connection** to the remote device
2. **Auto-locate the `openclaw` binary** — first via `which openclaw`, then searches `/home`, `/usr/local/bin`, `/usr/bin`, `/opt` (including symlinks)
3. **Launch OpenClaw TUI** — executes `openclaw tui` with UTF-8 environment
4. **Open an embedded terminal** — displays the full TUI interface within the FlyMode window

The entire process requires a single click — no need to remember hostnames, IPs, paths, or commands.

#### Local OpenClaw

FlyMode doesn't just detect remote devices — it also detects **local** OpenClaw. If OpenClaw is running on the local machine, a ">_" button appears on the "This Device" card. Clicking it connects via SSH localhost.

- Local detection uses `pgrep` (no SSH needed) — faster than remote detection
- If no SSH key is found on the local machine, a password prompt appears
- The password is remembered for the current FlyMode session (no re-prompting)

#### Multi-Device Tab Switching

If you have multiple machines running OpenClaw (including the local machine), FlyMode provides a **browser-tab style** multi-session terminal:

```
┌──────────────────────────────────────────────────────────┐
│ [● My PC (localhost)] [○ Office Server] [○ Home NAS]  [x] │
├──────────────────────────────────────────────────────────┤
│                                                            │
│              active device's OpenClaw TUI                  │
│              (xterm.js)                                    │
│                                                            │
└──────────────────────────────────────────────────────────┘
```

- **Device Navbar**: Lists all devices with OpenClaw detected (local machine first)
- **Status dots**: 🟢 Connected, 🔵 Connecting (pulsing), 🔴 Error, ⚪ Not yet connected
- **First click** on a device → establishes SSH connection + xterm instance
- **Subsequent clicks** on a connected device → just switches display, session keeps running
- Each xterm instance runs independently (show/hide toggle, no destroy/recreate)
- Closing `[x]` → closes **all** SSH sessions, modal disappears
- If only one device has OpenClaw → navbar is hidden, behaves like a single session

Combine with Tailscale for cross-network management (local machine, home NAS, office server, cloud VPS) — all switchable within the same window.

#### Terminal Features

| Feature | Description |
|---------|-------------|
| CJK Input | Full support for fcitx5, iBus, and other CJK input methods — no duplicate character issues |
| Clipboard Copy | Select text to auto-copy to system clipboard |
| Clipboard Paste | `Ctrl+Shift+V` to paste |
| Dynamic Resize | Terminal columns and rows auto-adjust when the window is resized |
| Cursor | Blinking block cursor, clearly visible on any background |
| 256 Colors | Full xterm-256color support |
| WebGL Rendering | GPU-accelerated rendering for smooth TUI operation |

#### Use Cases

| Scenario | Description |
|----------|-------------|
| Remote server management | Connect to OpenClaw servers at the office or in the cloud via Tailscale |
| Multi-node monitoring | Check the status of multiple OpenClaw nodes from one machine |
| Mobile workflow | Connect back to home or office OpenClaw from your laptop, anywhere |
| Cross-platform | Linux / macOS / Windows (planned) can all connect to OpenClaw on any platform |

#### Technical Details

| Item | Details |
|------|---------|
| Terminal engine | xterm.js v6.1 (beta) + WebGL renderer |
| Connection protocol | SSH PTY (xterm-256color, dynamic cols/rows) |
| Path discovery | `bash -lc 'which openclaw'` → multi-directory `find` including symlinks |
| Encoding | UTF-8 (auto-sets `LANG=en_US.UTF-8 LC_ALL=en_US.UTF-8`) |
| IME handling | 50ms dedup + compositionend clear, prevents duplicate characters and text accumulation |
| Session management | Multi-session tab switching, each with unique Session ID, show/hide without destroy |
| Local detection | `pgrep` (no SSH), auto-detects username and SSH key path |

---

### 4.6 Wireless Scheduling

#### Creating Rules

| Field | Description |
|-------|-------------|
| Name | Rule name |
| Target | WiFi / Bluetooth / Airplane Mode / Custom Command |
| Action | Enable / Disable / Toggle / Run Command |
| Start Time | Start time (HH:MM format) |
| End Time | End time (optional, HH:MM format) |
| Days | Select which days to run (Mon-Sun) |
| Command | Custom command (only for Custom Command target) |

#### Rule Behavior

- **Single time point**: Only Start Time set > executes once at that time
- **Time range**: Start + End Time > Action at start, reverse action at end
- **Overnight range**: e.g., 22:00 - 06:00, active from 10 PM to 6 AM next day
- **Check interval**: Default every 60 seconds (configurable in Settings)

#### Management

| Action | Description |
|--------|-------------|
| Toggle | Enable/disable without deleting |
| Execute Now | Manually trigger immediately |
| Edit | Modify rule settings |
| Delete | Remove the rule |

---

### 4.7 Quick Actions

#### Wireless Controls

Three instant toggle buttons:

| Button | Function |
|--------|----------|
| WiFi | Enable/disable WiFi |
| Bluetooth | Enable/disable Bluetooth |
| Airplane Mode | Enable/disable Airplane Mode |

#### Custom Commands

Enter any shell command in the text field, click "Run". Output is displayed below.

> **Note:** Commands run with the current user's permissions. Root commands require `sudo` (with passwordless sudo configured).

---

### 4.8 Settings

| Setting | Description | Default |
|---------|-------------|---------|
| Show Notifications | Show system notifications | On |
| Minimize to Tray | Minimize to system tray on close | Off |
| Launch at Startup | Auto-start on system boot | Off |
| Require Password | Require system password on open | Off |
| Check Interval | Rule check interval (seconds) | 60 |

#### System Password Lock

When "Require Password" is enabled:
- System login password is required each time FlyMode opens
- Restoring from system tray after being hidden >1 second also requires re-authentication
- Uses Linux PAM (`unix_chkpwd`) for verification

#### System Tray

When "Minimize to Tray" is enabled:
- Closing the window minimizes to the system tray instead of quitting
- Click the tray icon to restore the window
- Right-click the tray icon > "Show FlyMode" or "Quit"

#### Version Info

The bottom of the Settings page shows:
- App version (e.g., v0.2.0)
- Git commit hash

---

## 5. Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                       FlyMode v0.2.0                         │
├─────────────────────────────────────────────────────────────┤
│                                                               │
│  ┌────────────── Frontend (Preact + TypeScript) ────────────┐ │
│  │                                                           │ │
│  │  📝 Notes  🔗 Devices  🔄 Sync  📤 Transfer             │ │
│  │  ⏰ Schedule  ⚡ Quick  ⚙️ Settings  🔒 Lock  >_ Terminal │ │
│  │                                                           │ │
│  └────────────────────────┬──────────────────────────────────┘ │
│                           │ Tauri IPC                          │
│  ┌────────────────────────▼──────────────────────────────────┐ │
│  │              Backend (Rust + Tauri 2 + Tokio)              │ │
│  │                                                            │ │
│  │  ┌──────────┐ ┌──────────┐ ┌──────────┐ ┌──────────┐     │ │
│  │  │  Notes   │ │   P2P    │ │   Sync   │ │ Transfer │     │ │
│  │  │ (SQLite) │ │  (SSH2)  │ │ (Engine) │ │  (SFTP)  │     │ │
│  │  └──────────┘ └──────────┘ └──────────┘ └──────────┘     │ │
│  │  ┌──────────┐ ┌──────────┐ ┌──────────┐ ┌──────────┐     │ │
│  │  │Scheduler │ │ Wireless │ │ Terminal │ │  Config  │     │ │
│  │  │ (Cron)   │ │ (nmcli)  │ │(SSH PTY) │ │  (JSON)  │     │ │
│  │  └──────────┘ └──────────┘ └──────────┘ └──────────┘     │ │
│  │                                                            │ │
│  └────────────────────────────────────────────────────────────┘ │
│                                                               │
│  ┌──────────────────────────────────────────────────────────┐ │
│  │         System Tray + Auto-Start + TCP Pairing (4827)     │ │
│  └──────────────────────────────────────────────────────────┘ │
│                                                               │
└─────────────────────────────────────────────────────────────┘
```

### Module Overview

| Module | Path | Description |
|--------|------|-------------|
| `main.rs` | `src-tauri/src/` | App entry — initializes all state, registers IPC commands, starts scheduler and auto-sync |
| `commands/` | `src-tauri/src/` | All Tauri IPC command handlers (30+ commands) |
| `notes/` | `src-tauri/src/` | Note CRUD, SQLite operations, sync_hash calculation, full-text search |
| `p2p/` | `src-tauri/src/` | Device management, SSH client, Tailscale discovery, TCP pairing service |
| `sync/` | `src-tauri/src/` | Sync engine, LWW conflict resolution, note merging |
| `transfer/` | `src-tauri/src/` | SFTP file transfer, queue management, progress tracking, concurrency control |
| `terminal/` | `src-tauri/src/` | SSH PTY management, OpenClaw detection, xterm.js backend |
| `scheduler/` | `src-tauri/src/` | Rule evaluation, time range calculation, action execution |
| `wireless/` | `src-tauri/src/` | Platform-specific WiFi/Bluetooth/Airplane Mode control |
| `config/` | `src-tauri/src/` | AppConfig and P2PConfig serialization (JSON) |
| `crypto/` | `src-tauri/src/` | SSH password encryption/decryption |

---

## 6. Data Storage

### File Locations

| Data | Path | Format |
|------|------|--------|
| App config | `~/.config/flymode/config.json` | JSON |
| P2P config | `~/.config/flymode/p2p.json` | JSON |
| Notes database | `~/.local/share/flymode/notes.db` | SQLite |
| Sync folder | `~/.local/share/flymode/sync/` | Files |

### config.json Example

```json
{
  "rules": [
    {
      "id": "550e8400-e29b-41d4-a716-446655440000",
      "name": "Disable WiFi at bedtime",
      "enabled": true,
      "action": "Disable",
      "target": "Wifi",
      "start_time": "23:00",
      "end_time": "07:00",
      "days": [0, 1, 2, 3, 4, 5, 6],
      "command": null
    }
  ],
  "check_interval_seconds": 60,
  "show_notifications": true,
  "minimize_to_tray": true,
  "auto_start": false,
  "require_password": false
}
```

### p2p.json Example

```json
{
  "device_id": "a1b2c3d4-e5f6-7890-abcd-ef1234567890",
  "device_name": "my-laptop",
  "listen_port": 4827,
  "peers": [
    {
      "id": "f9e8d7c6-b5a4-3210-fedc-ba0987654321",
      "name": "Office PC",
      "hostname": "my-desktop",
      "ip_address": "100.64.0.2",
      "port": 22,
      "connection_type": "Tailscale",
      "status": "Online",
      "last_seen": "2026-02-28T10:30:00Z",
      "ssh_user": "alice",
      "ssh_key_path": "~/.ssh/id_ed25519",
      "ssh_password": null,
      "is_trusted": true,
      "tailscale_hostname": "my-desktop",
      "flymode_version": "0.2.0"
    }
  ],
  "auto_discover_tailscale": true,
  "sync_enabled": true,
  "sync_interval_seconds": 300
}
```

### SQLite Database Schema

```sql
CREATE TABLE notes (
    id TEXT PRIMARY KEY,
    title TEXT NOT NULL,
    content TEXT NOT NULL,
    color TEXT NOT NULL,          -- Yellow/Pink/Blue/Green/Purple/Orange/White/Gray
    category TEXT NOT NULL,       -- General/Work/Personal/Ideas/Tasks/Important/Archive
    pinned INTEGER NOT NULL DEFAULT 0,
    archived INTEGER NOT NULL DEFAULT 0,
    created_at TEXT NOT NULL,     -- ISO 8601
    updated_at TEXT NOT NULL,     -- ISO 8601
    tags TEXT NOT NULL DEFAULT '[]',  -- JSON array
    position_x INTEGER NOT NULL DEFAULT 0,
    position_y INTEGER NOT NULL DEFAULT 0,
    width INTEGER NOT NULL DEFAULT 280,
    height INTEGER NOT NULL DEFAULT 200,
    device_id TEXT NOT NULL,
    sync_hash TEXT,               -- SHA-256
    deleted INTEGER NOT NULL DEFAULT 0   -- soft delete
);

-- Indexes
CREATE INDEX idx_notes_updated ON notes(updated_at);
CREATE INDEX idx_notes_category ON notes(category);
CREATE INDEX idx_notes_deleted ON notes(deleted);
```

---

## 7. Security

### SSH Communication

All inter-device communication (sync, file transfer, terminal) goes through encrypted SSH channels.

| Auth Method | Priority | Description |
|-------------|----------|-------------|
| SSH Key | 1 (recommended) | Most secure, no password needed |
| SSH Password | 2 | Password stored in local config file |

### Password Storage

- SSH passwords are AES-encrypted in `p2p.json` using the device_id as the encryption key
- The system password lock does not store passwords — verification is done via the system PAM mechanism each time

> **Security Note:** SSH key authentication is strongly recommended over password authentication. The SSH password encryption uses a static application salt with SHA-256 key derivation, which provides basic protection but is not suitable for high-security environments.

### Network Security

- FlyMode does not expose any external service ports (except TCP 4827 for pairing)
- SSH connections are always outbound (acting as a client)
- Supports Tailscale private networks (WireGuard encryption)

### Recommendations

1. **Use SSH key authentication** instead of passwords — more secure and convenient
2. **Enable Tailscale** to avoid exposing SSH port on public networks
3. **Enable password lock** (Settings > Require Password) to prevent unauthorized use
4. Keep FlyMode and system packages up to date

---

## 8. Troubleshooting

### Installation Issues

| Problem | Solution |
|---------|----------|
| `cargo tauri build` fails | Ensure all system dependencies are installed (libgtk, libwebkit, etc.) |
| `npm install` fails | Ensure Node.js >= 18 (`node -v`) |
| `flymode` command not found | Ensure `~/.local/bin` is in your PATH |

### P2P Connection Issues

| Problem | Solution |
|---------|----------|
| Device shows Offline | 1. Verify SSH server is running on the peer<br>2. Verify IP is correct<br>3. Check firewall allows port 22<br>4. Test manually: `ssh user@ip` |
| Pair request not received | 1. Check firewall allows port 4827<br>2. Ensure peer's FlyMode is running<br>3. Both on same network (or Tailscale) |
| Tailscale can't find devices | 1. Run `tailscale status` to verify both are online<br>2. Ensure both logged into the same Tailscale account |
| SSH connection fails | 1. Verify correct username<br>2. Verify correct key path or password<br>3. Debug: `ssh -v user@ip` |

### Sync Issues

| Problem | Solution |
|---------|----------|
| Notes not updating after sync | Ensure both sides have Trusted each other |
| Sync fails | Check sync history for error messages — usually SSH connection issues |
| My changes were overwritten | LWW uses timestamps. Avoid editing the same note on two machines simultaneously |

### OpenClaw / Terminal Issues

| Problem | Solution |
|---------|----------|
| No ">_" button | 1. Ensure OpenClaw is running (local: `pgrep -f openclaw`; remote: must be Trusted and Online)<br>2. Wait 120 seconds for the detection scan to complete |
| Local connection fails "No SSH key or password" | No SSH key on local machine → click ">_" to get a password prompt, enter your system password |
| Terminal connection fails | 1. Verify SSH is working (test sync first)<br>2. Ensure `openclaw` is installed and its path is discoverable |
| "openclaw not found" error | Ensure the `openclaw` binary is in PATH, or in `/usr/local/bin`, `/usr/bin`, `/opt`, etc. |
| CJK input duplicates | Update to the latest FlyMode version (fixed) |
| Cursor not visible | Update to the latest FlyMode version (fixed with WebGL renderer) |

### Wireless Control Issues

| Problem | Solution |
|---------|----------|
| WiFi toggle not working | Linux: Ensure `nmcli` is available (`which nmcli`) |
| Bluetooth toggle not working | Linux: Ensure `rfkill` is available (`which rfkill`) |
| Airplane mode not working | Linux: Ensure `rfkill` is available |

---

## 9. Technical Reference

### Backend Dependencies

| Crate | Version | Purpose |
|-------|---------|---------|
| `tauri` | 2.x | Desktop app framework |
| `tokio` | 1.x | Async runtime |
| `rusqlite` | 0.31 | SQLite database (bundled) |
| `ssh2` | 0.9 | SSH / SFTP communication |
| `serde` / `serde_json` | 1.x | JSON serialization |
| `chrono` | 0.4 | Time handling |
| `sha2` | 0.10 | sync_hash calculation |
| `crossbeam-channel` | 0.5 | Terminal I/O channel |
| `thiserror` | 1.x | Error type definitions |
| `tracing` | 0.1 | Logging |
| `uuid` | 1.x | UUID generation |
| `tauri-plugin-autostart` | 2.x | Auto-start on boot |
| `tauri-plugin-dialog` | 2.x | File dialogs |
| `tauri-plugin-notification` | 2.x | System notifications |

### Frontend Dependencies

| Package | Version | Purpose |
|---------|---------|---------|
| `preact` | 10.x | UI framework |
| `@tauri-apps/api` | 2.x | Tauri IPC |
| `@xterm/xterm` | 6.1.0-beta | Terminal emulator |
| `@xterm/addon-fit` | 0.11 | Terminal auto-resize |
| `@xterm/addon-webgl` | 0.19 | WebGL renderer |
| `vite` | 5.x | Build tool |
| `typescript` | 5.x | Type checking |

### IPC Commands

#### Configuration

| Command | Function |
|---------|----------|
| `get_config` | Load app settings |
| `save_config` | Save app settings |
| `get_build_info` | Get version and git hash |

#### Wireless Control

| Command | Function |
|---------|----------|
| `get_status` | Get WiFi/Bluetooth/Airplane Mode status |
| `toggle_wifi` | Toggle WiFi |
| `toggle_bluetooth` | Toggle Bluetooth |
| `toggle_airplane_mode` | Toggle Airplane Mode |
| `run_custom_command` | Execute custom command |

#### Scheduling

| Command | Function |
|---------|----------|
| `add_rule` | Create scheduling rule |
| `update_rule` | Modify rule |
| `delete_rule` | Delete rule |
| `toggle_rule` | Enable/disable rule |
| `execute_rule_now` | Execute rule immediately |

#### Notes

| Command | Function |
|---------|----------|
| `create_note` | Create note |
| `update_note` | Update note |
| `delete_note` | Delete note |
| `get_note` | Get single note |
| `list_notes` | List all notes |
| `search_notes` | Search notes |
| `get_note_colors` | Get available colors |
| `get_note_categories` | Get available categories |

#### Device Management

| Command | Function |
|---------|----------|
| `get_p2p_config` | Load P2P config |
| `save_p2p_config` | Save P2P config |
| `add_peer` | Add device |
| `remove_peer` | Remove device |
| `update_peer` | Update device |
| `check_peer_status` | Check single device status |
| `check_all_peers` | Check all device statuses |
| `discover_tailscale` | Tailscale auto-discovery |
| `get_device_id` | Get local device ID |
| `get_device_name` | Get local device name |

#### Pairing

| Command | Function |
|---------|----------|
| `pair_with_peer` | Send pairing request |
| `get_pending_pair_requests` | Get pending pair requests |
| `accept_pair_request` | Accept pairing |
| `reject_pair_request` | Reject pairing |

#### Sync

| Command | Function |
|---------|----------|
| `get_sync_state` | Get sync status |
| `sync_with_peer` | Sync with specific device |
| `sync_all_peers` | Sync with all trusted devices |
| `export_notes` | Export notes as JSON |
| `import_notes` | Import notes from JSON |
| `get_sync_folder` | Get sync folder path |

#### File Transfer

| Command | Function |
|---------|----------|
| `get_transfer_queue` | Get transfer queue |
| `upload_file` | Upload file |
| `download_file` | Download file |
| `cancel_transfer` | Cancel transfer |
| `clear_completed_transfers` | Clear completed transfers |
| `get_transfer_progress` | Get single transfer progress |
| `browse_remote_files` | Browse remote files |

#### Terminal

| Command | Function |
|---------|----------|
| `check_local_openclaw` | Detect local OpenClaw status (pgrep, no SSH) |
| `get_local_ssh_info` | Get local SSH username and key path |
| `check_openclaw_status` | Detect remote OpenClaw status |
| `open_terminal` | Open SSH PTY connection |
| `send_terminal_input` | Send keystrokes to terminal |
| `resize_terminal` | Resize terminal |
| `close_terminal` | Close terminal |

#### Authentication

| Command | Function |
|---------|----------|
| `verify_system_password` | Verify system password |

### Platform-Specific Implementation

#### Wireless Control

| Platform | WiFi | Bluetooth | Airplane Mode |
|----------|------|-----------|---------------|
| Linux | `nmcli radio wifi` | `rfkill block/unblock bluetooth` | `rfkill block/unblock all` |
| Windows | PowerShell network adapters | `Get-Service bthserv` | Not yet implemented |
| macOS | `networksetup -getairportpower` | `blueutil --power` | Not yet implemented |

#### System Password Verification

| Platform | Method |
|----------|--------|
| Linux | `/usr/sbin/unix_chkpwd` (PAM helper, setuid root) |
| Windows | Not yet implemented (planned: WinAPI `LogonUserA`) |
| macOS | Not yet implemented |

---

*Last updated: 2026-03-01*
*Version: v0.2.0*
