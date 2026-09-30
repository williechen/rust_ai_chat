# Rust AI Chat

Rust AI Chat 是一個以 Rust workspace 組成的學習／實作專案，核心包含：

- **Axum + Leptos 0.8** 的即時聊天室
- **WebSocket per-room broadcast** 即時訊息
- **PostgreSQL + SQLx** 基礎持久化層
- **Tauri 2 + Leptos** 桌面 AI 小寵物
- 可擴充到 **AI Persona、多角色聊天室、streaming、presence、typing、observability** 的共用事件模型

目前專案已完成 Day 1～Day 12 的核心路線；AI 多角色聊天室的資料模型與 WebSocket event contract 已開始預留，但真正的多角色 orchestrator、AI streaming 與訊息持久化仍屬下一階段。

> README 內容以目前 `main` 分支實際程式碼為準，不把 roadmap 中尚未落地的功能列為已完成。

## 目前狀態

### Web 聊天室

已實作：

- Leptos 0.8 SSR + hydration
- Leptos Router：`/`、`/room/:room_id`
- UUID room
- Axum WebSocket：`/ws/{room_id}`
- per-room `RoomHub` broadcast
- shared client/server event protocol
- PostgreSQL `PgPool`
- SQLx migration 啟動流程
- `rooms` table migration
- health check：`/health`

目前聊天室訊息仍只存在記憶體 broadcast 流程中，**尚未寫入 PostgreSQL**，重新整理頁面也不會載入歷史訊息。

### AI 能力

`crates/ai_core` 已建立：

- `ChatModel` trait
- `ChatRequest`
- `ChatResponse`
- `AiError`
- `MockChatModel`

目前 Web 聊天室尚未把 `ChatModel` 接到 WebSocket message flow，因此使用者訊息不會自動產生 AI 回覆。

共用 protocol 已預留：

- `ServerEvent::AiDelta`
- `ServerEvent::AiCompleted`
- `ServerEvent::PresenceChanged`
- `ClientEvent::Typing`

這些 event 目前只是協定層準備，尚未完整實作 server-side 行為。

### AI 多角色聊天室

Domain 已把訊息作者從單純的 user/assistant role 擴充成：

- `AnonymousUser`
- `User { user_id }`
- `Ai { persona_id }`
- `System`

因此一則 AI 訊息可以帶有不同的 `persona_id`，這是未來同一房間存在多個 AI 角色的基礎。

預計的資料流會是：

```text
User Message
    │
    ▼
Room
    │
    ├── Persona A ──► AI Model
    ├── Persona B ──► AI Model
    └── Persona C ──► AI Model
             │
             ▼
      MessageAuthor::Ai
        { persona_id }
             │
             ▼
        Room WebSocket
```

目前尚未實作：

- `AiPersona` 設定模型
- room 與 AI persona membership
- 多 persona 排程／orchestrator
- persona-specific system prompt
- AI streaming 真正串入 WebSocket
- AI 訊息資料庫持久化

因此目前 README 將它列為**架構方向與已具備的基礎能力**，而不是已完成的聊天室功能。

### Desktop AI Pet

Tauri 2 桌面應用目前包含：

- 透明、無邊框、always-on-top 視窗
- Leptos WebAssembly 前端
- `PetMachine` domain state machine
- `Idle` / `Interacting` / `Sleeping`
- command 驅動的狀態轉換
- revision-based snapshot
- Tauri event：`pet://state-changed`
- 眨眼與簡單動畫
- context menu
- `chat_with_pet` Tauri async command
- `MockChatModel` 注入到 Tauri state

目前桌面聊天前端仍尚未完成 bridge：

- Rust/Tauri backend 已有 `chat_with_pet`
- frontend `bridge.rs` 尚未 expose chat command
- ChatBubble 的送出按鈕目前仍 disabled
- 尚未顯示 AI response
- 尚未接真實 AI provider
- 尚未支援 streaming

## 系統架構

