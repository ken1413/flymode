<p align="center">
  <img src="src-tauri/icons/128x128.png" alt="FlyMode" width="120" />
</p>

<h1 align="center">FlyMode</h1>

<p align="center">
  <strong>你的裝置，你的資料，不需要雲端。</strong>
</p>

<p align="center">
  <a href="https://github.com/ken1413/flymode/releases/latest"><img src="https://img.shields.io/github/v/release/ken1413/flymode?style=flat-square&color=blue&v=2" alt="Release" /></a>
  <a href="https://github.com/ken1413/flymode/blob/main/LICENSE"><img src="https://img.shields.io/github/license/ken1413/flymode?style=flat-square&v=2" alt="License" /></a>
  <a href="https://github.com/ken1413/flymode/releases"><img src="https://img.shields.io/github/downloads/ken1413/flymode/total?style=flat-square&color=green&v=2" alt="Downloads" /></a>
</p>

<p align="center">
  <strong>繁體中文</strong> | <a href="./README.md">English</a>
</p>

---

FlyMode 是一套**完全去中心化**的桌面應用程式，讓你的裝置直接互連 — 不需要雲端、不需要中央伺服器、不需要訂閱費用。同步筆記、傳輸檔案、遠端管理 [OpenClaw](https://github.com/nicholasgasior/openclaw) 節點、自動化無線控制，全部透過你自己機器之間的 SSH 加密通道完成。

以 **Rust + Tauri 2** 打造，擁有原生效能；前端採用 **Preact**，輕量且流暢。

---

## 為什麼選擇 FlyMode？

| | 傳統雲端應用 | FlyMode |
|---|---|---|
| **資料所有權** | 存在別人的伺服器上 | 留在你自己的裝置上 |
| **隱私保護** | 服務商可以存取你的資料 | 端對端 SSH 加密，零第三方存取 |
| **費用** | 月費/年費訂閱 | 免費開源，永遠免費 |
| **網路需求** | 必須有網際網路連線 | 區域網路、Tailscale、任何網路都行 |
| **掌控權** | 受制於供應商，條款隨時變更 | 程式碼、資料，全部由你掌控 |
| **可用性** | 服務中斷、關閉 | 只要你的機器開著就隨時可用 |

---

## 支援平台

| 平台 | 狀態 | 套件格式 |
|------|------|----------|
| **Linux** (Ubuntu 20.04+, Debian 11+) | 完整支援 | `.deb`、`.AppImage` |
| **Linux** (Fedora 36+) | 完整支援 | `.rpm`、`.AppImage` |
| **Linux** (Arch, Manjaro) | 完整支援 | `.AppImage`、從原始碼建置 |
| **macOS** (12 Monterey+) | 可從原始碼建置 | — |
| **Windows** | 規劃中 | — |

> FlyMode 目前以 Linux 為主。macOS 可從原始碼建置。Windows 支援已列入開發計畫。

### 系統需求

| 需求 | 最低規格 |
|------|---------|
| 記憶體 | 256 MB |
| 磁碟空間 | 100 MB（應用程式本身） |
| 螢幕解析度 | 1024 x 768 |
| 網路 | 區域網路、Tailscale VPN、或任何 IP 可達的網路 |

---

## 功能總覽

### 1. OpenClaw 遠端管理

在同一個視窗中管理你所有的 [OpenClaw](https://github.com/nicholasgasior/openclaw) 節點。FlyMode 自動發現你裝置上執行中的 OpenClaw Gateway，一鍵就能開啟 OpenClaw TUI — 不需要手動 SSH 登入。

```
┌─────────────────────────────────────────────────────────────────┐
│  [● 家裡伺服器]  [● 辦公室 VPS]  [○ 雲端節點]              [x] │
├─────────────────────────────────────────────────────────────────┤
│                                                                 │
│   ╔═══════════════════════════════════════════════════════════╗  │
│   ║                   OpenClaw TUI v1.x                      ║  │
│   ║                                                          ║  │
│   ║   節點狀態: Active                                       ║  │
│   ║   Peers: 12    頻寬: 1.2 GB/s                            ║  │
│   ║   運行時間: 14d 3h 22m                                   ║  │
│   ║                                                          ║  │
│   ╚═══════════════════════════════════════════════════════════╝  │
│                                                                 │
└─────────────────────────────────────────────────────────────────┘
```

#### 運作方式

| 步驟 | 做了什麼 | 詳細說明 |
|------|---------|---------|
| **自動偵測** | FlyMode 每 120 秒掃描所有信任裝置 | 透過 SSH 偵測執行中的 `openclaw-gateway` 行程。偵測到時，裝置卡片上出現 `>_` 按鈕。 |
| **一鍵連線** | 點擊 `>_` 按鈕 | FlyMode 自動建立 SSH PTY 連線、自動定位 `openclaw` 執行檔（透過 `which`、PATH 搜尋和多目錄 `find`，含 symlink），然後啟動 TUI 並設定好 UTF-8 編碼。 |
| **多節點分頁** | 瀏覽器風格即時切換 | 分頁列顯示所有有 OpenClaw 的裝置。每個 session 獨立運作 — 切換分頁不會中斷或重啟任何連線。 |
| **完整終端機** | 生產級終端體驗 | xterm-256color 搭配 WebGL GPU 渲染、動態視窗縮放、完整中文/日文輸入法支援（fcitx5/iBus）、剪貼簿整合（選取即複製、Ctrl+Shift+V 貼上）。 |

#### 本機支援

FlyMode 也會偵測**本機**上的 OpenClaw（透過 `pgrep`，不需要 SSH）。如果偵測到，「This Device」卡片上會出現 `>_` 按鈕。點擊後透過 SSH localhost 連線 — 如果沒有 SSH 金鑰，會自動彈出密碼輸入框。密碼在目前的 FlyMode 執行期間會被記住。

#### 終端機功能

| 功能 | 說明 |
|------|------|
| 色彩支援 | 完整 xterm-256color（1670 萬色） |
| 渲染引擎 | WebGL GPU 加速（自動回退到 canvas） |
| 視窗縮放 | 自動調整 — 終端機行列數即時跟隨視窗大小 |
| 中文/日文輸入 | 完整支援 fcitx5、iBus 等輸入法，零重複字元 |
| 剪貼簿 | 選取文字即自動複製；Ctrl+Shift+V 貼上 |
| 游標 | 閃爍方塊游標，在任何背景下都清晰可見 |
| 編碼 | UTF-8（自動設定 `LANG` 和 `LC_ALL` 環境變數） |
| Session 持續 | Show/hide 切換分頁不會中斷連線 — 每個 session 保持存活 |

#### 使用情境

- **多節點監控** — 5 台機器都跑 OpenClaw？打開 FlyMode，5 台全部以分頁出現。點擊連線，點擊切換。從一台筆電監控你整個 OpenClaw 網路。
- **遠端伺服器管理** — 透過 Tailscale 連線到辦公室伺服器或雲端 VPS。管理 OpenClaw 不需要記住主機名、IP 或路徑。
- **行動辦公** — 在咖啡廳用筆電？連回家裡的 NAS 或辦公室的伺服器，就像坐在它面前一樣管理 OpenClaw。

---

### 2. P2P 裝置同步與管理

不需要任何雲端服務，直接連結你的裝置。FlyMode 使用自訂 TCP 配對協議，並整合 Tailscale 實現自動裝置發現。

#### 裝置發現與配對

| 方式 | 說明 |
|------|------|
| **Tailscale 自動發現** | 點「Discover Tailscale Peers」— FlyMode 查詢 `tailscale status --json`，自動找到你 Tailscale 網路上的所有機器。不需要手動輸入 IP。 |
| **手動新增** | 輸入對方的 IP、SSH 使用者名稱和 port。適用於任何機器互相可達的網路。 |
| **TCP 配對** | 裝置 A 發送配對請求（TCP port 19131，可在 `p2p.json` 自訂）→ 裝置 B 接受 → 雙方互相加入裝置列表，並交換裝置資訊。 |

#### 信任機制

FlyMode 有兩級存取模型：

| 等級 | 可做的事 |
|------|---------|
| **已配對（未信任）** | 只能看到在線/離線狀態 |
| **已信任** | 完整權限：筆記同步、檔案傳輸、遠端終端機、OpenClaw 管理 |

雙方都必須互相信任，才能雙向同步。

#### 連線類型

| 圖示 | 類型 | 說明 |
|------|------|------|
| 🦎 | Tailscale | 透過 Tailscale VPN（WireGuard 加密） |
| 🏠 | LAN Direct | 同一個區域網路 |
| 🌐 | WAN Direct | 透過網際網路（需確保 SSH port 可存取） |

#### 即時狀態監控

每個裝置都有色彩指示燈，每 30 秒自動更新：

- 🟢 **Online** — SSH 可達，可以同步/傳輸
- 🔴 **Offline** — 無法連線
- ⚪ **Unknown** — 尚未檢查

---

### 3. 便利貼筆記 — 跨裝置同步

功能完整的筆記系統，自動同步到你所有信任的裝置。

#### 筆記功能

| 功能 | 說明 |
|------|------|
| **顏色** | 8 種選擇：黃、粉、藍、綠、紫、橙、白、灰 |
| **類別** | 7 種內建：一般、工作、個人、點子、任務、重要、封存 |
| **標籤** | 自訂標籤（如 `#專案A`、`#緊急`），顯示在筆記卡片上 |
| **釘選** | 將重要筆記釘選到列表頂部 |
| **搜尋** | 全文搜尋，涵蓋標題和內容 |
| **顯示模式** | 格狀（卡片排列）或列表（精簡） — 一鍵切換 |
| **軟刪除** | 刪除的筆記不會永久移除。刪除狀態透過同步傳播，筆記可以恢復。 |

#### 同步策略

FlyMode 使用 **Last-Write-Wins (LWW)** 衝突解決機制：

- 每筆筆記有 `updated_at` 時間戳和 `sync_hash`（SHA-256）
- 兩台裝置同時修改同一筆記時，以較新的時間戳為準
- `sync_hash` 偵測實際內容變更 — 相同的編輯不會觸發不必要的覆寫
- 只存在一端的筆記會自動同步到所有信任裝置
- 自動同步間隔可設定：1 分鐘、5 分鐘、15 分鐘、30 分鐘、或 1 小時

#### 匯出與匯入

匯出所有筆記為 JSON 檔案，方便備份。從 JSON 匯入可還原筆記，或在沒有 SSH 連線時手動交換。

---

### 4. 安全檔案傳輸（SFTP）

使用 SFTP 在裝置間直接傳輸檔案 — 加密、點對點、沒有中間人、無檔案大小限制。

| 功能 | 說明 |
|------|------|
| **上傳** | 透過系統檔案選擇器選擇本機檔案 → 指定遠端目標路徑 → 開始傳輸 |
| **下載** | 視覺化瀏覽遠端檔案系統 → 點擊檔案下載 → 選擇本機存放路徑 |
| **遠端檔案瀏覽器** | 瀏覽目錄、查看檔名、大小、修改時間。點 `..` 回到上層。 |
| **進度追蹤** | 每個檔案即時顯示百分比和傳輸速度 |
| **佇列管理** | 最多 3 筆並行傳輸。額外檔案排入佇列，有空位時自動開始。 |
| **取消** | 隨時可取消任何進行中或排隊中的傳輸 |

傳輸狀態：`Pending` → `InProgress` → `Completed`（或 `Failed` / `Cancelled`）

---

### 5. 無線排程與快速操作

按排程自動控制你的無線硬體，或即時切換。

#### 排程規則

建立每日或每週自動執行的規則：

| 欄位 | 選項 |
|------|------|
| **目標** | WiFi、藍牙、飛航模式、或自訂命令 |
| **動作** | 開啟、關閉、切換、或執行命令 |
| **時間** | 開始時間（HH:MM）。可選設定結束時間。 |
| **星期** | 勾選任意星期一到星期日的組合 |

**時間範圍範例：** 設定 WiFi 在 23:00 關閉、07:00 重新開啟 — FlyMode 自動處理跨日時間範圍。

**自訂命令範例：** 每天凌晨 03:00 執行 `sudo systemctl restart nginx`。

規則可以啟用/停用（無需刪除），也可以手動觸發「立即執行」。

#### 快速操作

三個即時切換按鈕：WiFi、藍牙、飛航模式 — 不需排程，點擊即生效。

另有命令執行器：輸入任意 shell 命令，立即執行並顯示輸出。

| 平台 | WiFi | 藍牙 | 飛航模式 |
|------|------|------|----------|
| Linux | `nmcli` | `rfkill` | `rfkill` |
| macOS | `networksetup` | `blueutil` | 規劃中 |
| Windows | 規劃中 | 規劃中 | 規劃中 |

---

### 6. 安全性與隱私

| 功能 | 說明 |
|------|------|
| **全部通訊加密** | 裝置間的每一個操作（同步、檔案傳輸、終端機）都經過 SSH 加密通道 |
| **SSH 金鑰認證** | 支援 ed25519 和 RSA 金鑰。建議優先於密碼認證。 |
| **密碼認證** | SSH 密碼在存入本機設定檔前會經過 AES 加密 |
| **系統密碼鎖** | 需要你的作業系統登入密碼才能開啟 FlyMode（Linux PAM 透過 `unix_chkpwd`） |
| **系統匣重新驗證** | 從系統匣還原視窗超過 1 秒後，需要重新輸入密碼 |
| **不對外連線** | FlyMode 不會回傳任何資料。唯一的監聽 port 是 TCP 19131（本機配對用，可在 `~/.config/flymode/p2p.json` 自訂）。 |
| **Tailscale 相容** | 搭配 Tailscale 使用 WireGuard 加密的私有網路 |
| **開機自動啟動** | 可選擇開機自動啟動，搭配系統匣背景執行 |

---

## 快速安裝

### 預編譯套件（推薦）

一行指令下載安裝 — **不需要 Rust 或 Node.js 環境**：

```bash
curl -fsSL https://raw.githubusercontent.com/ken1413/flymode/main/install.sh | bash
```

安裝腳本會自動偵測你的發行版，安裝對應格式（`.deb`、`.rpm`、或 `.AppImage`）。

使用 AppImage（免 sudo）：

```bash
curl -fsSL https://raw.githubusercontent.com/ken1413/flymode/main/install.sh | bash -s -- --appimage
```

安裝完成後啟動：

```bash
flymode
```

### 從原始碼建置

適合開發者或未支援的發行版。需要 **Rust 1.70+** 和 **Node.js 18+**：

```bash
curl -fsSL https://raw.githubusercontent.com/ken1413/flymode/main/setup.sh | bash
```

此腳本會自動安裝所有依賴（Rust、Node.js、系統函式庫）、clone 專案、並從原始碼建置。

或手動操作：

```bash
git clone https://github.com/ken1413/flymode.git
cd flymode
cd src-ui && npm install && cd ..
cargo tauri build
# 執行檔在：target/release/flymode
```

<details>
<summary><strong>手動建置所需的系統依賴</strong></summary>

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

## 必裝軟體與前置條件

FlyMode 本身是一個單一執行檔，但它的 P2P 功能依賴幾個系統服務。以下是你需要的東西：

### 必要（P2P 功能需要）

| 軟體 | 為什麼需要 | 如何安裝 |
|------|-----------|---------|
| **SSH Server** (openssh-server) | 裝置間所有通訊（同步、檔案傳輸、終端機）都使用 SSH。**兩台機器都必須安裝**。 | Ubuntu: `sudo apt install openssh-server && sudo systemctl enable --now ssh`<br>Fedora: `sudo dnf install openssh-server && sudo systemctl enable --now sshd`<br>Arch: `sudo pacman -S openssh && sudo systemctl enable --now sshd`<br>macOS: 系統設定 → 一般 → 共享 → 遠端登入 → 開啟 |
| **SSH 金鑰或密碼** | FlyMode 透過 SSH 認證來連線到對方裝置。你需要 SSH 金鑰對或對方使用者的密碼。 | 產生金鑰: `ssh-keygen -t ed25519`<br>複製到對方: `ssh-copy-id user@remote-ip` |

> **注意：** `install.sh` 安裝腳本會自動安裝並啟用 SSH server。如果你使用預編譯套件安裝，這步驟已自動處理。

### 建議（選用但非常實用）

| 軟體 | 為什麼推薦 | 如何安裝 |
|------|-----------|---------|
| **Tailscale** | 讓不同網路的裝置（家裡、辦公室、雲端）像在同一個區域網路一樣互連。零設定 VPN，WireGuard 加密。FlyMode 會自動發現 Tailscale 上的裝置。 | `curl -fsSL https://tailscale.com/install.sh \| sh && sudo tailscale up` |

### 防火牆 Port（如果有防火牆的話）

如果你的機器有啟用防火牆，以下 port 需要開放：

| Port | 協議 | 用途 |
|------|------|------|
| **22** | TCP | SSH — 所有 P2P 通訊（同步、傳輸、終端機） |
| **19131** | TCP | FlyMode 配對協議 — 裝置發現和配對請求 |

```bash
# Ubuntu (ufw)
sudo ufw allow 22/tcp && sudo ufw allow 19131/tcp

# Fedora (firewalld)
sudo firewall-cmd --permanent --add-service=ssh
sudo firewall-cmd --permanent --add-port=19131/tcp
sudo firewall-cmd --reload
```

> 如果兩台機器在同一個區域網路且沒有防火牆，或都使用 Tailscale，不需要額外設定 port。

### 不需要的東西

| 軟體 | 狀態 |
|------|------|
| 雲端帳號 | 不需要。FlyMode 完全去中心化。 |
| 資料庫伺服器 | 不需要。FlyMode 使用內嵌式 SQLite。 |
| Docker | 不需要。FlyMode 是單一原生執行檔。 |
| 網際網路連線 | 區域網路同步不需要。只有 Tailscale 或 WAN 連線才需要。 |

---

## 開始使用（一步一步來）

### 步驟 1：在兩台電腦上安裝 FlyMode

使用上方的[快速安裝](#快速安裝)指引。

### 步驟 2：連結裝置

**方式 A — Tailscale 自動發現（推薦，適合遠端/跨網路）：**

1. 在兩台電腦安裝 [Tailscale](https://tailscale.com)
2. 執行 `sudo tailscale up`，兩台都用同一個帳號登入
3. 在 FlyMode → 裝置頁面 → 點 **「Discover Tailscale Peers」**
4. 對方電腦自動出現在裝置列表中

**方式 B — 手動新增（適合區域網路或任何直接可達的網路）：**

1. 在對方電腦確認 IP：`hostname -I`（區域網路）或 `tailscale ip`（Tailscale）
2. 在 FlyMode → 裝置頁面 → 點 **「Add Peer」**
3. 填入：名稱、IP 位址、SSH Port（22）、SSH 使用者名稱

### 步驟 3：設定 SSH 認證

配對只交換裝置資訊（名稱、IP）。**你必須手動設定 SSH 認證：**

1. 點對方裝置卡片上的 **「Edit」** 按鈕
2. 填入 **SSH User**（對方電腦的登入使用者名稱）
3. 選擇認證方式：
   - **SSH Key Path**（建議）：如 `~/.ssh/id_ed25519` — 需要先做過 `ssh-copy-id`
   - **SSH Password**：對方使用者的登入密碼
4. 儲存

> **重要：** 兩台電腦都必須各自設定對方的 SSH 認證。同步、檔案傳輸、終端機功能都需要 SSH 連線才能運作。

**快速 SSH 金鑰設定（建議）：**

```bash
# 在電腦 A 上：
ssh-keygen -t ed25519                    # 產生金鑰（已有則跳過）
ssh-copy-id youruser@machine-b-ip       # 複製公鑰到電腦 B

# 在電腦 B 上：
ssh-keygen -t ed25519
ssh-copy-id youruser@machine-a-ip       # 複製公鑰到電腦 A
```

### 步驟 4：配對裝置

在裝置列表中，點對方裝置的 **「Pair」** 按鈕。對方的 FlyMode 會收到配對請求 — 點 **「Accept」** 接受。

### 步驟 5：建立信任

點對方裝置卡片上的 **「Trust」** 按鈕。信任之後：

- 筆記在背景自動同步
- 可以傳輸檔案
- 可以開啟遠端終端機（包括 OpenClaw TUI）

> 兩台電腦都必須互相信任，才能雙向同步。

### 步驟 6：驗證一切正常

- **同步頁面** → 點「Sync Now」→ 確認筆記出現在兩台電腦上
- **傳輸頁面** → 上傳一個小測試檔案 → 確認對方收到
- **裝置頁面** → 狀態應顯示 🟢 Online
- **OpenClaw** → 如果有在執行，`>_` 按鈕應在 120 秒內出現

---

## 系統架構

```
┌─────────────────────────────────────────────────────────────────┐
│                        FlyMode v0.3.1                           │
├─────────────────────────────────────────────────────────────────┤
│                                                                 │
│  ┌─────────────── 前端 (Preact + TypeScript) ─────────────────┐ │
│  │                                                             │ │
│  │  📝 筆記  🔗 裝置  🔄 同步  📤 傳輸                        │ │
│  │  ⏰ 排程  ⚡ 快速  ⚙️ 設定  🔒 鎖定  >_ 終端機              │ │
│  │                                                             │ │
│  └────────────────────────┬────────────────────────────────────┘ │
│                           │ Tauri IPC（30+ 命令）                │
│  ┌────────────────────────▼────────────────────────────────────┐ │
│  │              後端 (Rust + Tauri 2 + Tokio)                  │ │
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
│  │     系統匣 + 開機自動啟動 + TCP 配對服務 (port 19131)         │ │
│  └─────────────────────────────────────────────────────────────┘ │
│                                                                 │
└─────────────────────────────────────────────────────────────────┘
```

### 技術架構

| 層級 | 技術 | 用途 |
|------|------|------|
| **後端** | Rust, Tauri 2, Tokio | 原生效能、非同步 I/O、系統整合 |
| **資料庫** | SQLite（內嵌式、自帶） | 筆記存儲、全文搜尋、零設定 |
| **網路** | SSH2, SFTP | 加密裝置通訊、檔案傳輸 |
| **前端** | Preact, TypeScript, Vite | 輕量 UI（gzip 後約 50 KB） |
| **終端機** | xterm.js + WebGL | GPU 加速終端模擬器 |
| **VPN 整合** | Tailscale | 跨網路裝置發現 |

### 資料存儲

所有資料都留在你的本機上：

| 資料 | 位置 | 格式 |
|------|------|------|
| 應用設定 | `~/.config/flymode/config.json` | JSON |
| 裝置列表與 P2P 設定 | `~/.config/flymode/p2p.json` | JSON |
| 筆記資料庫 | `~/.local/share/flymode/notes.db` | SQLite |
| 同步工作目錄 | `~/.local/share/flymode/sync/` | 檔案 |

---

## 文件

更詳細的設定說明、功能教學和疑難排解：

- **[完整使用說明 (繁體中文)](./DOCUMENTATION.md)** — 安裝、設定、功能詳解、疑難排解、技術參考
- **[Full User Guide (English)](./DOCUMENTATION.en.md)** — installation, setup, all features, troubleshooting, technical reference

---

## 開發

```bash
# 安裝前端依賴（首次需要）
cd src-ui && npm install && cd ..

# 開發模式（前端 + 後端 hot reload）
cargo tauri dev

# 執行所有 Rust 測試（150+ 測試）
cd src-tauri && cargo test

# 執行前端測試
cd src-ui && npm test

# 生產建置
cargo tauri build

# 版本升級（同步更新 Cargo.toml, tauri.conf.json, package.json）
./bump-version.sh minor    # 或：patch, major, 0.4.0
```

詳細架構與開發流程請參閱 [CLAUDE.md](./CLAUDE.md)。

---

## 貢獻

歡迎貢獻！請先開 Issue 討論你想要做的更改。

## 授權

[MIT](./LICENSE)
