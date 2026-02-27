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

## 一鍵安裝

在任何 Linux (Ubuntu/Fedora/Arch) 或 macOS 電腦上執行：

```bash
curl -fsSL https://gist.githubusercontent.com/ken1413/756e1cd8131583561c138a33cc401984/raw/setup.sh | bash
```

> **注意：** 此為私有 repo，安裝過程中會自動安裝 GitHub CLI (`gh`) 並提示登入 GitHub 帳號以取得 clone 權限。

安裝腳本會自動處理：
- 系統依賴（GTK、WebKit、OpenSSL 等）
- Rust 工具鏈 + Node.js 22 LTS
- GitHub CLI（認證 clone 私有 repo）
- Tauri CLI
- Clone、編譯、安裝 binary 到 `~/.local/bin/flymode`
- 建立桌面捷徑（Linux）

安裝完成後直接執行：

```bash
flymode
```

## 手動安裝

### 系統需求

- Rust 1.70+
- Node.js 18+
- Tailscale（可選，用於自動發現）

### Linux 系統依賴

```bash
sudo apt install libgtk-3-dev libwebkit2gtk-4.1-dev \
    libappindicator3-dev librsvg2-dev patchelf \
    libssl-dev libsoup-3.0-dev libjavascriptcoregtk-4.1-dev
```

### 建置步驟

```bash
git clone https://github.com/ken1413/flymode.git
cd flymode/src-ui && npm install && cd ..
cargo tauri build
```

## 開發

```bash
# 安裝前端依賴
cd src-ui && npm install

# 開發模式（hot reload）
cd .. && cargo tauri dev

# 執行測試
cd src-tauri && cargo test
```

## 文件

詳細技術文件：[DOCUMENTATION.md](./DOCUMENTATION.md)

## 授權

MIT