```mermaid
flowchart TB
    Browser["Browser / Leptos UI"]
    Router["Leptos Router"]
    WSClient["ChatSocket"]
    Axum["Axum Server"]
    WS["WebSocket Handler"]
    Hub["RoomHub"]
    ChatService["ChatService"]
    Domain["chat_domain"]
    Protocol["shared events"]
    PG["PostgreSQL"]
    Persist["persistence / SQLx"]

    Desktop["Tauri Desktop Pet"]
    PetUI["Leptos WASM Frontend"]
    Bridge["JS / Tauri Bridge"]
    PetDomain["pet_domain"]
    AiCore["ai_core"]
    Mock["MockChatModel"]

    Browser --> Router
    Router --> WSClient
    WSClient <-->|ClientEvent / ServerEvent| WS
    WS --> Axum
    WS --> Hub
    WS --> ChatService
    ChatService --> Domain
    WS --> Protocol

    Axum --> Persist
    Persist --> PG

    Desktop --> PetUI
    PetUI --> Bridge
    Bridge --> Desktop
    Desktop --> PetDomain
    Desktop --> AiCore
    AiCore --> Mock
```

### Workspace 結構

| 路徑 | 職責 |
| --- | --- |
| `apps/server` | Axum server、SSR、WebSocket、DB bootstrap |
| `apps/desktop_pet` | Tauri desktop backend、Pet state、AI command |
| `apps/desktop_pet/frontend` | Leptos WASM desktop UI |
| `crates/ui` | Web Leptos UI、Router、WebSocket client |
| `crates/chat_domain` | Chat message 與 `MessageAuthor` domain |
| `crates/chat_application` | `ChatService` application layer |
| `crates/shared` | Client/Server WebSocket event contract |
| `crates/ai_core` | AI abstraction、`ChatModel`、mock model |
| `crates/persistence` | PostgreSQL connection / SQLx migrations |
| `crates/pet_domain` | Desktop pet state machine |
| `crates/telemetry` | Observability 預留 crate，目前尚未實作 |
| `migrations` | SQLx database migrations |
| `deploy` | nginx / systemd / OTel collector 預留設定 |

## Day 1～Day 12

目前 Day 1～Day 12 對應的主線如下。

| Day | 主題 | Repository 現況 |
| --- | --- | --- |
| Day 1 | Rust workspace | ✅ 多 crate workspace |
| Day 2 | Chat domain / shared protocol | ✅ `ChatMessage`、`MessageAuthor`、Client/Server events |
| Day 3 | Axum HTTP server | ✅ Axum server、`/health` |
| Day 4 | WebSocket 基礎 | ✅ `/ws/{room_id}` |
| Day 5 | ChatService | ✅ application service 建立訊息 |
| Day 6 | Broadcast | ✅ Tokio broadcast |
| Day 7 | Per-room RoomHub | ✅ UUID room 隔離 |
| Day 8 | Leptos reactive UI | ✅ room UI、signals、message list |
| Day 9 | Browser WebSocket adapter | ✅ `ChatSocket` |
| Day 10 | SSR + hydration | ✅ Leptos SSR / WASM hydration |
| Day 11 | Router | ✅ home + room route |
| Day 12 | PostgreSQL + SQLx | ✅ PgPool、migration runner、`rooms` table |

### Day 12 目前做到哪裡？

Server 啟動時會：

```text
DATABASE_URL
    │
    ▼
PgPool::connect
    │
    ▼
persistence::migrate
    │
    ▼
Axum AppState
```

目前 migration 已建立 `rooms` table，但尚未加入：

- messages table
- users table
- AI personas table
- room AI membership table
- message repository
- room repository application flow

所以 PostgreSQL 已進入 runtime bootstrap，但還不是完整 persistence architecture。

## WebSocket Protocol

Client events：

```rust
ClientEvent::SendMessage { room_id, content }
ClientEvent::Typing { room_id, active }
ClientEvent::Ping
```

Server events：

```rust
ServerEvent::MessageCreated(...)
ServerEvent::AiDelta { message_id, delta }
ServerEvent::AiCompleted { message_id }
ServerEvent::PresenceChanged { user_id, online }
ServerEvent::Error { code, message }
ServerEvent::Pong
```

目前真正處理完成的是：

