# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

FlyMode is a cross-platform desktop app (Rust + Tauri 2 + Preact) combining wireless scheduling, P2P device sync, sticky notes, and file transfer — all without a central server.

## Build & Development Commands

```bash
# Install frontend dependencies (required first time)
cd src-ui && npm install && cd ..

# Development mode (hot reload for both frontend and backend)
cargo tauri dev

# Production build
cargo tauri build

# Run Rust tests
cd src-tauri && cargo test

# Run a single test
cd src-tauri && cargo test test_name

# Run tests for a specific module
cd src-tauri && cargo test notes::   # or p2p::, sync::, transfer::

# Frontend dev server only (no Tauri backend)
cd src-ui && npm run dev
```

### Linux System Dependencies

```bash
sudo apt install libgtk-3-dev libwebkit2gtk-4.0-dev \
                 libappindicator3-dev librsvg2-dev patchelf \
                 libssl-dev pkg-config
```

## Architecture

**Monorepo workspace** with two main directories:

- `src-tauri/` — Rust backend (Tauri 2, Tokio async runtime)
- `src-ui/` — TypeScript frontend (Preact, Vite)

Communication: Frontend ↔ Backend via **Tauri IPC** (`#[tauri::command]` handlers in `commands/mod.rs`, invoked from frontend with `@tauri-apps/api`).

### Backend Modules (`src-tauri/src/`)

| Module | Responsibility |
|--------|---------------|
| `main.rs` | App entry — initializes all state as `Arc<RwLock<T>>`, registers 30+ IPC commands, spawns scheduler and auto-sync tasks |
| `commands/` | All `#[tauri::command]` IPC handlers — thin layer that delegates to module logic |
| `notes/` | SQLite-backed CRUD with soft deletes, sync_hash for change detection, full-text search |
| `p2p/` | PeerDevice management, SSHClient (connect/execute/SFTP), Tailscale auto-discovery via `tailscale status --json` |
| `sync/` | SyncEngine — orchestrates note/file sync across peers using SSH, Last-Write-Wins conflict resolution (compare `updated_at` + `sync_hash`) |
| `transfer/` | TransferQueue with concurrent upload/download tracking via SFTP |
| `scheduler/` | Cron-based rule evaluation on a tokio interval, triggers wireless actions |
| `wireless/` | Platform-specific WiFi/Bluetooth/AirplaneMode control |
| `config/` | AppConfig and P2PConfig serialization (JSON files in `~/.config/flymode/`) |

### Frontend Components (`src-ui/src/`)

Tab-based UI in `App.tsx`. Each tab is a self-contained component in `components/`:
`NotesTab`, `P2PTab`, `SyncTab`, `TransferTab`, `RulesTab`, `QuickActionsTab`, `SettingsTab`

## Key Patterns

- **State management**: All backend state is `Arc<RwLock<T>>`, injected into IPC handlers via Tauri's `State<'_, T>`
- **Cross-platform**: Platform-specific code uses `#[cfg(target_os = "linux|windows|macos")]` — Linux uses nmcli/rfkill/zbus, Windows uses Win32 APIs via PowerShell, macOS uses IOKit/CoreFoundation
- **Database**: SQLite bundled via `rusqlite` (feature `bundled`). Schema initialized in `NotesStore::init_db()`. Indexes on `updated_at`, `category`, `deleted`
- **Sync strategy**: Optimistic sync with Last-Write-Wins — `sync_hash` detects changes, `updated_at` breaks ties
- **Soft deletes**: Notes use a `deleted` boolean flag, never hard-deleted
- **SSH**: `ssh2` crate with vendored OpenSSL for P2P communication and SFTP file transfer
- **Error types**: Each module defines its own error enum with `thiserror`

## Data Locations

| Data | Path |
|------|------|
| App config | `~/.config/flymode/config.json` |
| P2P config | `~/.config/flymode/p2p.json` |
| Notes DB | `~/.local/share/flymode/notes.db` |
| Sync folder | `~/.local/share/flymode/sync/` |

## Testing

- Unit tests: inline `#[cfg(test)]` modules within each backend module
- Integration tests: `src-tauri/tests/integration_test.rs`
- Test helpers: `test_utils/` module with `TestContext` (temp dirs), `create_test_note()`, `create_test_peer()`
- Dev dependencies: `mockall` (mocking), `tempfile`, `pretty_assertions`, `proptest` (property-based), `criterion` (benchmarks)
- Feature flag `test-helpers` available for conditional test code

## Language Note

README, DOCUMENTATION.md, and code comments are in Traditional Chinese (繁體中文).
