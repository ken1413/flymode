# FlyMode

[繁體中文](./README.md) | **English**

A cross-platform desktop app with built-in **[OpenClaw](https://github.com/openclaw) remote management**, P2P device sync, sticky notes, file transfer, and wireless scheduling — all **fully decentralized** with no central server. Devices communicate directly via SSH.

## OpenClaw Integration

FlyMode provides deep integration with OpenClaw — automatically detects OpenClaw Gateway running on remote devices and lets you manage them through an embedded terminal with one click:

- **Auto-Detection**: Scans the local machine and all trusted devices every 120 seconds; shows a ">_" button when OpenClaw is found
- **One-Click Connect**: Click the button to SSH into the machine, auto-locate the `openclaw` binary, and launch its TUI
- **Full Terminal Experience**: xterm-256color, dynamic window resizing, CJK IME input support, clipboard integration
- **Multi-Device Tab Switching**: Browser-tab style multi-session terminal — switch between local and remote OpenClaw instances in a single window

## Features

| Feature | Description |
|---------|-------------|
| **OpenClaw Remote Management** | Auto-detect local and remote OpenClaw, one-click TUI launch, multi-device tab switching, CJK IME support |
| **Sticky Notes** | 8 colors, 7 categories, tags, pinning, full-text search, auto-sync across trusted devices |
| **P2P Device Management** | TCP pairing protocol, Tailscale auto-discovery, SSH key/password auth, trust model |
| **Data Sync** | Last-Write-Wins conflict resolution, auto/manual sync, JSON export/import |
| **File Transfer** | SFTP upload/download, remote file browser, queue management, progress bars, up to 3 concurrent |
| **Wireless Scheduling** | Scheduled WiFi / Bluetooth / Airplane Mode toggle, custom CLI commands |
| **Quick Actions** | Instant WiFi / Bluetooth / Airplane Mode toggle, run custom commands |
| **Security** | System password lock, system tray background mode, auto-start on boot |

## Quick Install

On Linux (Ubuntu/Fedora/Arch) or macOS:

```bash
curl -fsSL https://raw.githubusercontent.com/ken1413/flymode/main/setup.sh | bash
```

Then run:

```bash
flymode
```

## Post-Install Setup

The installer only handles compilation and installation. For P2P features, you also need:

1. **SSH Server** — required on both machines (the installer handles this automatically)
2. **Tailscale** (recommended) — install on both machines and log in with the same account for auto-discovery
3. **Firewall** — allow TCP port 4827 (pairing) and 22 (SSH)

See [DOCUMENTATION.en.md](./DOCUMENTATION.en.md) for detailed setup instructions.

## Manual Install

### Requirements

- Rust 1.70+, Node.js 18+
- Linux: GTK3, WebKit2GTK 4.1, OpenSSL

```bash
# Linux (Ubuntu/Debian) system dependencies
sudo apt install libgtk-3-dev libwebkit2gtk-4.1-dev \
    libayatana-appindicator3-dev librsvg2-dev patchelf \
    libssl-dev libsoup-3.0-dev libjavascriptcoregtk-4.1-dev

# Clone & Build
git clone https://github.com/ken1413/flymode.git
cd flymode/src-ui && npm install && cd ..
cargo tauri build

# Install binary
cp target/release/flymode ~/.local/bin/
```

## Development

```bash
cd src-ui && npm install && cd ..   # Install frontend dependencies
cargo tauri dev                      # Dev mode (hot reload)
cd src-tauri && cargo test           # Run tests
```

## Documentation

- [Full User Guide (English)](./DOCUMENTATION.en.md) — install, setup, features, troubleshooting
- [完整使用說明 (繁體中文)](./DOCUMENTATION.md)

## License

MIT
