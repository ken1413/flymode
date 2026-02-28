# FlyMode 使用說明

跨平台桌面應用程式 — 無線控制 + P2P 裝置同步 + 便利貼 + 檔案傳輸 + 遠端終端機

版本：v0.3.0 | 最後更新：2026-03-01

---

## 目錄

1. [安裝](#1-安裝)
2. [安裝後設定](#2-安裝後設定)
3. [兩台電腦配對教學](#3-兩台電腦配對教學)
4. [功能說明](#4-功能說明)
   - [便利貼筆記](#41-便利貼筆記-📝)
   - [裝置管理](#42-裝置管理-🔗)
   - [資料同步](#43-資料同步-🔄)
   - [檔案傳輸](#44-檔案傳輸-📤)
   - [OpenClaw 遠端管理](#45-openclaw-遠端管理)
   - [無線排程](#46-無線排程-⏰)
   - [快速操作](#47-快速操作-⚡)
   - [設定](#48-設定-⚙️)
5. [系統架構](#5-系統架構)
6. [資料存儲](#6-資料存儲)
7. [安全性](#7-安全性)
8. [疑難排解](#8-疑難排解)
9. [技術參考](#9-技術參考)

---

## 1. 安裝

### 1.1 一鍵安裝（建議）

在 Linux (Ubuntu/Fedora/Arch) 或 macOS 上執行：

```bash
curl -fsSL https://raw.githubusercontent.com/ken1413/flymode/main/setup.sh | bash
```

安裝腳本會自動處理：

| 步驟 | 說明 |
|------|------|
| 系統依賴 | GTK3, WebKit2GTK, OpenSSL, pkg-config 等 |
| Rust 工具鏈 | 自動安裝 `rustup` + stable toolchain |
| Node.js 22 LTS | 前端建置需要 |
| GitHub CLI (`gh`) | 驗證帳號以 clone repo |
| Tauri CLI | `cargo install tauri-cli` |
| SSH Server | `openssh-server`，自動啟動 |
| 編譯 | `cargo tauri build`（release 模式） |
| 安裝 | binary 放到 `~/.local/bin/flymode` |
| 桌面捷徑 | Linux 建立 `.desktop` 文件 |

> **注意：** 首次安裝需要 GitHub 帳號認證。安裝過程中 `gh auth login` 會引導你登入。

安裝完成後啟動：

```bash
flymode
```

### 1.2 手動安裝

#### 系統需求

| 工具 | 最低版本 | 說明 |
|------|---------|------|
| Rust | 1.70+ | 後端編譯 |
| Node.js | 18+ | 前端建置 |
| npm | 9+ | 套件管理 |

#### Linux (Ubuntu/Debian) 系統依賴

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
# 安裝 Xcode 命令列工具
xcode-select --install

# 安裝 Homebrew（若尚未安裝）
/bin/bash -c "$(curl -fsSL https://raw.githubusercontent.com/Homebrew/install/HEAD/install.sh)"
```

#### 編譯與安裝

```bash
# Clone
git clone https://github.com/ken1413/flymode.git
cd flymode

# 安裝前端依賴
cd src-ui && npm install && cd ..

# 編譯
cargo tauri build

# 安裝到 PATH
mkdir -p ~/.local/bin
cp target/release/flymode ~/.local/bin/
```

### 1.3 更新

重新執行一鍵安裝腳本即可。已有的 repo 會 `git pull`，然後重新編譯：

```bash
curl -fsSL https://raw.githubusercontent.com/ken1413/flymode/main/setup.sh | bash
```

或手動更新：

```bash
cd ~/app/flymode
git pull
cd src-ui && npm install && cd ..
cargo tauri build
cp target/release/flymode ~/.local/bin/
```

---

## 2. 安裝後設定

安裝程式只處理編譯和安裝。要使用 P2P 同步和檔案傳輸功能，需要額外設定。

### 2.1 SSH Server（必要）

兩台要互相通訊的電腦**都**需要 SSH Server。安裝腳本會自動處理，但若手動安裝則需自行設定：

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

macOS: 系統設定 → 一般 → 共享 → 遠端登入 → 開啟

#### SSH 金鑰認證（建議）

使用金鑰認證可免除每次輸入密碼：

```bash
# 在本機產生金鑰（若尚未有）
ssh-keygen -t ed25519

# 複製公鑰到遠端電腦
ssh-copy-id user@remote-ip
```

完成後，FlyMode 會自動使用 `~/.ssh/id_ed25519` 或 `~/.ssh/id_rsa` 連線。

### 2.2 Tailscale（建議）

Tailscale 讓兩台不在同一個區域網路的電腦也能直接連線（如家裡和辦公室）。

```bash
# 安裝 Tailscale
curl -fsSL https://tailscale.com/install.sh | sh

# 啟動並登入
sudo tailscale up
```

兩台電腦登入同一個 Tailscale 帳號後，FlyMode 會**自動發現**對方。

### 2.3 防火牆（必要時）

FlyMode 需要以下 port：

| Port | 協議 | 用途 |
|------|------|------|
| 19131 | TCP | 裝置配對請求（可在 `~/.config/flymode/p2p.json` 自訂）|
| 22 | TCP | SSH 連線（同步、傳輸、終端機）|

```bash
# Ubuntu (ufw)
sudo ufw allow 19131/tcp
sudo ufw allow 22/tcp

# Fedora (firewalld)
sudo firewall-cmd --permanent --add-port=19131/tcp
sudo firewall-cmd --permanent --add-service=ssh
sudo firewall-cmd --reload
```

> 若兩台電腦都在同一個區域網路且沒有防火牆限制，通常不需要額外設定。
> 若使用 Tailscale，則 Tailscale 網路內部不受防火牆限制。

---

## 3. 兩台電腦配對教學

以下假設：
- **電腦 A**：已安裝 FlyMode
- **電腦 B**：已安裝 FlyMode
- 兩台都已安裝 SSH Server
- 兩台在同一個 Tailscale 網路（或區域網路）

### 步驟 1：取得對方 IP

**方法 A — Tailscale 自動發現（建議）：**

在電腦 A 的 FlyMode → 🔗 裝置頁面 → 點「Discover Tailscale Peers」按鈕。
電腦 B 會自動出現在裝置列表中。

**方法 B — 手動新增：**

先在電腦 B 確認 IP 位址：

```bash
# Tailscale IP
tailscale ip

# 或區域網路 IP
hostname -I
```

在電腦 A 的 FlyMode → 🔗 裝置頁面 → 點「Add Peer」，填入：
- **Name**：自訂名稱（如「辦公室電腦」）
- **IP Address**：電腦 B 的 IP
- **SSH User**：電腦 B 的使用者名稱
- **SSH Port**：22（預設）
- **認證方式**：SSH 金鑰路徑 或 密碼

### 步驟 2：配對

在裝置列表中找到電腦 B → 點「Pair」按鈕。

此時電腦 B 的 FlyMode 會收到配對請求。在電腦 B 的 🔗 裝置頁面 → 「Incoming Pair Requests」區塊 → 點「Accept」。

配對完成後，兩台電腦互相出現在裝置列表中。

### 步驟 3：設定 SSH 認證（必要）

配對只交換了裝置資訊（IP、名稱），**不會自動設定 SSH 連線帳密**。你必須手動設定：

1. 在裝置列表中，點對方裝置的「Edit」按鈕
2. 填入 **SSH User**（對方電腦的使用者名稱）
3. 填入認證方式（二擇一）：
   - **SSH Key Path**：如 `~/.ssh/id_ed25519`（建議，需先做 `ssh-copy-id`）
   - **SSH Password**：對方電腦的登入密碼
4. 儲存

> **重要：** 兩台電腦都需要各自設定對方的 SSH 帳密。如果未設定，同步、檔案傳輸、終端機等功能都會因 SSH 連線失敗而無法使用。

設定完成後，裝置狀態應顯示為 **Online**（綠色圓點）。若仍為 Offline，請參考[疑難排解](#8-疑難排解)。

### 步驟 4：建立信任

在裝置列表中，點對方裝置的「Trust」按鈕。標記為信任後：
- 自動同步筆記到該裝置
- 可以傳輸檔案
- 可以開啟遠端終端機

> **兩台電腦都需要互相 Trust 對方，才能雙向同步。**

### 步驟 5：驗證連線

- 🔄 同步頁面：點「Sync Now」手動同步，確認筆記同步成功
- 📤 傳輸頁面：嘗試上傳一個小檔案到對方
- 狀態列應顯示對方為 Online（綠色圓點）

---

## 4. 功能說明

### 4.1 便利貼筆記 📝

#### 基本操作

| 操作 | 說明 |
|------|------|
| 建立筆記 | 點「+ New Note」，輸入標題和內容 |
| 編輯筆記 | 點筆記卡片上的「Edit」按鈕 |
| 刪除筆記 | 點「Delete」按鈕（軟刪除，可透過同步恢復）|
| 釘選筆記 | 點「Pin」按鈕，釘選的筆記會顯示在最上方 |
| 搜尋 | 在搜尋欄輸入關鍵字，搜尋標題和內容 |

#### 顏色

| 顏色 | 色碼 | 建議用途 |
|------|------|----------|
| 黃色 Yellow | `#fef08a` | 一般筆記 |
| 粉色 Pink | `#fbcfe8` | 個人/心情 |
| 藍色 Blue | `#bfdbfe` | 工作/專案 |
| 綠色 Green | `#bbf7d0` | 已完成事項 |
| 紫色 Purple | `#e9d5ff` | 創意點子 |
| 橙色 Orange | `#fed7aa` | 待辦事項 |
| 白色 White | `#ffffff` | 簡潔筆記 |
| 灰色 Gray | `#e5e7eb` | 封存/參考 |

#### 類別

| 類別 | 說明 |
|------|------|
| General | 一般筆記 |
| Work | 工作相關 |
| Personal | 個人事物 |
| Ideas | 創意點子 |
| Tasks | 待辦任務 |
| Important | 重要事項 |
| Archive | 封存筆記 |

#### 標籤

在編輯筆記時可加入標籤（如 `#專案A`、`#待辦`）。標籤會顯示在筆記卡片上。

#### 顯示模式

點右上角的切換按鈕，可在**格狀（Grid）**和**列表（List）**模式間切換。

#### 同步行為

- 筆記每 5 秒自動載入最新狀態
- 若已設定信任裝置且開啟自動同步，筆記會在背景同步到所有信任裝置
- 刪除的筆記不會真正從資料庫移除（軟刪除），同步時會把刪除狀態傳播到其他裝置

---

### 4.2 裝置管理 🔗

#### 裝置資訊

頁面頂部顯示本機資訊：

- **Device ID**：本機的唯一識別碼（UUID）
- **Device Name**：本機的主機名稱
- **Listen Port**：TCP 配對服務監聽的 port（預設 19131，可在 `~/.config/flymode/p2p.json` 自訂）

#### 新增裝置

| 欄位 | 說明 | 範例 |
|------|------|------|
| Name | 自訂名稱 | 辦公室電腦 |
| Hostname | 對方主機名稱 | my-desktop |
| IP Address | 對方 IP | 100.64.0.2 |
| SSH Port | SSH 端口 | 22 |
| SSH User | SSH 使用者 | alice |
| Connection Type | 連線類型 | Tailscale / LAN Direct / WAN Direct |
| SSH Key Path | 金鑰路徑（選填）| ~/.ssh/id_ed25519 |
| SSH Password | 密碼（選填）| 若不使用金鑰認證 |

#### 連線類型標示

| 標示 | 類型 | 說明 |
|------|------|------|
| 🦎 | Tailscale | 透過 Tailscale VPN 連線 |
| 🏠 | LAN Direct | 區域網路直連 |
| 🌐 | WAN Direct | 廣域網路直連 |

#### 裝置狀態

| 狀態 | 說明 |
|------|------|
| 🟢 Online | 連線正常（SSH 可達）|
| 🔴 Offline | 無法連線 |
| ⚪ Unknown | 尚未檢查 |

狀態每 30 秒自動更新一次。

#### Tailscale 自動發現

點「Discover Tailscale Peers」按鈕，FlyMode 會執行 `tailscale status --json` 取得同一 Tailscale 網路中所有裝置的資訊，自動加入裝置列表。

#### TCP 配對協議

FlyMode 內建 TCP 配對服務（預設 port 19131，可在 `p2p.json` 自訂）：

1. **電腦 A** 點「Pair」→ 發送配對請求到電腦 B
2. **電腦 B** 收到請求 → 在「Incoming Pair Requests」區塊顯示
3. **電腦 B** 點「Accept」→ 雙方互加對方到裝置列表
4. 配對完成後，雙方自動連線並標記為已連接

#### 信任機制

- **未信任裝置**：只能看到在線狀態，無法同步或傳輸
- **已信任裝置**：可自動同步筆記、傳輸檔案、開啟遠端終端機
- 點裝置卡片上的「Trust / Untrust」按鈕切換

#### OpenClaw 偵測

FlyMode 會偵測本機和遠端裝置上的 OpenClaw：

- **本機**：使用 `pgrep`（不需 SSH），若偵測到則「This Device」卡片顯示「>_」按鈕
- **遠端裝置**：透過 SSH 執行 `pgrep -f openclaw`，偵測到則裝置卡片顯示「>_」按鈕

偵測每 120 秒自動執行一次。

---

### 4.3 資料同步 🔄

#### 同步策略

FlyMode 使用 **Last-Write-Wins (LWW)** 衝突解決策略：

- 兩台裝置同時修改同一筆記時，以 `updated_at` 時間戳較新的為準
- 使用 `sync_hash`（SHA-256）偵測實際變更，避免不必要的覆蓋
- 只存在一端的筆記會直接同步到另一端
- 刪除狀態也會同步（軟刪除）

#### 自動同步

| 設定 | 說明 |
|------|------|
| 開啟/關閉 | 🔄 同步頁面上的 Auto-Sync 開關 |
| 同步間隔 | 可選 1 分鐘、5 分鐘、15 分鐘、30 分鐘、1 小時 |
| 同步對象 | 所有已信任且在線的裝置 |

#### 手動操作

| 操作 | 說明 |
|------|------|
| Sync Now | 立即同步所有信任裝置 |
| Sync with Peer | 只同步指定裝置 |

#### 同步紀錄

頁面下方顯示最近 10 次同步結果：

- 對方裝置名稱
- 同步狀態（成功/失敗）
- 同步筆記數量
- 花費時間
- 錯誤訊息（若有）

#### 匯出 / 匯入

| 操作 | 說明 |
|------|------|
| Export Notes | 匯出所有筆記為 JSON 檔案 |
| Import Notes | 從 JSON 檔案匯入筆記 |

可用於備份還原，或在不方便 SSH 連線時手動交換筆記。

---

### 4.4 檔案傳輸 📤

#### 上傳檔案

1. 選擇目標裝置（必須已信任、在線）
2. 點「Upload」按鈕 → 系統檔案選擇器
3. 選擇要上傳的檔案
4. 輸入遠端目標路徑（預設為對方家目錄）
5. 傳輸開始，顯示進度條

#### 下載檔案

1. 選擇來源裝置
2. 瀏覽遠端檔案系統（可點目錄進入）
3. 點目標檔案的「Download」按鈕
4. 選擇本地存放路徑
5. 傳輸開始，顯示進度條

#### 遠端檔案瀏覽器

- 顯示檔案名稱、大小、最後修改時間
- 目錄可點擊進入
- 支援 `..` 回到上層目錄

#### 傳輸管理

| 功能 | 說明 |
|------|------|
| 進度條 | 即時顯示百分比和傳輸速度 |
| 取消 | 可取消進行中的傳輸 |
| 清除完成 | 清除已完成的傳輸紀錄 |
| 並行限制 | 最多同時 3 筆傳輸 |

#### 傳輸狀態

| 狀態 | 說明 |
|------|------|
| Pending | 排隊等待中 |
| InProgress | 傳輸中 |
| Completed | 已完成 |
| Failed | 失敗（顯示錯誤訊息）|
| Cancelled | 已取消 |

---

### 4.5 OpenClaw 遠端管理

FlyMode 深度整合 [OpenClaw](https://github.com/openclaw)，提供從偵測、連線到操作的一站式遠端管理體驗。你可以在 FlyMode 的裝置頁面直接管理多台安裝 OpenClaw 的遠端機器，無需手動 SSH 登入。

#### 自動偵測 OpenClaw Gateway

FlyMode 會自動偵測遠端裝置上的 OpenClaw：

| 項目 | 說明 |
|------|------|
| 偵測方式 | 透過 SSH 執行 `pgrep -f openclaw-gateway` |
| 偵測間隔 | 每 120 秒自動掃描一次 |
| 偵測對象 | 所有**已信任**且**在線**的裝置 |
| UI 指示 | 偵測到時，裝置卡片上顯示「>_」終端機按鈕 |

不需要任何額外設定 — 只要遠端裝置的 OpenClaw Gateway 正在執行，FlyMode 就會自動發現。

#### 一鍵開啟 OpenClaw TUI

點擊裝置卡片上的「>_」按鈕，FlyMode 會自動完成以下步驟：

1. **建立 SSH PTY 連線**到遠端裝置
2. **自動定位 `openclaw` 路徑** — 先透過 `which openclaw` 查找，若找不到則搜尋 `/home`、`/usr/local/bin`、`/usr/bin`、`/opt` 等目錄（包含 symlink）
3. **啟動 OpenClaw TUI** — 執行 `openclaw tui`，自動設定 UTF-8 環境
4. **開啟內嵌式終端機** — 在 FlyMode 視窗內顯示完整的 TUI 介面

整個過程只需一次點擊，無需記住遠端主機名、IP、路徑或任何命令。

#### 本機 OpenClaw

FlyMode 不只偵測遠端裝置，也會偵測**本機**的 OpenClaw。若本機正在執行 OpenClaw，「This Device」卡片右上角會出現「>_」按鈕，點擊即可透過 SSH localhost 開啟本機的 OpenClaw TUI。

- 本機偵測使用 `pgrep`（不需 SSH），速度更快
- 若本機沒有 SSH 金鑰，點擊時會彈出密碼輸入框
- 密碼在同一次 FlyMode 執行期間會被記住，不需重複輸入

#### 多裝置分頁切換

如果你有多台機器都安裝了 OpenClaw（包含本機），FlyMode 提供**瀏覽器分頁風格**的多 session 終端機：

```
┌──────────────────────────────────────────────────────────┐
│ [● 我的電腦 (localhost)] [○ 辦公室伺服器] [○ 家裡NAS] [x] │
├──────────────────────────────────────────────────────────┤
│                                                            │
│              active device 的 OpenClaw TUI                 │
│              (xterm.js)                                    │
│                                                            │
└──────────────────────────────────────────────────────────┘
```

- **Device Navbar**：列出所有偵測到 OpenClaw 的裝置（本機排在最前面）
- **狀態圓點**：🟢 已連線、🔵 連線中（脈動動畫）、🔴 連線失敗、⚪ 尚未連線
- **首次點擊**某裝置 → 建立 SSH 連線 + xterm 實例
- **再次點擊**已連線裝置 → 只切換顯示，session 保持運作
- 每個 xterm 實例獨立運行（show/hide 切換，不銷毀/重建）
- 關閉 `[x]` → 關閉**所有** SSH session，modal 消失
- 若只有一台裝置有 OpenClaw → navbar 不顯示，行為和單一 session 相同

你可以搭配 Tailscale 跨網路管理（如本機、家裡的 NAS、辦公室的伺服器、雲端 VPS），在同一個視窗中快速切換。

#### 終端機功能

| 功能 | 說明 |
|------|------|
| 中文/日文輸入 | 完整支援 fcitx5、iBus 等 CJK 輸入法，無重複字元問題 |
| 剪貼簿複製 | 選取文字即自動複製到系統剪貼簿 |
| 剪貼簿貼上 | `Ctrl+Shift+V` 貼上 |
| 動態縮放 | 視窗大小改變時自動調整終端機列數和行數 |
| 游標 | 閃爍方塊游標，在任何背景下都清晰可見 |
| 256 色 | 支援 xterm-256color 完整色彩 |
| WebGL 渲染 | 使用 GPU 加速渲染，流暢的 TUI 操作體驗 |

#### 使用情境

| 情境 | 說明 |
|------|------|
| 遠端伺服器管理 | 透過 Tailscale 連線到辦公室或雲端的 OpenClaw 伺服器 |
| 多節點監控 | 在一台電腦上依序檢查多台 OpenClaw 節點的狀態 |
| 行動辦公 | 在筆電上隨時連回家中或辦公室的 OpenClaw |
| 跨平台操作 | Linux / macOS / Windows（計畫中）都能連線到任意平台的 OpenClaw |

#### 技術細節

| 項目 | 說明 |
|------|------|
| 終端機引擎 | xterm.js v6.1 (beta) + WebGL 渲染器 |
| 連線協議 | SSH PTY（xterm-256color, 動態 cols/rows）|
| 路徑搜尋 | `bash -lc 'which openclaw'` → 多目錄 `find` 含 symlink |
| 編碼 | UTF-8（自動設定 `LANG=en_US.UTF-8 LC_ALL=en_US.UTF-8`）|
| IME 處理 | 50ms 去重機制 + compositionend 清除，防止重複字元和文字累積 |
| 工作階段管理 | 多 session 分頁切換，每個連線獨立 Session ID，show/hide 不銷毀 |
| 本機偵測 | `pgrep`（不需 SSH），自動取得 username 和 SSH key path |

---

### 4.6 無線排程 ⏰

#### 建立規則

| 欄位 | 說明 |
|------|------|
| Name | 規則名稱 |
| Target | WiFi / Bluetooth / Airplane Mode / Custom Command |
| Action | Enable / Disable / Toggle / Run Command |
| Start Time | 開始時間（HH:MM 格式）|
| End Time | 結束時間（選填，HH:MM 格式）|
| Days | 勾選要執行的星期（一到日）|
| Command | 自定義命令（僅 Custom Command 目標時填寫）|

#### 規則行為

- **單一時間點**：只設 Start Time → 在該時間執行一次
- **時間範圍**：設 Start + End Time → 開始時執行 Action，結束時執行反向動作
- **跨日範圍**：如 22:00 - 06:00，會在晚上 10 點到隔天早上 6 點期間生效
- **執行間隔**：預設每 60 秒檢查一次（可在設定頁面調整）

#### 管理

| 操作 | 說明 |
|------|------|
| Toggle | 啟用/停用規則（不需刪除）|
| Execute Now | 立即手動執行一次 |
| Edit | 修改規則設定 |
| Delete | 刪除規則 |

---

### 4.7 快速操作 ⚡

#### 無線控制

三個即時切換按鈕：

| 按鈕 | 功能 |
|------|------|
| WiFi | 開啟/關閉 WiFi |
| Bluetooth | 開啟/關閉藍牙 |
| Airplane Mode | 開啟/關閉飛航模式 |

#### 自定義命令

在文字欄位輸入任意 shell 命令，點「Run」執行。執行結果會顯示在下方。

> **注意：** 命令以當前使用者權限執行。需要 root 權限的命令需加 `sudo`（需免密碼 sudo 設定）。

---

### 4.8 設定 ⚙️

| 設定項目 | 說明 | 預設值 |
|----------|------|--------|
| Show Notifications | 顯示系統通知 | 開啟 |
| Minimize to Tray | 關閉視窗時最小化到系統匣 | 關閉 |
| Launch at Startup | 開機自動啟動 | 關閉 |
| Require Password | 開啟時需輸入系統密碼 | 關閉 |
| Check Interval | 排程規則檢查間隔（秒）| 60 |

#### 系統密碼鎖定

啟用「Require Password」後：
- 每次開啟 FlyMode 需輸入**系統登入密碼**（非額外設定的密碼）
- 從系統匣還原視窗時，若隱藏超過 1 秒，也需要重新輸入密碼
- 使用 Linux PAM 機制驗證（`unix_chkpwd`）

#### 系統匣

啟用「Minimize to Tray」後：
- 關閉視窗不會退出程式，而是最小化到系統匣
- 點系統匣圖示可還原視窗
- 右鍵系統匣圖示 → 可選「Show FlyMode」或「Quit」

#### 版本資訊

設定頁面底部顯示：
- 應用版本號（如 v0.3.0）
- Git commit hash

---

## 5. 系統架構

```
┌─────────────────────────────────────────────────────────────┐
│                       FlyMode v0.3.0                         │
├─────────────────────────────────────────────────────────────┤
│                                                               │
│  ┌─────────────────── 前端 (Preact + TypeScript) ──────────┐ │
│  │                                                           │ │
│  │  📝 Notes  🔗 Devices  🔄 Sync  📤 Transfer             │ │
│  │  ⏰ Schedule  ⚡ Quick  ⚙️ Settings  🔒 Lock  >_ Terminal │ │
│  │                                                           │ │
│  └────────────────────────┬──────────────────────────────────┘ │
│                           │ Tauri IPC                          │
│  ┌────────────────────────▼──────────────────────────────────┐ │
│  │                後端 (Rust + Tauri 2 + Tokio)               │ │
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
│  │          系統匣 + 開機啟動 + TCP 配對服務 (19131)          │ │
│  └──────────────────────────────────────────────────────────┘ │
│                                                               │
└─────────────────────────────────────────────────────────────┘
```

### 模組說明

| 模組 | 路徑 | 說明 |
|------|------|------|
| `main.rs` | `src-tauri/src/` | 應用入口，初始化所有狀態，註冊 IPC 命令，啟動排程和自動同步 |
| `commands/` | `src-tauri/src/` | 所有 Tauri IPC 命令處理器（30+ 命令）|
| `notes/` | `src-tauri/src/` | 便利貼 CRUD、SQLite 操作、sync_hash 計算、全文搜尋 |
| `p2p/` | `src-tauri/src/` | 裝置管理、SSH 客戶端、Tailscale 發現、TCP 配對服務 |
| `sync/` | `src-tauri/src/` | 同步引擎、LWW 衝突解決、筆記合併 |
| `transfer/` | `src-tauri/src/` | SFTP 檔案傳輸、佇列管理、進度追蹤、並行控制 |
| `terminal/` | `src-tauri/src/` | SSH PTY 管理、OpenClaw 偵測、xterm.js 後端 |
| `scheduler/` | `src-tauri/src/` | 排程規則評估、時間範圍計算、動作執行 |
| `wireless/` | `src-tauri/src/` | 平台特定的 WiFi/藍牙/飛航模式控制 |
| `config/` | `src-tauri/src/` | AppConfig 和 P2PConfig 的序列化（JSON）|
| `crypto/` | `src-tauri/src/` | SSH 密碼加密/解密 |

---

## 6. 資料存儲

### 檔案位置

| 資料 | 路徑 | 格式 |
|------|------|------|
| 應用設定 | `~/.config/flymode/config.json` | JSON |
| P2P 設定 | `~/.config/flymode/p2p.json` | JSON |
| 便利貼資料庫 | `~/.local/share/flymode/notes.db` | SQLite |
| 同步資料夾 | `~/.local/share/flymode/sync/` | 檔案 |

### config.json 範例

```json
{
  "rules": [
    {
      "id": "550e8400-e29b-41d4-a716-446655440000",
      "name": "睡前關 WiFi",
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

### p2p.json 範例

```json
{
  "device_id": "a1b2c3d4-e5f6-7890-abcd-ef1234567890",
  "device_name": "my-laptop",
  "listen_port": 19131,
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
      "flymode_version": "0.3.0"
    }
  ],
  "auto_discover_tailscale": true,
  "sync_enabled": true,
  "sync_interval_seconds": 300
}
```

### SQLite 資料庫 Schema

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
    deleted INTEGER NOT NULL DEFAULT 0   -- 軟刪除
);

-- 索引
CREATE INDEX idx_notes_updated ON notes(updated_at);
CREATE INDEX idx_notes_category ON notes(category);
CREATE INDEX idx_notes_deleted ON notes(deleted);
```

---

## 7. 安全性

### SSH 通訊

所有裝置間的通訊（同步、檔案傳輸、終端機）都透過 SSH 加密通道。

| 認證方式 | 優先順序 | 說明 |
|----------|---------|------|
| SSH 金鑰 | 1（建議）| 最安全，免密碼 |
| SSH 密碼 | 2 | 密碼存在本地設定檔中 |

### 密碼存儲

- SSH 密碼使用 AES 加密存儲在 `p2p.json` 中，以 device_id 作為加密金鑰
- 系統密碼鎖不存儲密碼，每次驗證都透過系統 PAM 機制

### 網路安全

- FlyMode 不開放任何對外服務端口（除了 TCP 19131 配對服務，可在 `p2p.json` 自訂 port）
- SSH 連線都是主動向外連線（作為客戶端）
- 支援 Tailscale 私有網路（WireGuard 加密）

### 建議

1. **使用 SSH 金鑰認證**（而非密碼），更安全且方便
2. **啟用 Tailscale**，避免在公開網路暴露 SSH port
3. **啟用密碼鎖**（Settings → Require Password），防止他人使用
4. 定期更新 FlyMode 和系統套件

---

## 8. 疑難排解

### 安裝問題

| 問題 | 解決方式 |
|------|---------|
| `gh auth login` 失敗 | 確認有 GitHub 帳號且有 repo 存取權限 |
| `cargo tauri build` 失敗 | 確認已安裝所有系統依賴（libgtk, libwebkit 等）|
| `npm install` 失敗 | 確認 Node.js >= 18（`node -v`）|
| 找不到 `flymode` 命令 | 確認 `~/.local/bin` 在 PATH 中 |

### P2P 連線問題

| 問題 | 解決方式 |
|------|---------|
| 裝置顯示 Offline | 1. 確認對方 SSH Server 正在執行<br>2. 確認 IP 正確<br>3. 確認防火牆允許 port 22<br>4. `ssh user@ip` 手動測試 |
| 配對請求收不到 | 1. 確認防火牆允許配對 port（預設 19131，見 `p2p.json`）<br>2. 確認對方 FlyMode 正在執行<br>3. 兩台在同一網路（或都在 Tailscale）|
| Tailscale 發現不到裝置 | 1. `tailscale status` 確認兩台都在線<br>2. 確認登入同一個 Tailscale 帳號 |
| SSH 連線失敗 | 1. 確認使用者名稱正確<br>2. 確認金鑰路徑正確或密碼正確<br>3. `ssh -v user@ip` 看詳細錯誤 |

### 同步問題

| 問題 | 解決方式 |
|------|---------|
| 同步後筆記沒更新 | 確認兩端都已 Trust 對方 |
| 同步失敗 | 查看同步紀錄的錯誤訊息，通常是 SSH 連線問題 |
| 衝突覆蓋了我的修改 | LWW 策略以時間戳為準。避免同時在兩台修改同一筆記 |

### OpenClaw / 終端機問題

| 問題 | 解決方式 |
|------|---------|
| 看不到「>_」按鈕 | 1. 確認裝置正在執行 OpenClaw（本機：`pgrep -f openclaw`；遠端：需已 Trust 且 Online）<br>2. 等待 120 秒讓偵測掃描完成 |
| 本機連線失敗「No SSH key or password」| 本機沒有 SSH 金鑰 → 點「>_」時會彈出密碼輸入框，輸入系統密碼即可 |
| 終端機連線失敗 | 1. 確認 SSH 連線正常（先測試同步是否正常）<br>2. 確認遠端有安裝 `openclaw` 且路徑可被找到 |
| 顯示「openclaw not found」| 確認遠端 `openclaw` binary 在 PATH 中，或位於 `/usr/local/bin`、`/usr/bin`、`/opt` 等目錄 |
| 中文輸入重複 | 升級到最新版 FlyMode（已修復）|
| 游標看不到 | 升級到最新版 FlyMode（已修復，使用 WebGL 渲染器）|

### 無線控制問題

| 問題 | 解決方式 |
|------|---------|
| WiFi 切換無效 | Linux: 確認 `nmcli` 可用（`which nmcli`）|
| 藍牙切換無效 | Linux: 確認 `rfkill` 可用（`which rfkill`）|
| 飛航模式無效 | Linux: 確認 `rfkill` 可用 |

---

## 9. 技術參考

### 後端依賴

| 套件 | 版本 | 用途 |
|------|------|------|
| `tauri` | 2.x | 桌面應用框架 |
| `tokio` | 1.x | 非同步運行時 |
| `rusqlite` | 0.31 | SQLite 資料庫（bundled）|
| `ssh2` | 0.9 | SSH / SFTP 通訊 |
| `serde` / `serde_json` | 1.x | JSON 序列化 |
| `chrono` | 0.4 | 時間處理 |
| `sha2` | 0.10 | sync_hash 計算 |
| `crossbeam-channel` | 0.5 | 終端機 I/O 通道 |
| `thiserror` | 1.x | 錯誤型別定義 |
| `tracing` | 0.1 | 日誌輸出 |
| `uuid` | 1.x | UUID 產生 |
| `tauri-plugin-autostart` | 2.x | 開機自動啟動 |
| `tauri-plugin-dialog` | 2.x | 檔案對話框 |
| `tauri-plugin-notification` | 2.x | 系統通知 |

### 前端依賴

| 套件 | 版本 | 用途 |
|------|------|------|
| `preact` | 10.x | UI 框架 |
| `@tauri-apps/api` | 2.x | Tauri IPC |
| `@xterm/xterm` | 6.1.0-beta | 終端機模擬器 |
| `@xterm/addon-fit` | 0.11 | 終端機自動縮放 |
| `@xterm/addon-webgl` | 0.19 | WebGL 渲染器 |
| `vite` | 5.x | 建置工具 |
| `typescript` | 5.x | 型別檢查 |

### IPC 命令一覽

#### 設定

| 命令 | 功能 |
|------|------|
| `get_config` | 載入應用設定 |
| `save_config` | 儲存應用設定 |
| `get_build_info` | 取得版本號和 git hash |

#### 無線控制

| 命令 | 功能 |
|------|------|
| `get_status` | 取得 WiFi/藍牙/飛航模式狀態 |
| `toggle_wifi` | 開關 WiFi |
| `toggle_bluetooth` | 開關藍牙 |
| `toggle_airplane_mode` | 開關飛航模式 |
| `run_custom_command` | 執行自定義命令 |

#### 排程

| 命令 | 功能 |
|------|------|
| `add_rule` | 新增排程規則 |
| `update_rule` | 修改規則 |
| `delete_rule` | 刪除規則 |
| `toggle_rule` | 啟用/停用規則 |
| `execute_rule_now` | 立即執行規則 |

#### 便利貼

| 命令 | 功能 |
|------|------|
| `create_note` | 建立筆記 |
| `update_note` | 修改筆記 |
| `delete_note` | 刪除筆記 |
| `get_note` | 取得單一筆記 |
| `list_notes` | 列出所有筆記 |
| `search_notes` | 搜尋筆記 |
| `get_note_colors` | 取得可用顏色 |
| `get_note_categories` | 取得可用類別 |

#### 裝置管理

| 命令 | 功能 |
|------|------|
| `get_p2p_config` | 載入 P2P 設定 |
| `save_p2p_config` | 儲存 P2P 設定 |
| `add_peer` | 新增裝置 |
| `remove_peer` | 移除裝置 |
| `update_peer` | 修改裝置 |
| `check_peer_status` | 檢查單一裝置狀態 |
| `check_all_peers` | 檢查所有裝置狀態 |
| `discover_tailscale` | Tailscale 自動發現 |
| `get_device_id` | 取得本機 ID |
| `get_device_name` | 取得本機名稱 |

#### 配對

| 命令 | 功能 |
|------|------|
| `pair_with_peer` | 發送配對請求 |
| `get_pending_pair_requests` | 取得待處理的配對請求 |
| `accept_pair_request` | 接受配對 |
| `reject_pair_request` | 拒絕配對 |

#### 同步

| 命令 | 功能 |
|------|------|
| `get_sync_state` | 取得同步狀態 |
| `sync_with_peer` | 與指定裝置同步 |
| `sync_all_peers` | 與所有信任裝置同步 |
| `export_notes` | 匯出筆記為 JSON |
| `import_notes` | 從 JSON 匯入筆記 |
| `get_sync_folder` | 取得同步資料夾路徑 |

#### 檔案傳輸

| 命令 | 功能 |
|------|------|
| `get_transfer_queue` | 取得傳輸佇列 |
| `upload_file` | 上傳檔案 |
| `download_file` | 下載檔案 |
| `cancel_transfer` | 取消傳輸 |
| `clear_completed_transfers` | 清除已完成傳輸 |
| `get_transfer_progress` | 取得單一傳輸進度 |
| `browse_remote_files` | 瀏覽遠端檔案 |

#### 終端機

| 命令 | 功能 |
|------|------|
| `check_local_openclaw` | 偵測本機 OpenClaw 狀態（pgrep，不需 SSH）|
| `get_local_ssh_info` | 取得本機 SSH 使用者名稱和金鑰路徑 |
| `check_openclaw_status` | 偵測遠端 OpenClaw 狀態 |
| `open_terminal` | 開啟 SSH PTY 連線 |
| `send_terminal_input` | 傳送按鍵到終端機 |
| `resize_terminal` | 調整終端機大小 |
| `close_terminal` | 關閉終端機 |

#### 認證

| 命令 | 功能 |
|------|------|
| `verify_system_password` | 驗證系統密碼 |

### 平台特定實作

#### 無線控制

| 平台 | WiFi | 藍牙 | 飛航模式 |
|------|------|------|----------|
| Linux | `nmcli radio wifi` | `rfkill block/unblock bluetooth` | `rfkill block/unblock all` |
| Windows | PowerShell 網路介面卡 | `Get-Service bthserv` | 尚未實作 |
| macOS | `networksetup -getairportpower` | `blueutil --power` | 尚未實作 |

#### 系統密碼驗證

| 平台 | 方式 |
|------|------|
| Linux | `/usr/sbin/unix_chkpwd`（PAM helper, setuid root）|
| Windows | 尚未實作（計畫使用 WinAPI `LogonUserA`）|
| macOS | 尚未實作 |

---

*文件最後更新：2026-03-01*
*版本：v0.3.0*
