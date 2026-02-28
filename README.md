<p align="center">
  <img src="src-tauri/icons/128x128.png" alt="FlyMode" width="120" />
</p>

<h1 align="center">FlyMode</h1>

<p align="center">
  <strong>Your devices. Your data. No cloud required.</strong>
</p>

<p align="center">
  <a href="https://github.com/ken1413/flymode/releases/latest"><img src="https://img.shields.io/github/v/release/ken1413/flymode?style=flat-square&color=blue" alt="Release" /></a>
  <a href="https://github.com/ken1413/flymode/blob/main/LICENSE"><img src="https://img.shields.io/github/license/ken1413/flymode?style=flat-square" alt="License" /></a>
  <a href="https://github.com/ken1413/flymode/releases"><img src="https://img.shields.io/github/downloads/ken1413/flymode/total?style=flat-square&color=green" alt="Downloads" /></a>
</p>

<p align="center">
  <a href="./README.zh-TW.md">繁體中文</a> | <strong>English</strong>
</p>

---

FlyMode is a **fully decentralized** desktop application that connects your devices directly — no cloud, no central server, no subscription. Sync notes, transfer files, manage remote [OpenClaw](https://github.com/nicholasgasior/openclaw) nodes, and automate wireless controls, all through encrypted SSH tunnels between your own machines.

Built with **Rust + Tauri 2** for native performance and **Preact** for a lightweight, responsive UI.

---

## Why FlyMode?

| | Traditional Cloud Apps | FlyMode |
|---|---|---|
| **Data ownership** | Stored on someone else's server | Stays on YOUR devices |
| **Privacy** | Provider can access your data | End-to-end SSH encryption, zero third-party access |
| **Cost** | Monthly subscription fees | Free and open source, forever |
| **Network** | Requires internet connection | Works on LAN, Tailscale, or any network |
| **Control** | Vendor lock-in, ToS changes | You own the code, the data, everything |
| **Availability** | Service outages, shutdowns | Always available as long as your machines are on |

---

## Supported Platforms

| Platform | Status | Package Formats |
|----------|--------|-----------------|
| **Linux** (Ubuntu 20.04+, Debian 11+) | Fully supported | `.deb`, `.AppImage` |
| **Linux** (Fedora 36+) | Fully supported | `.rpm`, `.AppImage` |
| **Linux** (Arch, Manjaro) | Fully supported | `.AppImage`, build from source |
| **macOS** (12 Monterey+) | Build from source | — |
| **Windows** | Planned | — |

> FlyMode is currently Linux-first. macOS can be built from source. Windows support is on the roadmap.

### System Requirements

| Requirement | Minimum |
|-------------|---------|
| RAM | 256 MB |
| Disk | 100 MB (application) |
| Display | 1024 x 768 |
| Network | LAN, Tailscale VPN, or any IP-reachable network |

---

## Feature Overview

### 1. OpenClaw Remote Management

Manage all your [OpenClaw](https://github.com/nicholasgasior/openclaw) nodes from a single window. FlyMode automatically discovers OpenClaw Gateway instances running across your devices and provides one-click access to the OpenClaw TUI — no manual SSH required.

```
┌─────────────────────────────────────────────────────────────────┐
│  [● Home Server]  [● Office VPS]  [○ Cloud Node]           [x] │
├─────────────────────────────────────────────────────────────────┤
│                                                                 │
│   ╔═══════════════════════════════════════════════════════════╗  │
│   ║                   OpenClaw TUI v1.x                      ║  │
│   ║                                                          ║  │
│   ║   Node Status: Active                                    ║  │
│   ║   Peers: 12    Bandwidth: 1.2 GB/s                       ║  │
│   ║   Uptime: 14d 3h 22m                                     ║  │
│   ║                                                          ║  │
│   ╚═══════════════════════════════════════════════════════════╝  │
│                                                                 │
└─────────────────────────────────────────────────────────────────┘
```

#### How It Works

| Step | What Happens | Details |
|------|-------------|---------|
| **Auto-Detection** | FlyMode scans all trusted devices every 120 seconds | Detects running `openclaw-gateway` processes via SSH. When found, a `>_` button appears on the device card. |
| **One-Click Connect** | Click the `>_` button | FlyMode establishes an SSH PTY session, auto-locates the `openclaw` binary (via `which`, PATH search, and multi-directory `find` including symlinks), then launches the TUI with proper UTF-8 encoding. |
| **Multi-Node Tabs** | Switch between nodes instantly | Browser-style tab bar shows all OpenClaw-enabled devices. Each session runs independently — switching tabs doesn't disconnect or restart anything. |
| **Full Terminal** | Production-grade terminal experience | xterm-256color with WebGL GPU rendering, dynamic window resize, CJK input method support (fcitx5/iBus), clipboard integration (auto-copy on select, Ctrl+Shift+V to paste). |

#### Local Machine Support

FlyMode also detects OpenClaw running on the **local machine** (via `pgrep`, no SSH needed). If found, a `>_` button appears on the "This Device" card. Clicking it connects through SSH localhost — if no SSH key is configured, a password prompt appears automatically. The password is remembered for the current session.

#### Terminal Capabilities

| Capability | Details |
|------------|---------|
| Color support | Full xterm-256color (16.7M colors) |
| Rendering | WebGL GPU-accelerated (falls back to canvas) |
| Window resize | Automatic — terminal rows/columns adjust to window size in real-time |
| CJK input | Full fcitx5, iBus, and other IME support with zero duplicate characters |
| Clipboard | Select text to auto-copy; Ctrl+Shift+V to paste |
| Cursor | Blinking block cursor, clearly visible on any background |
| Encoding | UTF-8 (auto-set via `LANG` and `LC_ALL` environment variables) |
| Session persistence | Show/hide tabs without disconnecting — each session stays alive |

#### Use Cases

- **Multi-node monitoring** — Run OpenClaw on 5 machines? Open FlyMode, all 5 appear as tabs. Click to connect, click to switch. Monitor your entire OpenClaw network from one laptop.
- **Remote server management** — Connect to your office server or cloud VPS via Tailscale. Manage OpenClaw without remembering hostnames, IPs, or paths.
- **Mobile workflow** — On a laptop at a coffee shop? Connect back to your home NAS or office server and manage OpenClaw as if you were sitting in front of it.

---

### 2. P2P Device Sync & Management

Connect your devices directly without any cloud service. FlyMode uses a custom TCP pairing protocol and integrates with Tailscale for automatic device discovery.

#### Device Discovery & Pairing

| Method | How It Works |
|--------|-------------|
| **Tailscale auto-discovery** | Click "Discover Tailscale Peers" — FlyMode queries `tailscale status --json` and automatically finds all machines on your Tailscale network. No manual IP entry needed. |
| **Manual add** | Enter the remote machine's IP address, SSH user, and port. Works on any network where machines can reach each other. |
| **TCP pairing** | Machine A sends a pair request (TCP port 19131, configurable in `p2p.json`) → Machine B accepts → both machines are added to each other's device list with exchanged metadata. |

#### Trust Model

FlyMode has a two-level access model:

| Level | Capabilities |
|-------|-------------|
| **Paired (untrusted)** | See online/offline status only |
| **Trusted** | Full access: note sync, file transfer, remote terminal, OpenClaw management |

Both machines must trust each other for bidirectional sync.

#### Connection Types

| Icon | Type | Description |
|------|------|-------------|
| 🦎 | Tailscale | Via Tailscale VPN (WireGuard encrypted) |
| 🏠 | LAN Direct | Same local network |
| 🌐 | WAN Direct | Over the internet (ensure SSH port is accessible) |

#### Real-Time Status Monitoring

Every device shows its status with color-coded indicators, auto-refreshed every 30 seconds:

- 🟢 **Online** — SSH reachable, ready for sync/transfer
- 🔴 **Offline** — Unreachable
- ⚪ **Unknown** — Not yet checked

---

### 3. Sticky Notes with Cross-Device Sync

A full-featured note-taking system that automatically syncs across all your trusted devices.

#### Note Features

| Feature | Details |
|---------|---------|
| **Colors** | 8 options: Yellow, Pink, Blue, Green, Purple, Orange, White, Gray |
| **Categories** | 7 built-in: General, Work, Personal, Ideas, Tasks, Important, Archive |
| **Tags** | Custom tags (e.g., `#projectA`, `#urgent`) displayed on note cards |
| **Pin** | Pin important notes to the top of the list |
| **Search** | Full-text search across titles and content |
| **View modes** | Grid (card layout) or List (compact) — toggle with one click |
| **Soft delete** | Deleted notes aren't permanently removed. Deletion propagates via sync, and notes can be recovered. |

#### Sync Strategy

FlyMode uses **Last-Write-Wins (LWW)** conflict resolution:

- Each note has an `updated_at` timestamp and a `sync_hash` (SHA-256)
- When two devices modify the same note, the newer timestamp wins
- The `sync_hash` detects actual content changes — identical edits don't trigger unnecessary overwrites
- Notes that only exist on one device are automatically synced to all trusted peers
- Auto-sync runs at configurable intervals: 1 min, 5 min, 15 min, 30 min, or 1 hour

#### Export & Import

Export all notes as a JSON file for backup. Import from JSON to restore or transfer notes between machines without SSH.

---

### 4. Secure File Transfer (SFTP)

Transfer files directly between your devices using SFTP — encrypted, peer-to-peer, no middleman, no file size limits.

| Feature | Details |
|---------|---------|
| **Upload** | Select local files via native file picker → choose remote destination path → transfer starts |
| **Download** | Browse remote file system visually → click any file to download → choose local save path |
| **Remote file browser** | Navigate directories, see file names, sizes, modification dates. Click `..` to go up. |
| **Progress tracking** | Real-time progress bar with percentage and transfer speed for each file |
| **Queue management** | Up to 3 concurrent transfers. Queue additional files — they start automatically when slots open. |
| **Cancel** | Cancel any in-progress or queued transfer at any time |

Transfer states: `Pending` → `InProgress` → `Completed` (or `Failed` / `Cancelled`)

---

### 5. Wireless Scheduling & Quick Actions

Automate your wireless hardware on a schedule, or toggle them instantly.

#### Scheduling Rules

Create rules that run on a daily or weekly schedule:

| Field | Options |
|-------|---------|
| **Target** | WiFi, Bluetooth, Airplane Mode, or Custom Command |
| **Action** | Enable, Disable, Toggle, or Run Command |
| **Time** | Start time (HH:MM). Optional end time for time ranges. |
| **Days** | Select any combination of Monday through Sunday |

**Time range example:** Set WiFi to disable at 23:00 and re-enable at 07:00 — FlyMode handles overnight ranges automatically.

**Custom command example:** Run `sudo systemctl restart nginx` every day at 03:00.

Rules can be enabled/disabled without deleting, and manually triggered with "Execute Now".

#### Quick Actions

Three instant toggle buttons for WiFi, Bluetooth, and Airplane Mode — no schedule needed, just click.

Plus a command runner: type any shell command and execute it immediately with output displayed.

| Platform | WiFi | Bluetooth | Airplane Mode |
|----------|------|-----------|---------------|
| Linux | `nmcli` | `rfkill` | `rfkill` |
| macOS | `networksetup` | `blueutil` | Planned |
| Windows | Planned | Planned | Planned |

---

### 6. Security & Privacy

| Feature | Details |
|---------|---------|
| **All communication encrypted** | Every operation between devices (sync, file transfer, terminal) goes through SSH encrypted channels |
| **SSH key authentication** | Supports ed25519 and RSA keys. Recommended over password auth. |
| **Password authentication** | SSH passwords are AES-encrypted before storing in the local config file |
| **System password lock** | Require your OS login password to open FlyMode (Linux PAM via `unix_chkpwd`) |
| **System tray re-auth** | When restoring from tray after >1 second, re-authentication is required |
| **No external services** | FlyMode does not phone home. The only listening port is TCP 19131 for local pairing (customizable in `~/.config/flymode/p2p.json`). |
| **Tailscale compatible** | Combine with Tailscale for WireGuard-encrypted private networking |
| **Auto-start** | Optional launch at system boot with system tray background mode |

---

## Quick Install

### Pre-Built Package (Recommended)

Download and install in one command — **no Rust or Node.js required**:

```bash
curl -fsSL https://raw.githubusercontent.com/ken1413/flymode/main/install.sh | bash
```

The installer automatically detects your distro and installs the appropriate format (`.deb`, `.rpm`, or `.AppImage`).

For AppImage specifically (no sudo needed):

```bash
curl -fsSL https://raw.githubusercontent.com/ken1413/flymode/main/install.sh | bash -s -- --appimage
```

Then launch:

```bash
flymode
```

### Build from Source

For developers or unsupported distros. Requires **Rust 1.70+** and **Node.js 18+**:

```bash
curl -fsSL https://raw.githubusercontent.com/ken1413/flymode/main/setup.sh | bash
```

This script installs all dependencies (Rust, Node.js, system libraries), clones the repo, and builds from source.

Or manually:

```bash
git clone https://github.com/ken1413/flymode.git
cd flymode
cd src-ui && npm install && cd ..
cargo tauri build
# Binary at: target/release/flymode
```

<details>
<summary><strong>System dependencies for manual builds</strong></summary>

#### Ubuntu / Debian

```bash
sudo apt install build-essential curl wget git pkg-config \
    libgtk-3-dev libwebkit2gtk-4.1-dev \
    libayatana-appindicator3-dev librsvg2-dev patchelf \
    libssl-dev libsoup-3.0-dev libjavascriptcoregtk-4.1-dev
```

#### Fedora

```bash
sudo dnf install gcc gcc-c++ make curl wget git pkg-config \
    gtk3-devel webkit2gtk4.1-devel \
    libappindicator-gtk3-devel librsvg2-devel \
    openssl-devel libsoup3-devel javascriptcoregtk4.1-devel
```

#### Arch / Manjaro

```bash
sudo pacman -Syu --needed base-devel curl wget git pkg-config \
    gtk3 webkit2gtk-4.1 libappindicator-gtk3 librsvg patchelf \
    openssl libsoup3
```

#### macOS

```bash
xcode-select --install
```

</details>

---

## Required Software & Prerequisites

FlyMode itself is a single binary, but its P2P features depend on a few system services. Here's what you need:

### Required (for P2P features)

| Software | Why It's Needed | How to Install |
|----------|----------------|----------------|
| **SSH Server** (openssh-server) | All device communication (sync, file transfer, terminal) uses SSH. **Both machines** must have it. | Ubuntu: `sudo apt install openssh-server && sudo systemctl enable --now ssh`<br>Fedora: `sudo dnf install openssh-server && sudo systemctl enable --now sshd`<br>Arch: `sudo pacman -S openssh && sudo systemctl enable --now sshd`<br>macOS: System Settings → General → Sharing → Remote Login → On |
| **SSH Key or Password** | FlyMode authenticates via SSH to connect to peers. You need either an SSH key pair or the remote user's password. | Generate key: `ssh-keygen -t ed25519`<br>Copy to remote: `ssh-copy-id user@remote-ip` |

> **Note:** The `install.sh` installer automatically installs and enables SSH server. If you use the pre-built package, this is handled for you.

### Recommended (optional but highly useful)

| Software | Why It's Useful | How to Install |
|----------|-----------------|----------------|
| **Tailscale** | Lets devices on different networks (home, office, cloud) connect as if they're on the same LAN. Zero-config VPN with WireGuard encryption. FlyMode auto-discovers Tailscale peers. | `curl -fsSL https://tailscale.com/install.sh \| sh && sudo tailscale up` |

### Firewall Ports (if applicable)

If your machines have an active firewall, these ports need to be open:

| Port | Protocol | Purpose |
|------|----------|---------|
| **22** | TCP | SSH — all P2P communication (sync, transfer, terminal) |
| **19131** | TCP | FlyMode pairing protocol — device discovery and pairing requests |

```bash
# Ubuntu (ufw)
sudo ufw allow 22/tcp && sudo ufw allow 19131/tcp

# Fedora (firewalld)
sudo firewall-cmd --permanent --add-service=ssh
sudo firewall-cmd --permanent --add-port=19131/tcp
sudo firewall-cmd --reload
```

> If both machines are on the same LAN with no firewall, or both use Tailscale, no port configuration is needed.

### Not Required

| Software | Status |
|----------|--------|
| Cloud account | Not needed. FlyMode is fully decentralized. |
| Database server | Not needed. FlyMode uses embedded SQLite. |
| Docker | Not needed. FlyMode is a single native binary. |
| Internet connection | Not needed for LAN sync. Only needed for Tailscale or WAN connections. |

---

## Getting Started (Step by Step)

### Step 1: Install FlyMode on both machines

Use the [Quick Install](#quick-install) instructions above.

### Step 2: Connect your devices

**Option A — Tailscale auto-discovery (recommended for remote/cross-network):**

1. Install [Tailscale](https://tailscale.com) on both machines
2. Run `sudo tailscale up` and log in with the same account on both
3. In FlyMode → Devices tab → click **"Discover Tailscale Peers"**
4. The other machine appears automatically in your device list

**Option B — Manual add (for LAN or any directly reachable network):**

1. On the remote machine, find its IP: `hostname -I` (LAN) or `tailscale ip` (Tailscale)
2. In FlyMode → Devices tab → click **"Add Peer"**
3. Fill in: Name, IP Address, SSH Port (22), SSH User

### Step 3: Configure SSH credentials

Pairing only exchanges device metadata (name, IP). **You must manually configure SSH credentials:**

1. Click **"Edit"** on the peer's device card
2. Enter the **SSH User** (the remote machine's login username)
3. Choose authentication method:
   - **SSH Key Path** (recommended): e.g., `~/.ssh/id_ed25519` — requires prior `ssh-copy-id`
   - **SSH Password**: the remote user's login password
4. Save

> **Important:** Both machines must configure each other's SSH credentials. This is required for sync, file transfer, and terminal to work.

**Quick SSH key setup (recommended):**

```bash
# On Machine A:
ssh-keygen -t ed25519                    # Generate key (skip if you already have one)
ssh-copy-id youruser@machine-b-ip       # Copy public key to Machine B

# On Machine B:
ssh-keygen -t ed25519
ssh-copy-id youruser@machine-a-ip       # Copy public key to Machine A
```

### Step 4: Pair the devices

In the device list, click **"Pair"** on the peer device. The other machine's FlyMode will show an incoming pair request — click **"Accept"**.

### Step 5: Establish trust

Click the **"Trust"** button on the peer's device card. After trusting:

- Notes auto-sync in the background
- File transfer is enabled
- Remote terminal (including OpenClaw TUI) is available

> Both machines must trust each other for bidirectional sync.

### Step 6: Verify everything works

- **Sync tab** → Click "Sync Now" → verify notes appear on both machines
- **Transfer tab** → Upload a small test file → verify it arrives
- **Devices tab** → Status should show 🟢 Online
- **OpenClaw** → If running, the `>_` button should appear within 120 seconds

---

## Architecture

```
┌─────────────────────────────────────────────────────────────────┐
│                        FlyMode v0.3.0                           │
├─────────────────────────────────────────────────────────────────┤
│                                                                 │
│  ┌─────────────── Frontend (Preact + TypeScript) ─────────────┐ │
│  │                                                             │ │
│  │  📝 Notes  🔗 Devices  🔄 Sync  📤 Transfer                │ │
│  │  ⏰ Schedule  ⚡ Quick  ⚙️ Settings  🔒 Lock  >_ Terminal   │ │
│  │                                                             │ │
│  └────────────────────────┬────────────────────────────────────┘ │
│                           │ Tauri IPC (30+ commands)             │
│  ┌────────────────────────▼────────────────────────────────────┐ │
│  │              Backend (Rust + Tauri 2 + Tokio)               │ │
│  │                                                             │ │
│  │  ┌──────────┐ ┌──────────┐ ┌──────────┐ ┌──────────┐      │ │
│  │  │  Notes   │ │   P2P    │ │   Sync   │ │ Transfer │      │ │
│  │  │ (SQLite) │ │  (SSH2)  │ │ (Engine) │ │  (SFTP)  │      │ │
│  │  └──────────┘ └──────────┘ └──────────┘ └──────────┘      │ │
│  │  ┌──────────┐ ┌──────────┐ ┌──────────┐ ┌──────────┐      │ │
│  │  │Scheduler │ │ Wireless │ │ Terminal │ │  Config  │      │ │
│  │  │ (Cron)   │ │(nmcli/   │ │(SSH PTY) │ │  (JSON)  │      │ │
│  │  │          │ │ rfkill)  │ │          │ │          │      │ │
│  │  └──────────┘ └──────────┘ └──────────┘ └──────────┘      │ │
│  │                                                             │ │
│  └─────────────────────────────────────────────────────────────┘ │
│                                                                 │
│  ┌─────────────────────────────────────────────────────────────┐ │
│  │      System Tray + Auto-Start + TCP Pairing (port 19131)     │ │
│  └─────────────────────────────────────────────────────────────┘ │
│                                                                 │
└─────────────────────────────────────────────────────────────────┘
```

### Tech Stack

| Layer | Technology | Purpose |
|-------|-----------|---------|
| **Backend** | Rust, Tauri 2, Tokio | Native performance, async I/O, system integration |
| **Database** | SQLite (embedded, bundled) | Notes storage, full-text search, zero setup |
| **Networking** | SSH2, SFTP | Encrypted device communication, file transfer |
| **Frontend** | Preact, TypeScript, Vite | Lightweight UI (~50 KB gzipped) |
| **Terminal** | xterm.js + WebGL | GPU-accelerated terminal emulation |
| **VPN integration** | Tailscale | Cross-network device discovery |

### Data Storage

All data stays local on your machine:

| Data | Location | Format |
|------|----------|--------|
| App settings | `~/.config/flymode/config.json` | JSON |
| Device list & P2P config | `~/.config/flymode/p2p.json` | JSON |
| Notes database | `~/.local/share/flymode/notes.db` | SQLite |
| Sync working directory | `~/.local/share/flymode/sync/` | Files |

---

## Documentation

For detailed setup instructions, feature guides, and troubleshooting:

- **[Full User Guide (English)](./DOCUMENTATION.en.md)** — installation, setup, all features, troubleshooting, technical reference
- **[完整使用說明 (繁體中文)](./DOCUMENTATION.md)** — 安裝、設定、功能詳解、疑難排解、技術參考

---

## Development

```bash
# Install frontend dependencies (first time)
cd src-ui && npm install && cd ..

# Dev mode with hot reload (frontend + backend)
cargo tauri dev

# Run all Rust tests (150+ tests)
cd src-tauri && cargo test

# Run frontend tests
cd src-ui && npm test

# Production build
cargo tauri build

# Bump version (updates Cargo.toml, tauri.conf.json, package.json)
./bump-version.sh minor    # or: patch, major, 0.4.0
```

See [CLAUDE.md](./CLAUDE.md) for architecture details and development workflow.

---

## Contributing

Contributions are welcome! Please open an issue first to discuss what you'd like to change.

## License

[MIT](./LICENSE)
