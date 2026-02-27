# FlyMode

跨平台桌面應用程式，整合無線控制、P2P 同步、便利貼筆記三大功能。

## 功能

### 無線排程
- 定時開關 WiFi / 藍牙 / 飛航模式
- 自定義 CLI 命令執行
- 系統匣背景執行

### P2P 同步
- 透過 Tailscale/SSH 在裝置間同步
- 無需中央伺服器
- 自動發現 Tailscale 裝置

### 便利貼筆記
- 8 種顏色、7 種類別
- 標籤系統、釘選功能
- 自動同步到信任裝置

### 檔案傳輸
- 裝置間直接傳送檔案
- 瀏覽遠端檔案系統
- 傳輸佇列管理

## 快速開始

```bash
# 安裝前端依賴
cd src-ui && npm install

# 開發模式
cd .. && cargo tauri dev

# 生產建置
cargo tauri build
```

## 系統需求

- Rust 1.70+
- Node.js 18+
- Tailscale (可選，用於自動發現)

## 文件

詳細技術文件：[DOCUMENTATION.md](./DOCUMENTATION.md)

## 授權

MIT
