# FlyMode

**繁體中文** | [English](./README.md)

跨平台桌面應用程式，內建 **[OpenClaw](https://github.com/openclaw) 遠端管理**、P2P 裝置同步、便利貼筆記、檔案傳輸、無線排程等功能。
**完全去中心化** — 不需要任何伺服器，裝置間透過 SSH 直接通訊。

## OpenClaw 整合

FlyMode 深度整合 OpenClaw — 自動偵測遠端裝置上的 OpenClaw Gateway，一鍵開啟內嵌式終端機直接操作 OpenClaw TUI：

- **自動偵測**：每 120 秒掃描本機和所有信任裝置，偵測到 OpenClaw 即顯示「>_」按鈕
- **一鍵連線**：點擊按鈕即透過 SSH PTY 連線，自動定位 `openclaw` 路徑並啟動 TUI
- **完整終端體驗**：xterm-256color、動態視窗縮放、中文/日文 IME 輸入、剪貼簿整合
- **多裝置分頁切換**：瀏覽器分頁風格的多 session 終端，在同一個視窗中快速切換本機和遠端的 OpenClaw

## 功能一覽

| 功能 | 說明 |
|------|------|
| **OpenClaw 遠端管理** | 自動偵測本機和遠端 OpenClaw、一鍵開啟 TUI、多裝置分頁切換、中文 IME 支援 |
| **便利貼筆記** | 8 種顏色、7 種類別、標籤、釘選、全文搜尋，自動同步到所有信任裝置 |
| **P2P 裝置管理** | TCP 配對協議、Tailscale 自動發現、SSH 金鑰/密碼認證、信任機制 |
| **資料同步** | Last-Write-Wins 衝突解決、自動/手動同步、匯出/匯入 JSON |
| **檔案傳輸** | SFTP 上傳/下載、遠端檔案瀏覽器、佇列管理、進度條、最多 3 筆並行 |
| **無線排程** | 定時開關 WiFi / 藍牙 / 飛航模式，支援自定義 CLI 命令 |
| **快速操作** | 即時切換 WiFi / 藍牙 / 飛航模式、執行自定義命令 |
| **系統安全** | 系統密碼鎖定、系統匣背景執行、開機自動啟動 |

## 安裝

### 一般使用者（推薦）

下載預編譯套件，不需要 Rust/Node 環境：

```bash
curl -fsSL https://raw.githubusercontent.com/ken1413/flymode/main/install.sh | bash
```

或使用 AppImage（免 sudo）：

```bash
curl -fsSL https://raw.githubusercontent.com/ken1413/flymode/main/install.sh | bash -s -- --appimage
```

安裝完成後執行：

```bash
flymode
```

### 從原始碼建置（開發者）

需要 Rust 1.70+、Node.js 18+ 環境：

```bash
curl -fsSL https://raw.githubusercontent.com/ken1413/flymode/main/setup.sh | bash
```

> **注意：** 從原始碼建置需要安裝 GitHub CLI (`gh`) 並登入 GitHub 帳號。

## 安裝後必要設定

要使用 P2P 功能，還需要：

1. **SSH Server** — 兩台電腦都需要安裝（安裝腳本會自動處理）
2. **Tailscale**（建議）— 在兩台電腦上安裝並登入同一帳號，即可自動發現裝置
3. **防火牆** — 開放 TCP 4827（配對用）和 22（SSH 用）

詳細設定請參閱：[DOCUMENTATION.md](./DOCUMENTATION.md)

## 開發

```bash
cd src-ui && npm install && cd ..   # 安裝前端依賴
cargo tauri dev                      # 開發模式（hot reload）
cd src-tauri && cargo test           # 執行測試
./bump-version.sh minor              # 版本升級
```

## 文件

- [完整使用說明 (繁體中文)](./DOCUMENTATION.md) — 安裝、設定、功能詳解、疑難排解
- [Full User Guide (English)](./DOCUMENTATION.en.md)

## 授權

MIT
