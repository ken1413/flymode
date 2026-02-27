# FlyMode v0.2.0 — 全面分析評估報告

**分析日期：** 2026-02-27
**分析範圍：** 全部 Rust 後端、Preact 前端、建置配置、技術文件

---

## 目錄

1. [總覽](#1-總覽)
2. [功能完成度對照表](#2-功能完成度對照表)
3. [嚴重問題 (CRITICAL)](#3-嚴重問題-critical)
4. [高優先問題 (HIGH)](#4-高優先問題-high)
5. [中優先問題 (MEDIUM)](#5-中優先問題-medium)
6. [低優先問題 (LOW)](#6-低優先問題-low)
7. [前後端 IPC 介面不一致](#7-前後端-ipc-介面不一致)
8. [測試覆蓋分析](#8-測試覆蓋分析)
9. [安全性評估](#9-安全性評估)
10. [效能觀察](#10-效能觀察)
11. [建議修復優先順序](#11-建議修復優先順序)

---

## 1. 總覽

| 指標 | 狀態 |
|------|------|
| 編譯 | ✅ 通過，0 errors，約 18 個 compiler warnings |
| 單元測試 | ✅ 68 個全部通過 |
| 整合測試 | ✅ 4 個全部通過 |
| Unsafe 程式碼 | ✅ 無 |
| 整體程式碼品質 | B+（架構清晰、模組分離良好，但多處功能為 placeholder） |
| **整體生產就緒度** | **約 55%** |

### 各模組完成度

| 模組 | 行數 | 完成度 | 測試數 | 品質 |
|------|------|--------|--------|------|
| notes | 963 | 90% | 25 | 優秀 |
| p2p | 840 | 80% | 25 | 良好（安全性待加強）|
| transfer | 594 | 55% | 16 | 基本（進度追蹤未實作）|
| sync | 561 | 30% | 11 | **不完整（核心同步邏輯為 placeholder）** |
| commands | 329 | 95% | — | 優秀 |
| scheduler | 217 | 75% | 0 | 可用（無測試）|
| wireless | 122 | 70% | 0 | 可用（平台相依）|
| config | 113 | 90% | 0 | 良好 |
| 前端整體 | ~2000 | 70% | 0 | 可用（缺錯誤處理與驗證）|

---

## 2. 功能完成度對照表

以 DOCUMENTATION.md 所列功能為基準，逐項比對實際程式碼：

### 2.1 無線排程 (Wireless Scheduler)

| 文件描述功能 | 實作狀態 | 問題 |
|-------------|---------|------|
| 定時開關 WiFi | ✅ 已實作 | Linux 依賴 nmcli，Windows 寫死 "Wi-Fi" adapter 名稱（語系問題）|
| 定時開關藍牙 | ✅ 已實作 | macOS 依賴 blueutil（非系統預裝）|
| 定時開關飛航模式 | ✅ 已實作 | — |
| 自定義 CLI 命令 | ✅ 已實作 | 無輸入驗證，可能被注入惡意命令 |
| 系統匣背景執行 | ⚠️ 停用中 | `main.rs:81` 明確註解 "Tray icon temporarily disabled" |
| Cron 排程引擎 | ✅ 已實作 | 無單元測試，時間邏輯未經驗證 |
| 規則啟用/停用 | ✅ 已實作 | — |
| 立即執行規則 | ✅ 已實作 | — |

### 2.2 P2P 同步 (Peer-to-Peer Sync)

| 文件描述功能 | 實作狀態 | 問題 |
|-------------|---------|------|
| 裝置管理 CRUD | ✅ 已實作 | 前端無表單驗證（可建立空白裝置）|
| Tailscale 自動發現 | ✅ 已實作 | 平台分支皆有實作 |
| SSH 金鑰認證 | ✅ 已實作 | 預設讀取 `~/.ssh/id_rsa` |
| SSH 密碼認證 | ✅ 已實作 | **密碼明文存於記憶體和 p2p.json** |
| 信任機制 | ✅ 已實作 | — |
| 裝置狀態檢查 | ✅ 已實作 | TCP 連線檢查有 fallback IP 問題 |
| **實際 P2P 資料同步** | ❌ **未完成** | `sync/mod.rs:191-209` 使用寫死的 shell script，**回傳假資料** |

### 2.3 便利貼筆記 (Sticky Notes)

| 文件描述功能 | 實作狀態 | 問題 |
|-------------|---------|------|
| 8 種顏色 | ✅ 已實作 | — |
| 7 種類別 | ✅ 已實作 | — |
| 標籤系統 | ✅ 已實作 | — |
| 釘選功能 | ✅ 已實作 | — |
| CRUD 操作 | ✅ 已實作 | **前端 create_note 漏傳 color/category/tags/pinned 參數** |
| 搜尋功能 | ⚠️ 部分實作 | 搜尋結果未套用類別篩選器 |
| 軟刪除 | ✅ 已實作 | — |
| Sync Hash 變更偵測 | ✅ 已實作 | — |
| 自動同步到信任裝置 | ❌ **未完成** | 同步引擎為 placeholder |

### 2.4 檔案傳輸 (File Transfer)

| 文件描述功能 | 實作狀態 | 問題 |
|-------------|---------|------|
| 上傳檔案 | ⚠️ 基本實作 | **無實際進度追蹤**，直接從 0 跳到 100% |
| 下載檔案 | ⚠️ 基本實作 | 同上 |
| 瀏覽遠端檔案 | ✅ 已實作 | 前端路徑寫死 `/home/${ssh_user}/`（假設 Linux）|
| 傳輸佇列管理 | ⚠️ 部分實作 | `max_concurrent: 3` **宣告了但從未執行限制** |
| 取消傳輸 | ✅ 已實作 | — |
| 傳輸速度顯示 | ❌ 未實作 | `speed_bps` 欄位存在但永遠為 None |

---

## 3. 嚴重問題 (CRITICAL)

### C1: 同步引擎為 Placeholder — 核心功能不可用

**位置：** `src-tauri/src/sync/mod.rs:191-209`

同步功能是本應用的核心賣點之一，但實際實作僅是一段寫死的 shell script，回傳假的同步回應。`parse_sync_response()` 依靠字串標記解析回應，檔案同步數寫死為 0。

```rust
// 目前的實作：寫死 shell script，不會真正同步遠端筆記
let sync_script = format!("echo 'FLYMODE_SYNC_RESPONSE_START'...");
```

**影響：** 同步、自動同步功能完全無法運作。文件所述的 P2P 同步架構（雙向 payload 交換、衝突解決）均未真正實現。

---

### C2: CSP 安全策略被停用

**位置：** `src-tauri/tauri.conf.json:29`

```json
"security": { "csp": null }
```

Content Security Policy 設為 null，等於完全關閉。結合 shell plugin 的 `"open": true`，攻擊者若能注入前端程式碼，可直接執行系統命令。

**修復：**
```json
"csp": "default-src 'self'; script-src 'self'; style-src 'self' 'unsafe-inline'; img-src 'self' data:; connect-src 'self'"
```

---

### C3: SSH 密碼明文儲存

**位置：** `src-tauri/src/p2p/mod.rs:55, 400-401` + `~/.config/flymode/p2p.json`

SSH 密碼以 `String` 型別存於記憶體（不會被自動清零），也以明文寫入 JSON 設定檔。任何有讀取權限的程式都能取得。

**修復：** 使用 `zeroize` crate 處理記憶體中的密碼，設定檔密碼應加密或使用系統 keyring。

---

### C4: Shell 命令無白名單限制

**位置：** `src-tauri/tauri.conf.json:44-46`

Shell plugin 僅設定 `"open": true`，沒有 `scope` 限制可執行的命令。`run_custom_command` 接受使用者任意輸入的命令字串，前端也沒有任何驗證。

**風險：** 命令注入（Command Injection）。

---

## 4. 高優先問題 (HIGH)

### H1: 前端 create_note 漏傳關鍵參數

**位置：** `src-ui/src/components/NotesTab.tsx:108`

```typescript
await invoke('create_note', { title: form.title, content: form.content });
// 缺少: color, category, tags, pinned
```

後端 `create_note` 可能預期接收完整的 Note 結構，但前端只傳了 title 和 content。新建的筆記可能全部使用預設值，使用者在表單中選擇的顏色和類別被忽略。

---

### H2: 傳輸進度追蹤未實作

**位置：** `src-tauri/src/transfer/mod.rs:157-187`

`transferred_bytes` 在傳輸完成後直接設為 `total_bytes`，沒有中間進度回報。前端的進度條會從 0% 直接跳到 100%，或者長時間停在 0% 後突然完成。

**修復：** 實作分塊讀取（chunked read），每個 chunk 更新 `transferred_bytes`。

---

### H3: 並行傳輸限制未執行

**位置：** `src-tauri/src/transfer/mod.rs`

`TransferQueue.max_concurrent` 設為 3，但程式碼中從未檢查此限制。每次 upload/download 都會直接 `tokio::spawn`，可能同時產生大量 SSH 連線導致資源耗盡。

---

### H4: 版本號不一致

| 檔案 | 版本 |
|------|------|
| `src-tauri/Cargo.toml` | `0.2.0` |
| `src-tauri/tauri.conf.json` | `0.1.0` |
| `src-ui/package.json` | `0.2.0` |

`tauri.conf.json` 的版本落後，會導致打包產出的版本資訊錯誤。

---

### H5: 前端全面缺乏使用者錯誤回饋

所有前端元件（7 個 Tab）的 catch 區塊僅使用 `console.error()` 或 `alert()`：
- 使用者不會看到操作失敗的訊息（console.error 只在開發者工具可見）
- `alert()` 會阻塞 UI 且樣式不一致
- 無 toast/notification 系統

---

### H6: 前端表單驗證全面缺失

| 元件 | 缺失的驗證 |
|------|-----------|
| NotesTab | 無（可建立空白筆記）|
| P2PTab | 可建立空白名稱/IP 的裝置 |
| RulesTab | 可取消所有星期天選項（無效規則）、不驗證 start < end time |
| TransferTab | 不驗證路徑是否為目錄 |
| QuickActionsTab | 自定義命令無任何驗證 |

---

### H7: Tauri 插件權限範圍（Capability Scope）未設定

`tauri.conf.json` 中 35+ 個 IPC 命令皆無 capability scope 定義。FS、Dialog、Notification、Autostart 插件都已註冊但無存取範圍限制。

---

## 5. 中優先問題 (MEDIUM)

### M1: Scheduler 模組無單元測試

`scheduler/mod.rs` 包含複雜的時間範圍判斷邏輯（跨午夜、星期過濾），但完全沒有測試。3 個方法（`stop()`, `is_running()`, `execute_now()`）宣告了但從未被呼叫（dead code）。

---

### M2: Clippy Warnings（約 13 個）

- Notes 模組：NoteColor/NoteCategory 手動實作 Default（可用 derive）
- P2P 模組：冗餘閉包 `.map_err(|e| P2PError::Ssh(e))` → `.map_err(P2PError::Ssh)`
- P2P 模組：`map().flatten()` → 應用 `and_then()`
- P2P 模組：未使用的 `tracing::info` import
- Transfer 模組：未使用的 `Transfer` enum variant、`format_size()`、`format_speed()`

---

### M3: 前端 CSS 硬編碼顏色

`style.css` 中定義了 CSS variables 進行主題管理，但 note card 的文字顏色直接寫死（`#1e293b`, `#334155`），不會隨主題變更。

---

### M4: 前端無響應式設計

無 media query、無行動裝置斷點。雖然是桌面應用（Tauri），但視窗最小寬度 600px 時 UI 可能排版異常。

---

### M5: 自動同步 10 秒硬編碼延遲

**位置：** `main.rs:76`

```rust
tokio::time::sleep(tokio::time::Duration::from_secs(10)).await;
```

Auto-sync 啟動前的延遲寫死為 10 秒，應改為可配置。

---

### M6: Sync Engine 在 auto_sync 迴圈中重複建立實例

**位置：** `sync/mod.rs:294-299`

應重用現有的 SyncEngine 實例而非每次迴圈重新建立，這違反了 Tauri State 單例管理的設計意圖。

---

### M7: 依賴版本過於寬鬆

Cargo.toml 中 Tauri 相關依賴使用 `version = "2"` 允許任何 2.x.x 版本，建議改為 `"2.10"` 等更精確的範圍。`tokio` 使用 `features = ["full"]`，應改為只啟用實際需要的 feature。

---

### M8: Windows 平台相容性問題

- `wireless/mod.rs:77-93`：WiFi adapter 名稱寫死為 "Wi-Fi"，在非英文語系的 Windows 系統會失敗
- `transfer` 前端路徑使用 `/` 分隔符（`src-ui/src/components/TransferTab.tsx:85`），Windows 上可能出錯

---

## 6. 低優先問題 (LOW)

### L1: Dead Code

| 位置 | 項目 |
|------|------|
| `scheduler/mod.rs:70` | `stop()` 方法未被呼叫 |
| `scheduler/mod.rs:75` | `is_running()` 方法未被呼叫 |
| `scheduler/mod.rs:79` | `execute_now()` 方法未被呼叫 |
| `transfer/mod.rs:15` | `Transfer` variant 未被使用 |
| `transfer/mod.rs:331` | `format_size()` 未被呼叫 |
| `transfer/mod.rs:347` | `format_speed()` 未被呼叫 |

---

### L2: 前端無障礙（Accessibility）不足

- Tab 按鈕使用 emoji（📝🔗🔄📁⏰⚡⚙️）作為唯一圖示，無 alt text
- 切換開關為 `<div onClick>`，無法用鍵盤操作
- 顏色選擇器無文字標籤
- Modal 無 focus trap

---

### L3: 系統匣功能停用中

`main.rs:81` 明確註解 "Tray icon temporarily disabled"，文件中描述的系統匣背景執行功能目前不可用。

---

### L4: 設定頁面版本號寫死

`SettingsTab.tsx` 中版本號寫死為 `"v0.1.0"`，應從後端取得或讀取 package.json。

---

## 7. 前後端 IPC 介面不一致

| IPC 命令 | 問題 |
|----------|------|
| `create_note` | 前端只傳 `title`, `content`；缺少 `color`, `category`, `tags`, `pinned` |
| `toggle_wifi` / `toggle_bluetooth` / `toggle_airplane_mode` | 前端傳參方式為動態拼接 `{ [args[0]]: args[1] }`，型別不安全 |
| `check_all_peers` | 前端假設回傳型別為 `[string, DeviceStatus][]`，未經驗證 |
| `browse_remote_files` | 前端假設回傳結構含 `name`, `path`, `is_dir`, `size`，未驗證 |
| `add_peer` | 前端新建 peer 時 `id` 為空字串，需確認後端是否自動產生 UUID |

---

## 8. 測試覆蓋分析

| 模組 | 單元測試 | 整合測試 | 缺失的測試場景 |
|------|---------|---------|---------------|
| notes | 25 ✅ | 1 ✅ | — |
| p2p | 25 ✅ | 1 ✅ | SSH 實際連線（需 mock）、IP 驗證 |
| sync | 11 ✅ | 1 ✅ | **實際同步操作、衝突解決邏輯** |
| transfer | 16 ✅ | 1 ✅ | **實際傳輸操作、並行限制** |
| scheduler | **0** ❌ | — | **時間範圍計算、跨午夜邏輯、星期過濾** |
| wireless | **0** ❌ | — | 平台相依，難以測試 |
| config | **0** ❌ | — | 序列化/反序列化邊界值 |
| 前端 | **0** ❌ | — | 無任何前端測試 |

**整體測試覆蓋度估算：** 約 55-60%（目標 80%）

**最大缺口：**
1. Scheduler 時間邏輯完全無測試
2. 前端 0 測試
3. 整合測試過淺（不測試真實 SSH/同步/傳輸）

---

## 9. 安全性評估

| 類別 | 狀態 | 說明 |
|------|------|------|
| CSP | ❌ 停用 | `csp: null` — 無 XSS 防護 |
| SQL Injection | ✅ 安全 | 所有查詢使用參數化語句 |
| Command Injection | ❌ 有風險 | `run_custom_command` 無輸入驗證，Shell plugin 無範圍限制 |
| 密碼管理 | ❌ 不安全 | SSH 密碼明文存儲（記憶體和檔案）|
| Plugin Scope | ❌ 未設定 | FS/Shell/Dialog/Notification 無存取範圍 |
| 錯誤訊息洩漏 | ✅ 安全 | 錯誤轉為 String，無敏感資訊外洩 |
| 依賴安全 | ⚠️ 未審查 | 未使用 `cargo-audit` 掃描 |
| Unsafe 程式碼 | ✅ 無 | — |

---

## 10. 效能觀察

| 項目 | 狀態 | 建議 |
|------|------|------|
| SQLite Index | ✅ 有 3 個索引 | — |
| Tokio features | ⚠️ 使用 "full" | 改為選擇性啟用，減小 binary |
| 同步暫存檔 | ⚠️ 寫入磁碟 | 小型 payload 應用記憶體 buffer |
| 傳輸無串流 | ❌ 一次性讀寫 | 應分塊處理以支援大檔案 |
| Release profile | ✅ 最佳化 | LTO + strip + codegen-units=1 |
| 前端 bundle | ✅ Preact | 輕量框架選擇正確 |

---

## 11. 建議修復優先順序

### 第一階段：安全與核心功能（建議立即處理）

| # | 項目 | 對應問題 | 預估工作量 |
|---|------|---------|-----------|
| 1 | 啟用 CSP 安全策略 | C2 | 小 |
| 2 | 設定 Shell plugin scope 白名單 | C4 | 小 |
| 3 | 修復版本號不一致 | H4 | 極小 |
| 4 | 修復 create_note 前端漏傳參數 | H1 | 小 |
| 5 | 加密/保護 SSH 密碼 | C3 | 中 |
| 6 | 設定所有 plugin capability scope | H7 | 中 |

### 第二階段：核心功能補齊

| # | 項目 | 對應問題 | 預估工作量 |
|---|------|---------|-----------|
| 7 | **實作真正的同步引擎** | C1 | 大 |
| 8 | 實作傳輸進度追蹤（分塊讀取） | H2 | 中 |
| 9 | 實作並行傳輸限制（semaphore） | H3 | 小 |
| 10 | 前端錯誤回饋系統（toast） | H5 | 中 |
| 11 | 前端表單驗證 | H6 | 中 |

### 第三階段：品質與測試

| # | 項目 | 對應問題 | 預估工作量 |
|---|------|---------|-----------|
| 12 | Scheduler 模組單元測試 | M1 | 中 |
| 13 | 清理 clippy warnings | M2 | 小 |
| 14 | 清理 dead code | L1 | 小 |
| 15 | 前後端 IPC 介面型別統一 | §7 | 中 |
| 16 | 依賴版本固定 | M7 | 小 |

### 第四階段：體驗優化

| # | 項目 | 對應問題 | 預估工作量 |
|---|------|---------|-----------|
| 17 | 恢復系統匣功能 | L3 | 中 |
| 18 | 前端無障礙改善 | L2 | 中 |
| 19 | CSS 主題一致性 | M3 | 小 |
| 20 | Windows 平台相容性修復 | M8 | 中 |

---

*報告結束*
