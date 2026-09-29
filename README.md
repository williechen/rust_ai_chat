# Rust AI Chat

Rust AI Chat 是以 Rust workspace 管理的實驗性專案，包含 Axum + Leptos 聊天室，以及 Tauri + Leptos 桌面寵物。聊天伺服器目前提供即時房間訊息傳遞；AI 模型、資料庫持久化與遙測尚未整合。

## 專案功能

### 聊天室

- Leptos 提供伺服器端渲染（SSR）與 WebAssembly hydration。
- Axum 提供首頁、聊天室頁面、健康檢查及 WebSocket 端點。
- 使用 UUID 建立聊天室；同一房間的連線會收到即時訊息。
- 訊息和 WebSocket 事件使用共用的序列化型別。

目前聊天室只會廣播使用者訊息，沒有帳號、訊息歷史或 AI 回覆；重新整理後不會保留訊息。

### 桌面寵物

- Tauri 2 桌面視窗搭配 Leptos 前端；視窗為透明、置頂且不可調整大小。
- `pet_domain` 提供寵物狀態機：待機、互動、睡覺，以及相應的狀態轉換和 revision。
- 前端呈現寵物狀態、眨眼與簡單動畫，並提供互動、結束互動、睡覺及醒來操作。
- 內建聊天泡泡目前僅有介面，送出功能尚未啟用。

## 技術架構

| 路徑 | 職責 |
| --- | --- |
| `apps/server` | Axum 伺服器、Leptos SSR、聊天室 WebSocket |
| `apps/desktop_pet` | Tauri 桌面應用程式及寵物狀態管理 |
| `apps/desktop_pet/frontend` | Leptos WebAssembly 桌面前端 |
| `crates/ui` | 聊天室 Leptos UI 與 WebSocket 用戶端 |
| `crates/chat_domain` | 聊天訊息與角色型別 |
| `crates/chat_application` | 聊天訊息建立服務 |
| `crates/shared` | 前後端共用的聊天事件型別 |
| `crates/pet_domain` | 寵物狀態機、命令與快照 |
| `crates/ai_core` | AI 模型介面及 mock 實作，尚未接入聊天室 |
| `crates/persistence`、`crates/telemetry` | 預留模組，目前尚未實作資料持久化或遙測 |

## 開發需求

- Rust stable（edition 2024）
- WebAssembly 目標：`wasm32-unknown-unknown`
- 聊天室開發需要 `cargo-leptos`。
- 桌面前端建置需要 `trunk`；啟動 Tauri 桌面應用程式另需 Tauri CLI 2 及作業系統相依套件。

安裝 Rust 目標及開發工具：

```sh
rustup target add wasm32-unknown-unknown
cargo install cargo-leptos --locked
cargo install trunk --locked
cargo install tauri-cli --version "^2"
```

在 Linux 建置桌面應用程式時，也需要 Tauri/WebKitGTK 等原生套件；CI 使用 `build-essential`、`pkg-config`、`libssl-dev`、`libxdo-dev`、`librsvg2-dev` 及 `libwebkit2gtk-4.1-dev`。

## 執行

### 聊天室伺服器

在專案根目錄執行：

```sh
cargo leptos watch
```

開啟 <http://127.0.0.1:3000>。建立聊天室後，可將 `/room/{UUID}` 網址分享給其他連線者。健康檢查端點為 `/health`。

建立正式版：

```sh
cargo leptos build --release
```

### 桌面寵物

在專案根目錄執行：

```sh
cd apps/desktop_pet
cargo tauri dev
```

Tauri 開發命令會依設定先以 Trunk 啟動前端（`localhost:1420`）。

## 測試與檢查

執行寵物領域單元測試：

```sh
cargo test -p pet-domain
```

CI 也會檢查伺服器 SSR、聊天 UI 的 SSR 與 WebAssembly hydration、原生 workspace，以及桌面 WebAssembly 前端。桌面原生建置需先安裝相應的系統相依套件。

## 設定

伺服器目前綁定 `127.0.0.1:3000`，桌面前端的開發伺服器使用 `localhost:1420`。目前沒有需要設定的環境變數；`.env.example` 為空檔。