- SendMessage
- MessageCreated
- Ping / Pong
- room mismatch error

Typing、Presence 與 AI streaming 仍待後續實作。

## 開發需求

- Rust stable
- Rust edition 2024
- `wasm32-unknown-unknown`
- PostgreSQL
- `cargo-leptos`
- `trunk`
- Tauri CLI 2
- Linux 桌面開發需 WebKitGTK 等系統套件

Rust toolchain 已由 `rust-toolchain.toml` 設定 stable、rustfmt、clippy 與 WASM target。

安裝主要工具：

```bash
rustup target add wasm32-unknown-unknown
cargo install cargo-leptos --locked
cargo install trunk --locked
cargo install tauri-cli --version "^2"
```

Ubuntu / Debian 的 Tauri build dependencies 可參考 CI：

```bash
sudo apt-get install -y \
  build-essential \
  pkg-config \
  curl \
  wget \
  file \
  libssl-dev \
  libxdo-dev \
  librsvg2-dev \
  libwebkit2gtk-4.1-dev
```

## PostgreSQL 設定

建立 database，例如：

```sql
CREATE DATABASE rust_ai_chat;
```

Server 實際讀取的環境變數名稱是 **`DATABASE_URL`**。

在 repository root 建立 `.env`：

```dotenv
RUST_LOG=debug
DATABASE_URL=postgres://username:password@localhost:5432/rust_ai_chat
```

啟動 server 時會自動呼叫 SQLx migration runner。

## 執行 Web 聊天室

在 repository root：

```bash
cargo leptos watch
```

開啟：

```text
http://127.0.0.1:3000
```

首頁點擊 **Create Room** 後會產生 UUID，並導向：

```text
/room/{UUID}
```

同一 room URL 的多個瀏覽器連線會共享該 room 的即時 broadcast。

Health check：

```text
GET http://127.0.0.1:3000/health
```

Release build：

```bash
cargo leptos build --release
```

## 執行 Desktop AI Pet

```bash
cd apps/desktop_pet
cargo tauri dev
```

Tauri 的 `beforeDevCommand` 會在 `apps/desktop_pet/frontend` 啟動：

```bash
trunk serve --port 1420
```

桌面視窗預設：

- 320 × 360
- transparent
- decorations disabled
- always on top
- non-resizable

## 測試與檢查

常用指令：

```bash
cargo check -p server --features ssr
cargo check -p ui --features ssr
cargo check -p ui --features hydrate --target wasm32-unknown-unknown
cargo check --workspace --exclude desktop-pet-frontend

cargo test -p pet-domain
cargo test -p ai-core
cargo test -p shared
```

Desktop frontend：

```bash
cargo check -p desktop-pet-frontend --target wasm32-unknown-unknown
```

GitHub Actions 目前會檢查：

- server SSR
- UI SSR
- UI WASM hydration
- native workspace
- desktop WASM frontend
- pet domain tests
- release builds

## 已知限制

目前專案仍處於逐步建立架構的階段：

- Web chat 尚未接 AI model
- Web chat message 尚未 persistence
- history 尚未實作
- authentication / session 尚未實作
- typing event 尚未實作 server behavior
- presence 尚未實作
- AI streaming 尚未實作
- multi-persona orchestrator 尚未實作
- telemetry crate 尚未實作
- desktop pet frontend 尚未接 `chat_with_pet`
- desktop pet 尚未接 real AI provider

## 下一階段方向

接下來合理的演進順序是：

1. 建立 Room / Message repository
2. 完成 message persistence 與 history loading
3. 建立 `AiPersona` 與 room membership
4. 把 `ChatModel` 注入 Web chat application flow
5. 完成 `AiDelta` / `AiCompleted` streaming
6. 建立多 AI Persona orchestrator
7. 完成 typing / presence
8. 串接 desktop pet frontend → `chat_with_pet`
9. 接入 real provider
10. 加入 OpenTelemetry tracing

---

此專案的重點不只是完成聊天室，而是逐步建立一套可延伸到 **Web 即時聊天室、AI Persona、多 Agent 對話、桌面 AI 寵物與 observability** 的 Rust application architecture。
