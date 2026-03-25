# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

timewaver-monitor is a Windows system tray application (Rust) that monitors the TimeWaver Pro process and auto-restarts it if it stops. It lives as a subcrate in the flymode monorepo workspace.

## Build & Run

```bash
# Build (from this directory)
cargo build

# Run
cargo run

# Release build (stripped, LTO)
cargo build --release
```

There are no tests currently defined in this crate.

## Architecture

Single-binary Rust app with four modules:

| Module | Role |
|--------|------|
| `main.rs` | Entry point — initializes logging, spawns monitor thread, runs winit event loop for tray |
| `config.rs` | JSON config loaded from `%APPDATA%/timewaver-monitor/config.json`. Defines defaults for target exe path, check interval, restart limits |
| `monitor.rs` | Background thread that polls `sysinfo` to check if the target process is alive; restarts it via `std::process::Command` with cooldown/retry logic |
| `tray.rs` | System tray UI via `tray-icon` + `muda` crates — status display, pause/resume, manual restart, auto-start toggle, open log/config |
| `startup.rs` | Windows registry auto-start (`HKCU\...\Run`), no-op on non-Windows |

## Key Design Details

- **Event loop**: Uses `winit` event loop with `about_to_wait` polling every 500ms. Tray UI updates every 2s.
- **State sharing**: `Config` and `MonitorStatus` are `Arc<Mutex<T>>`, shared between the UI thread and monitor thread.
- **Process detection**: `sysinfo` crate, matches by executable filename then full path (case-insensitive).
- **Restart safety**: Max consecutive restart attempts (`max_restart_attempts`, default 5) before entering a cooldown period (`restart_cooldown_secs`, default 300s).
- **Tray icon**: Procedurally generated 16x16 circle — green when running, red when stopped.
- **Windows-only features**: Auto-start via registry (`startup.rs`), `explorer`/`notepad` for open actions. Non-Windows compiles but auto-start is a no-op.
- **Logging**: `tracing` + `tracing-appender` with daily rolling log files in `%LOCALAPPDATA%/timewaver-monitor/logs/`.

## Data Locations (Windows)

| Data | Path |
|------|------|
| Config | `%APPDATA%/timewaver-monitor/config.json` |
| Logs | `%LOCALAPPDATA%/timewaver-monitor/logs/` |

## Git Commit Rules

- Never include `Co-Authored-By` lines in commit messages.

## Default Monitored Process

`C:\Program Files (x86)\TimeWaverPro\TwPro.exe` — configurable via `config.json`.
