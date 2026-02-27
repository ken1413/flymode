# FlyMode

**繁體中文** | [English](./README.en.md)

跨平台桌面應用程式，整合無線控制、P2P 裝置同步、便利貼筆記、檔案傳輸、遠端終端機五大功能。
**完全去中心化** — 不需要任何伺服器，裝置間透過 SSH 直接通訊。

## 功能一覽

| 功能 | 說明 |
|------|------|
| **便利貼筆記** | 8 種顏色、7 種類別、標籤、釘選、全文搜尋，自動同步到所有信任裝置 |
| **P2P 裝置管理** | TCP 配對協議、Tailscale 自動發現、SSH 金鑰/密碼認證、信任機制 |
| **資料同步** | Last-Write-Wins 衝突解決、自動/手動同步、匯出/匯入 JSON |
| **檔案傳輸** | SFTP 上傳/下載、遠端檔案瀏覽器、佇列管理、進度條、最多 3 筆並行 |
| **遠端終端機** | SSH PTY 連線到遠端 OpenClaw TUI，內建中文 IME 支援、剪貼簿 |
| **無線排程** | 定時開關 WiFi / 藍牙 / 飛航模式，支援自定義 CLI 命令 |
| **快速操作** | 即時切換 WiFi / 藍牙 / 飛航模式、執行自定義命令 |
| **系統安全** | 系統密碼鎖定、系統匣背景執行、開機自動啟動 |

## 一鍵安裝

在 Linux (Ubuntu/Fedora/Arch) 或 macOS 上執行：

```bash
curl -fsSL https://raw.githubusercontent.com/ken1413/flymode/main/setup.sh | bash
```

安裝完成後執行：

```bash
flymode
```

> **注意：** 安裝過程中會自動安裝 GitHub CLI (`gh`) 並提示登入 GitHub 帳號。

## 安裝後必要設定

安裝程式只處理編譯和安裝。要使用 P2P 功能，還需要：

1. **SSH Server** — 兩台電腦都需要安裝（安裝腳本會自動處理）
2. **Tailscale**（建議）— 在兩台電腦上安裝並登入同一帳號，即可自動發現裝置
3. **防火牆** — 開放 TCP 4827（配對用）和 22（SSH 用）

詳細設定請參閱：[DOCUMENTATION.md](./DOCUMENTATION.md)

## 手動安裝

### 系統需求

- Rust 1.70+、Node.js 18+
- Linux: GTK3, WebKit2GTK 4.1, OpenSSL

```bash
# Linux (Ubuntu/Debian) 系統依賴
sudo apt install libgtk-3-dev libwebkit2gtk-4.1-dev \
    libayatana-appindicator3-dev librsvg2-dev patchelf \
    libssl-dev libsoup-3.0-dev libjavascriptcoregtk-4.1-dev

# Clone & Build
git clone https://github.com/ken1413/flymode.git
cd flymode/src-ui && npm install && cd ..
cargo tauri build

# 安裝 binary
cp target/release/flymode ~/.local/bin/
```

## 開發

```bash
cd src-ui && npm install && cd ..   # 安裝前端依賴
cargo tauri dev                      # 開發模式（hot reload）
cd src-tauri && cargo test           # 執行測試
```

## 文件

- [完整使用說明 (繁體中文)](./DOCUMENTATION.md) — 安裝、設定、功能詳解、疑難排解
- [Full User Guide (English)](./DOCUMENTATION.en.md)

## 授權

MIT
